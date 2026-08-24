//! Keyboard shortcuts: the action catalog, default bindings, persistence
//! serialization, the active binding map, and the global key subscription.
//!
//! The global subscription filters on [`event::Status::Ignored`] so a shortcut
//! never fires while a widget (composer text editor, search input, settings
//! text inputs, …) already consumed the key press — i.e. shortcuts are
//! automatically disabled whenever a text area has keyboard focus.

use crate::Message;
use cosmic::iced::Event;
use cosmic::iced::Subscription;
use cosmic::iced::event;
use cosmic::iced::keyboard;
use cosmic::iced::keyboard::Key;
use cosmic::iced::keyboard::key::Named;
pub use cosmic::widget::menu::key_bind::KeyBind;
use cosmic::widget::menu::key_bind::Modifier;
use iced_futures::subscription as iced_sub;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Every action reachable through a global keyboard shortcut.
///
/// Some actions have fixed bindings (they are part of the app's contract with
/// the platform / muscle memory and are not offered for rebinding); see
/// [`ShortcutAction::rebindable`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub enum ShortcutAction {
    ToggleAppSettings,
    ToggleUserSettings,
    ToggleRoomSettings,
    ToggleSpaceSettings,
    ToggleSpaceSwitcher,
    CloseThread,
    Quit,
    Search,
    /// Copy text is performed natively by the focused text widget (composer,
    /// selectable message text). Constellation registers the binding so it is
    /// documented on the shortcuts page; because the dispatch gate requires an
    /// ignored event, the app-level handler is a no-op and never steals
    /// Ctrl+C from a text area.
    CopyText,
    SelectRoomList,
    SelectSpaceSwitcher,
    ScrollUp,
    ScrollDown,
}

impl ShortcutAction {
    pub const ALL: [ShortcutAction; 13] = [
        ShortcutAction::ToggleAppSettings,
        ShortcutAction::ToggleUserSettings,
        ShortcutAction::ToggleRoomSettings,
        ShortcutAction::ToggleSpaceSettings,
        ShortcutAction::ToggleSpaceSwitcher,
        ShortcutAction::CloseThread,
        ShortcutAction::Quit,
        ShortcutAction::Search,
        ShortcutAction::CopyText,
        ShortcutAction::SelectRoomList,
        ShortcutAction::SelectSpaceSwitcher,
        ShortcutAction::ScrollUp,
        ShortcutAction::ScrollDown,
    ];

    /// Whether the user may rebind this action on the shortcuts page.
    pub fn rebindable(self) -> bool {
        !matches!(
            self,
            ShortcutAction::ToggleAppSettings
                | ShortcutAction::ToggleUserSettings
                | ShortcutAction::ToggleRoomSettings
                | ShortcutAction::Quit
                | ShortcutAction::Search
                | ShortcutAction::CopyText
        )
    }

    /// The shipped default binding, if any. `None` means unbound by default.
    pub fn default_keybind(self) -> Option<KeyBind> {
        let (modifiers, key): (&[Modifier], Key) = match self {
            ShortcutAction::ToggleAppSettings => (&[Modifier::Ctrl], char_key(',')),
            ShortcutAction::ToggleUserSettings => (&[Modifier::Ctrl], char_key('u')),
            ShortcutAction::ToggleRoomSettings => (&[Modifier::Ctrl], char_key('r')),
            ShortcutAction::ToggleSpaceSettings => (&[Modifier::Ctrl], char_key('s')),
            ShortcutAction::ToggleSpaceSwitcher => (&[Modifier::Alt], char_key('s')),
            ShortcutAction::CloseThread => (&[], named_key(Named::Escape)),
            ShortcutAction::Quit => (&[Modifier::Ctrl], char_key('q')),
            ShortcutAction::Search => (&[Modifier::Ctrl], char_key('f')),
            ShortcutAction::CopyText => (&[Modifier::Ctrl], char_key('c')),
            ShortcutAction::SelectRoomList => (&[], char_key('r')),
            ShortcutAction::SelectSpaceSwitcher => (&[], char_key('s')),
            ShortcutAction::ScrollUp => (&[], named_key(Named::PageUp)),
            ShortcutAction::ScrollDown => (&[], named_key(Named::PageDown)),
        };
        Some(KeyBind {
            modifiers: modifiers.to_vec(),
            key,
        })
    }
}
fn char_key(c: char) -> Key {
    // `Key::Character` stores a `SmolStr`, which has no `From<char>`.
    Key::Character(c.to_string().into())
}

fn named_key(named: Named) -> Key {
    Key::Named(named)
}

/// String-based serialization of [`KeyBind`] used to persist user overrides in
/// `Config`. We do not serialize `KeyBind` directly so the config schema does
/// not depend on libcosmic's private key representation (`SmolStr`, …).
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct SerializedKeyBind {
    pub modifiers: Vec<String>,
    pub key: String,
}

impl From<&KeyBind> for SerializedKeyBind {
    fn from(kb: &KeyBind) -> Self {
        let mut modifiers: Vec<String> = kb
            .modifiers
            .iter()
            .map(|m| match m {
                Modifier::Ctrl => "ctrl",
                Modifier::Shift => "shift",
                Modifier::Alt => "alt",
                Modifier::Super => "super",
            })
            .map(str::to_string)
            .collect();
        // Sort so the persisted form is stable regardless of insertion order.
        modifiers.sort();
        modifiers.dedup();

        let key = match &kb.key {
            Key::Character(c) => c.as_str().to_string(),
            Key::Named(n) => format!("named:{n:?}"),
            other => format!("{other:?}"),
        };

        Self { modifiers, key }
    }
}

impl SerializedKeyBind {
    pub fn to_keybind(&self) -> Option<KeyBind> {
        if self.is_unbound() {
            return None;
        }
        let modifiers = self
            .modifiers
            .iter()
            .filter_map(|m| match m.as_str() {
                "ctrl" => Some(Modifier::Ctrl),
                "shift" => Some(Modifier::Shift),
                "alt" => Some(Modifier::Alt),
                "super" => Some(Modifier::Super),
                _ => None,
            })
            .collect::<Vec<_>>();

        let key = parse_key(&self.key)?;

        Some(KeyBind { modifiers, key })
    }

    /// An empty binding marks an action as intentionally unbound.
    pub fn is_unbound(&self) -> bool {
        self.modifiers.is_empty() && self.key.is_empty()
    }
}

fn parse_key(s: &str) -> Option<Key> {
    if let Some(named) = s.strip_prefix("named:") {
        return parse_named(named);
    }
    // Single character keys are stored verbatim ("r", ",", "?", …).
    let mut chars = s.chars();
    let first = chars.next()?;
    if chars.next().is_none() && !first.is_control() {
        Some(char_key(first))
    } else {
        None
    }
}

fn parse_named(s: &str) -> Option<Key> {
    // Whitelist of Named variants we support for shortcuts. Extend when a new
    // default or user-recorded binding needs one.
    let named = match s {
        "Escape" => Named::Escape,
        "Enter" => Named::Enter,
        "Tab" => Named::Tab,
        "Backspace" => Named::Backspace,
        "Delete" => Named::Delete,
        "Insert" => Named::Insert,
        "Home" => Named::Home,
        "End" => Named::End,
        "PageUp" => Named::PageUp,
        "PageDown" => Named::PageDown,
        "ArrowUp" => Named::ArrowUp,
        "ArrowDown" => Named::ArrowDown,
        "ArrowLeft" => Named::ArrowLeft,
        "ArrowRight" => Named::ArrowRight,
        "F1" => Named::F1,
        "F2" => Named::F2,
        "F3" => Named::F3,
        "F4" => Named::F4,
        "F5" => Named::F5,
        "F6" => Named::F6,
        "F7" => Named::F7,
        "F8" => Named::F8,
        "F9" => Named::F9,
        "F10" => Named::F10,
        "F11" => Named::F11,
        "F12" => Named::F12,
        _ => return None,
    };
    Some(named_key(named))
}

/// Render a `KeyBind` for display in the UI.
///
/// `KeyBind` has no `Display` impl that reads well: bare `Key::Character`
/// renders Space as an invisible glyph and modifier names as Rust debug
/// variants. Special-case the unreadable ones.
pub fn format_keybind(kb: &KeyBind) -> String {
    fn key_label(key: &Key) -> String {
        match key {
            Key::Character(c) if c.as_str() == " " => "Space".to_string(),
            Key::Character(c) => c.to_uppercase(),
            Key::Named(n) => match n {
                Named::Escape => "Esc".to_string(),
                Named::PageUp => "Page Up".to_string(),
                Named::PageDown => "Page Down".to_string(),
                Named::ArrowUp => "\u{2191}".to_string(),
                Named::ArrowDown => "\u{2193}".to_string(),
                Named::ArrowLeft => "\u{2190}".to_string(),
                Named::ArrowRight => "\u{2192}".to_string(),
                other => format!("{other:?}"),
            },
            other => format!("{other:?}"),
        }
    }

    let mut parts: Vec<String> = kb
        .modifiers
        .iter()
        .map(|m| match m {
            Modifier::Ctrl => "Ctrl".to_string(),
            Modifier::Shift => "Shift".to_string(),
            Modifier::Alt => "Alt".to_string(),
            Modifier::Super => "Super".to_string(),
        })
        .collect();
    parts.push(key_label(&kb.key));
    parts.join(" + ")
}

/// The active bindings: a map built from defaults merged with user overrides,
/// plus a version counter so the subscription restarts when bindings change.
#[derive(Clone, Debug)]
pub struct Bindings {
    /// Active `KeyBind` → action. Mutate only via `set` / `reset_to_default`
    /// so `version` stays in sync.
    map: HashMap<KeyBind, ShortcutAction>,
    /// Bumped on every mutation; included in the subscription identity hash so
    /// rebinds take effect without an app restart.
    pub version: u64,
}

impl Default for Bindings {
    fn default() -> Self {
        Self::defaults()
    }
}

impl Bindings {
    /// Defaults only — never an empty map, which would leave the app without
    /// any keyboard control.
    pub fn defaults() -> Self {
        let mut map = HashMap::new();
        for action in ShortcutAction::ALL {
            if let Some(kb) = action.default_keybind() {
                let replaced = map.insert(kb, action);
                debug_assert!(replaced.is_none(), "duplicate default binding");
            }
        }
        Self { map, version: 0 }
    }

    /// Merge user overrides over the defaults. An override whose serialized
    /// form is empty/unparsable unbinds the action.
    pub fn with_overrides(overrides: &HashMap<ShortcutAction, SerializedKeyBind>) -> Self {
        let mut b = Self::defaults();
        for (&action, ser) in overrides {
            b.set(action, ser.to_keybind());
        }
        b
    }

    /// Consume `self` into the raw map for the subscription closure, which
    /// owns the map for the lifetime of the subscription.
    pub(crate) fn into_map(self) -> HashMap<KeyBind, ShortcutAction> {
        self.map
    }

    pub fn keybind_for(&self, action: ShortcutAction) -> Option<&KeyBind> {
        self.map.iter().find(|(_, a)| **a == action).map(|(k, _)| k)
    }

    /// The action currently bound to `kb`, if any.
    pub fn conflict_for(&self, kb: &KeyBind, ignore: ShortcutAction) -> Option<ShortcutAction> {
        match self.map.get(kb) {
            Some(&a) if a != ignore => Some(a),
            _ => None,
        }
    }

    /// Bind `action` to `kb` (`None` = unbound). Any other action holding the
    /// same combo loses its binding first.
    pub fn set(&mut self, action: ShortcutAction, kb: Option<KeyBind>) {
        self.map.retain(|_, a| *a != action);
        if let Some(kb) = kb {
            self.map.insert(kb, action);
        }
        self.version += 1;
    }

    pub fn reset_to_default(&mut self, action: ShortcutAction) {
        self.set(action, action.default_keybind());
    }
}

/// What list, if any, currently holds the keyboard selection
/// ("Selection to Rooms" / "Selection to Space Switcher" shortcuts).
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum SelectionMode {
    None,
    Rooms,
    Spaces,
}

/// Global keyboard-shortcut subscription.
///
/// Filters on [`event::Status::Ignored`] so shortcuts never fire when a widget
/// (text editor, text input, focused button) consumed the event first. While a
/// list selection mode is active, the arrow/Enter/Escape keys drive the
/// selection instead of matching bindings.
pub fn subscription(bindings: &Bindings, mode: SelectionMode) -> Subscription<Message> {
    #[derive(Hash)]
    struct ShortcutsId {
        version: u64,
        mode: SelectionMode,
    }

    let id = ShortcutsId {
        version: bindings.version,
        mode,
    };
    let map = bindings.clone().into_map();

    iced_sub::filter_map(id, move |event| {
        let iced_sub::Event::Interaction { event, status, .. } = event else {
            return None;
        };
        if status != event::Status::Ignored {
            return None;
        }
        let Event::Keyboard(keyboard::Event::KeyPressed {
            key,
            modified_key,
            physical_key,
            modifiers,
            ..
        }) = event
        else {
            return None;
        };

        // Contextual keys while a list-selection mode is active. These are
        // deliberately not user-rebindable.
        if mode != SelectionMode::None {
            return selection_message(&key, &modifiers);
        }

        // Match against `modified_key` (layout-aware), falling back to the raw
        // key and then the physical key position for non-Latin layouts.
        let phys = Some(&physical_key);
        let action = map
            .iter()
            .find(|(kb, _)| {
                kb.matches(modifiers, &modified_key, phys) || kb.matches(modifiers, &key, phys)
            })
            .map(|(_, a)| *a)?;

        Some(Message::ShortcutTriggered(action))
    })
}

fn selection_message(key: &Key, modifiers: &keyboard::Modifiers) -> Option<Message> {
    // Only plain arrows navigate (ignore e.g. Ctrl+arrow).
    if *modifiers != keyboard::Modifiers::empty() {
        return None;
    }
    let delta: i32 = match key {
        Key::Named(Named::ArrowUp) | Key::Named(Named::ArrowLeft) => -1,
        Key::Named(Named::ArrowDown) | Key::Named(Named::ArrowRight) => 1,
        Key::Named(Named::Home) => i32::MIN,
        Key::Named(Named::End) => i32::MAX,
        Key::Named(Named::Enter) => return Some(Message::SelectionCommit),
        Key::Named(Named::Escape) => return Some(Message::SelectionCancel),
        _ => return None,
    };
    Some(Message::SelectionMove(delta))
}

/// Active only while a shortcut recording dialog is open. Captures the next
/// non-modifier key press as the recorded combo; Escape cancels.
///
/// Unlike [`subscription`] this does NOT gate on `event::Status::Ignored`:
/// the recorder must be able to capture keys that focused widgets would
/// normally consume. Escape itself cannot be bound — it always cancels.
pub fn capture_subscription() -> Subscription<Message> {
    #[derive(Hash)]
    struct CaptureId;

    iced_sub::filter_map(CaptureId, |event| {
        let iced_sub::Event::Interaction { event, .. } = event else {
            return None;
        };
        let Event::Keyboard(keyboard::Event::KeyPressed { key, modifiers, .. }) = event else {
            return None;
        };

        if matches!(key, Key::Named(Named::Escape)) {
            return Some(Message::Shortcuts(
                crate::settings::shortcuts::Message::CancelRecording,
            ));
        }

        // Wait out presses of bare modifiers until the real key arrives.
        if matches!(
            &key,
            Key::Named(Named::Control | Named::Shift | Named::Alt | Named::Super | Named::Meta)
        ) {
            return None;
        }

        let combo = KeyBind {
            modifiers: modifier_vec(modifiers),
            key: key.clone(),
        };
        Some(Message::Shortcuts(
            crate::settings::shortcuts::Message::RecordingCaptured(combo),
        ))
    })
}

fn modifier_vec(m: keyboard::Modifiers) -> Vec<Modifier> {
    let mut v = Vec::new();
    if m.control() {
        v.push(Modifier::Ctrl);
    }
    if m.shift() {
        v.push(Modifier::Shift);
    }
    if m.alt() {
        v.push(Modifier::Alt);
    }
    if m.logo() {
        v.push(Modifier::Super);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_are_unique_and_complete() {
        let b = Bindings::defaults();
        assert_eq!(b.map.len(), ShortcutAction::ALL.len());
        for action in ShortcutAction::ALL {
            assert!(b.keybind_for(action).is_some(), "{action:?} unbound");
        }
    }

    #[test]
    fn rebindable_split_matches_issue() {
        // Fixed per issue #425.
        for fixed in [
            ShortcutAction::ToggleAppSettings,
            ShortcutAction::ToggleUserSettings,
            ShortcutAction::ToggleRoomSettings,
            ShortcutAction::Quit,
            ShortcutAction::Search,
            ShortcutAction::CopyText,
        ] {
            assert!(!fixed.rebindable(), "{fixed:?} should be fixed");
        }
        for changeable in [
            ShortcutAction::ToggleSpaceSettings,
            ShortcutAction::ToggleSpaceSwitcher,
            ShortcutAction::CloseThread,
            ShortcutAction::SelectRoomList,
            ShortcutAction::SelectSpaceSwitcher,
            ShortcutAction::ScrollUp,
            ShortcutAction::ScrollDown,
        ] {
            assert!(
                changeable.rebindable(),
                "{changeable:?} should be changeable"
            );
        }
    }

    #[test]
    fn serialization_round_trip() {
        for action in ShortcutAction::ALL {
            let kb = action.default_keybind().unwrap();
            let ser = SerializedKeyBind::from(&kb);
            assert_eq!(ser.to_keybind().as_ref(), Some(&kb));
        }
    }

    #[test]
    fn empty_serialized_is_unbound() {
        let ser = SerializedKeyBind::default();
        assert!(ser.is_unbound());
        assert!(ser.to_keybind().is_none());
    }

    #[test]
    fn override_replaces_default_and_unbinding_removes_it() {
        let new_kb = KeyBind {
            modifiers: vec![Modifier::Ctrl],
            key: char_key('j'),
        };

        let mut overrides = HashMap::new();
        overrides.insert(ShortcutAction::ScrollUp, SerializedKeyBind::from(&new_kb));
        let b = Bindings::with_overrides(&overrides);
        assert_eq!(b.keybind_for(ShortcutAction::ScrollUp), Some(&new_kb));

        // Unbinding: empty serialized value removes the default binding.
        let mut overrides = HashMap::new();
        overrides.insert(ShortcutAction::CloseThread, SerializedKeyBind::default());
        let b = Bindings::with_overrides(&overrides);
        assert!(b.keybind_for(ShortcutAction::CloseThread).is_none());
    }

    #[test]
    fn set_steals_combo_from_conflicting_action() {
        let mut b = Bindings::defaults();
        let stolen = ShortcutAction::SelectSpaceSwitcher
            .default_keybind()
            .unwrap();
        b.set(ShortcutAction::ScrollUp, Some(stolen.clone()));
        assert_eq!(b.keybind_for(ShortcutAction::ScrollUp), Some(&stolen));
        assert!(b.keybind_for(ShortcutAction::SelectSpaceSwitcher).is_none());
        assert_eq!(b.conflict_for(&stolen, ShortcutAction::ScrollUp), None);
    }

    #[test]
    fn set_bumps_version() {
        let mut b = Bindings::defaults();
        let v0 = b.version;
        b.reset_to_default(ShortcutAction::ScrollUp);
        assert_eq!(b.version, v0 + 1);
    }

    #[test]
    fn format_renders_named_keys_readably() {
        let esc = ShortcutAction::CloseThread.default_keybind().unwrap();
        assert_eq!(format_keybind(&esc), "Esc");
        let page_down = ShortcutAction::ScrollDown.default_keybind().unwrap();
        assert_eq!(format_keybind(&page_down), "Page Down");
        let ctrl_comma = ShortcutAction::ToggleAppSettings.default_keybind().unwrap();
        assert_eq!(format_keybind(&ctrl_comma), "Ctrl + ,");
    }

    #[test]
    fn parse_named_whitelist() {
        assert!(matches!(
            parse_key("named:PageUp"),
            Some(Key::Named(Named::PageUp))
        ));
        assert!(parse_key("named:Bogus").is_none());
        assert_eq!(parse_key("r"), Some(char_key('r')));
        assert!(parse_key("ctrl").is_none());
    }
}
