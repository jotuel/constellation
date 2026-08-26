//! Anchor-based timeline scrolling.
//!
//! Pixel offsets are not stable across reflow: when the room width changes
//! (window resize, members/pinned panel toggle), text re-wraps and every
//! absolute Y points somewhere else. Instead of remembering raw pixels, the
//! position is remembered as an *anchor*: which tagged timeline row the
//! viewport top sits in, and how far into that row.
//!
//! Row geometry is measured from the real widget tree with a small
//! [`Operation`] (`measure_timeline_task`) whenever it may have changed. Each
//! rendered row is wrapped in a container whose widget id encodes a stable key
//! (see `row_id` / `ConstellationItem::scroll_key`), so anchors survive
//! prepends and room switches.

use crate::{Message, THREADED_TIMELINE_ID, TIMELINE_ID};
use cosmic::{
    Action, Task,
    iced::{
        core::{
            Rectangle, Vector,
            widget::operation::{Operation, Outcome, Scrollable},
        },
        runtime::task,
        widget::Id,
    },
};

/// Prefix for main-timeline row ids. Part of the widget id string, so the
/// measuring operation can pick rows out of the whole tree.
pub(crate) const MAIN_ROW_PREFIX: &str = "tl|";
/// Prefix for threaded-timeline row ids.
pub(crate) const THREAD_ROW_PREFIX: &str = "ttl|";

/// Maximum number of times a pending reflow restore will re-request a
/// measurement that arrived stale (mid-resize) before giving up and keeping
/// whatever offset iced landed on.
pub(crate) const MAX_REFLOW_ATTEMPTS: u8 = 4;

/// Tolerance when comparing layout dimensions; sub-pixel drift is noise.
const DIM_EPSILON: f32 = 0.5;

/// Whether a measurement reflects geometry at least as new as everything
/// observed so far on this timeline.
pub(crate) fn measurement_is_fresh(
    tracker: &ScrollTracker,
    viewport_width: f32,
    content_height: f32,
) -> bool {
    content_height > 0.0
        && (content_height - tracker.observed_content_height).abs() <= DIM_EPSILON
        && (viewport_width - tracker.observed_viewport_width).abs() <= DIM_EPSILON
}

/// A resolved scroll anchor: the row key and the Y distance from the row's
/// top edge to the viewport top.
pub(crate) type Anchor = (String, f32);
/// How to restore the viewport after a reflow invalidated pixel offsets.
#[derive(Debug, Clone)]
pub(crate) enum PendingReflow {
    /// Restore relative to a measured row; if the row is gone by apply time
    /// (history unloaded underneath us), fall back to a content ratio.
    Anchor {
        key: String,
        intra_y: f32,
        fallback_ratio: f32,
    },
    /// No usable measurement existed; preserve proportional position.
    Ratio(f32),
}

/// Per-timeline scroll bookkeeping for anchored scrolling.
#[derive(Debug, Default)]
pub(crate) struct ScrollTracker {
    /// Measured rows: `(key, content-space Y of row top)` sorted ascending.
    /// Empty until the first successful measurement.
    pub children: Vec<(String, f32)>,
    /// Content height the `children` snapshot was measured at.
    pub children_content_height: f32,
    /// Viewport width the `children` snapshot was measured at.
    pub children_viewport_width: f32,
    /// Most recently observed geometry from `TimelineScrolled`, used to judge
    /// whether an in-flight measurement is already stale.
    pub observed_content_height: f32,
    pub observed_viewport_width: f32,
    /// A measurement request is in flight; don't queue another.
    pub measure_pending: bool,
    /// Reflow detected; resolved on the next fresh measurement.
    pub pending_reflow: Option<PendingReflow>,
    /// Re-request counter guarding the stale-measurement loop.
    pub reflow_attempts: u8,
    /// A layout-affecting toggle remounted the pane; trust the next
    /// measurement even if observed geometry hasn't caught up yet.
    pub expect_relayout: bool,
    /// A deferred measurement is pending; the restore tick fires it once the
    /// deadline passes.
    pub delayed_scheduled: bool,
    pub measure_deadline: Option<std::time::Instant>,
    /// An END re-snap is pending for a bottom-parked timeline.
    pub end_snap_scheduled: bool,
    pub end_snap_deadline: Option<std::time::Instant>,
}

impl ScrollTracker {
    pub fn reset(&mut self) {
        *self = Self {
            observed_content_height: self.observed_content_height,
            observed_viewport_width: self.observed_viewport_width,
            ..Self::default()
        };
    }
    pub fn note_observed(&mut self, viewport_width: f32, content_height: f32) {
        self.observed_viewport_width = viewport_width;
        self.observed_content_height = content_height;
    }

    /// Replace the measured snapshot.
    pub fn store(
        &mut self,
        children: Vec<(String, f32)>,
        content_height: f32,
        viewport_width: f32,
    ) {
        self.children = children;
        self.children_content_height = content_height;
        self.children_viewport_width = viewport_width;
    }

    /// Shift all cached row positions down by `dy` (content prepended above).
    pub fn shift(&mut self, dy: f32) {
        for (_, y) in &mut self.children {
            *y += dy;
        }
        self.children_content_height += dy;
    }

    /// Whether the measured snapshot no longer matches observed geometry.
    pub fn is_stale(&self) -> bool {
        self.children.is_empty()
            || (self.children_content_height - self.observed_content_height).abs() > DIM_EPSILON
            || (self.children_viewport_width - self.observed_viewport_width).abs() > DIM_EPSILON
    }

    /// Content-space Y of a row key in the current snapshot.
    pub fn y_of(&self, key: &str) -> Option<f32> {
        self.children
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, y)| *y)
    }
}

/// Decode the anchor for a viewport-top pixel offset against a measured
/// snapshot. Returns `None` when no usable snapshot exists.
pub(crate) fn decode_anchor(offset: f32, children: &[(String, f32)]) -> Option<Anchor> {
    let first = children.first()?;
    let idx = children.partition_point(|(_, y)| *y <= offset);
    if idx == 0 {
        // Above the first row (top padding/gap): pin to the first row.
        return Some((first.0.clone(), offset - first.1));
    }
    let (key, y) = &children[idx - 1];
    Some((key.clone(), offset - y))
}

/// Plan how to preserve the current view across a reflow.
///
/// `offset`/`measured_content_height` describe the pre-reflow geometry (the
/// last stable pixel state plus the snapshot it can be decoded against);
/// `new_content_height` is only used for the ratio fallback.
pub(crate) fn plan_reflow(
    offset: f32,
    children: &[(String, f32)],
    measured_content_height: f32,
    new_content_height: f32,
) -> Option<PendingReflow> {
    if measured_content_height <= 0.0 || new_content_height <= 0.0 {
        return None;
    }
    let fallback_ratio = (offset / measured_content_height).clamp(0.0, 1.0);
    Some(match decode_anchor(offset, children) {
        Some((key, intra_y)) => PendingReflow::Anchor {
            key,
            intra_y,
            fallback_ratio,
        },
        None => PendingReflow::Ratio(fallback_ratio),
    })
}

/// Resolve a pending reflow into a target content-space Y offset against a
/// freshly measured snapshot.
pub(crate) fn resolve_reflow(pending: &PendingReflow, tracker: &ScrollTracker) -> Option<f32> {
    match pending {
        PendingReflow::Anchor {
            key,
            intra_y,
            fallback_ratio,
        } => tracker
            .y_of(key)
            .map(|y| y + intra_y)
            .or_else(|| Some(fallback_ratio * tracker.observed_content_height)),
        PendingReflow::Ratio(ratio) => Some(ratio * tracker.observed_content_height),
    }
}

/// Widget id for a timeline row. `prefix` is [`MAIN_ROW_PREFIX`] or
/// [`THREAD_ROW_PREFIX`]; `key` identifies the item (see
/// `ConstellationItem::scroll_key`) or a synthesized date divider
/// (`d:<unix_secs>`).
pub(crate) fn row_id(prefix: &str, key: &str) -> Id {
    Id::new(format!("{prefix}{key}"))
}

/// Raw result of measuring one timeline's rows.
struct Measurement {
    rows: Vec<(String, f32)>,
    viewport_width: f32,
    content_height: f32,
    origin_y: Option<f32>,
}

struct MeasureTimeline {
    prefix: &'static str,
    timeline_id: Id,
    origin_y: Option<f32>,
    viewport_width: f32,
    content_height: f32,
    rows: Vec<(String, f32)>,
}

impl Operation<Measurement> for MeasureTimeline {
    fn traverse(&mut self, operate: &mut dyn FnMut(&mut dyn Operation<Measurement>)) {
        operate(self);
    }

    fn container(&mut self, id: Option<&Id>, bounds: Rectangle) {
        let Some(id) = id else { return };
        let text = id.to_string();
        if text.starts_with(self.prefix) {
            // Raw layout bounds are window-absolute; converted to content
            // space once the scrollable origin is known.
            self.rows.push((text, bounds.y));
        }
    }

    fn scrollable(
        &mut self,
        id: Option<&Id>,
        bounds: Rectangle,
        content_bounds: Rectangle,
        _translation: Vector,
        _state: &mut dyn Scrollable,
    ) {
        if id.is_some_and(|id| id == &self.timeline_id) {
            // Layout places the content starting at the scrollable's own top
            // edge; subtracting this origin yields pure content-space Y.
            self.origin_y = Some(bounds.y);
            self.viewport_width = bounds.width;
            self.content_height = content_bounds.height;
        }
    }

    fn finish(&self) -> Outcome<Measurement> {
        Outcome::Some(Measurement {
            rows: self.rows.clone(),
            viewport_width: self.viewport_width,
            content_height: self.content_height,
            origin_y: self.origin_y,
        })
    }
}
/// Queue an operation that measures the given timeline's rows on the current
/// widget tree and reports them back via [`Message::TimelineMeasured`].
pub(crate) fn measure_timeline_task(is_thread: bool, generation: u64) -> Task<Action<Message>> {
    let (prefix, timeline_id) = if is_thread {
        (THREAD_ROW_PREFIX, THREADED_TIMELINE_ID.clone())
    } else {
        (MAIN_ROW_PREFIX, TIMELINE_ID.clone())
    };
    let op = MeasureTimeline {
        prefix,
        timeline_id,
        origin_y: None,
        viewport_width: 0.0,
        content_height: 0.0,
        rows: Vec::new(),
    };
    task::widget(op).map(move |m| {
        let mut rows = m.rows;
        if let Some(origin) = m.origin_y {
            for (_, y) in rows.iter_mut() {
                *y -= origin;
            }
        }
        rows.sort_by(|a, b| a.1.total_cmp(&b.1));
        Action::from(Message::TimelineMeasured {
            is_thread,
            generation,
            viewport_width: m.viewport_width,
            content_height: m.content_height,
            rows,
        })
    })
}

/// Access the tracker for the given timeline.
pub(crate) fn tracker_mut(app: &mut crate::Constellation, is_thread: bool) -> &mut ScrollTracker {
    if is_thread {
        &mut app.scroll_thread
    } else {
        &mut app.scroll_main
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rows() -> Vec<(String, f32)> {
        vec![
            ("tl|a".to_string(), 0.0),
            ("tl|b".to_string(), 100.0),
            ("tl|c".to_string(), 250.0),
        ]
    }

    #[test]
    fn decode_anchor_inside_row() {
        let anchor = decode_anchor(120.0, &rows()).unwrap();
        assert_eq!(anchor, ("tl|b".to_string(), 20.0));
    }

    #[test]
    fn decode_anchor_in_gap_maps_to_upper_row() {
        // 260 lies past b's top but before c's top: attributed to c.
        let anchor = decode_anchor(260.0, &rows()).unwrap();
        assert_eq!(anchor, ("tl|c".to_string(), 10.0));
    }

    #[test]
    fn decode_anchor_above_first_pins_to_first() {
        let anchor = decode_anchor(-30.0, &rows()).unwrap();
        assert_eq!(anchor, ("tl|a".to_string(), -30.0));
    }

    #[test]
    fn decode_anchor_empty_snapshot_is_none() {
        assert!(decode_anchor(10.0, &[]).is_none());
    }

    #[test]
    fn plan_reflow_without_measurement_falls_back_to_ratio() {
        let plan = plan_reflow(400.0, &[], 800.0, 1000.0).unwrap();
        match plan {
            PendingReflow::Ratio(ratio) => assert!((ratio - 0.5).abs() < f32::EPSILON),
            _ => panic!("expected ratio fallback"),
        }
    }

    #[test]
    fn plan_reflow_needs_positive_heights() {
        assert!(plan_reflow(0.0, &rows(), 0.0, 100.0).is_none());
        assert!(plan_reflow(0.0, &rows(), 100.0, 0.0).is_none());
    }

    #[test]
    fn resolve_prefers_measured_row_over_fallback() {
        let mut tracker = ScrollTracker::default();
        tracker.store(rows(), 350.0, 500.0);
        tracker.note_observed(500.0, 350.0);
        let plan = PendingReflow::Anchor {
            key: "tl|b".to_string(),
            intra_y: 15.0,
            fallback_ratio: 0.9,
        };
        assert!((resolve_reflow(&plan, &tracker).unwrap() - 115.0).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_uses_fallback_when_row_vanished() {
        let mut tracker = ScrollTracker::default();
        tracker.store(vec![("tl|z".to_string(), 0.0)], 300.0, 500.0);
        tracker.note_observed(500.0, 300.0);
        let plan = PendingReflow::Anchor {
            key: "tl|b".to_string(),
            intra_y: 15.0,
            fallback_ratio: 0.9,
        };
        assert!((resolve_reflow(&plan, &tracker).unwrap() - 270.0).abs() < f32::EPSILON);
    }

    #[test]
    fn shift_moves_rows_and_height() {
        let mut tracker = ScrollTracker::default();
        tracker.store(rows(), 300.0, 500.0);
        tracker.shift(120.0);
        assert_eq!(tracker.children[1], ("tl|b".to_string(), 220.0));
        assert!((tracker.children_content_height - 420.0).abs() < f32::EPSILON);
    }

    #[test]
    fn measurement_freshness_tracks_observed_dims() {
        let mut tracker = ScrollTracker::default();
        tracker.note_observed(600.0, 900.0);
        assert!(measurement_is_fresh(&tracker, 600.0, 900.0));
        assert!(measurement_is_fresh(&tracker, 600.3, 900.3));
        assert!(!measurement_is_fresh(&tracker, 500.0, 900.0));
        assert!(!measurement_is_fresh(&tracker, 600.0, 902.0));
        assert!(!measurement_is_fresh(&tracker, 600.0, 0.0));
    }
}
