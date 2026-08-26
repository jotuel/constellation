use crate::constellation::scroll;
use crate::matrix::{self, TimelineItem};
use crate::{
    ApplyVectorDiffExt, Constellation, ConstellationItem, MediaSource, Message,
    THREADED_TIMELINE_ID, TIMELINE_ID,
};
use cosmic::iced::widget::scrollable;
use cosmic::{Action, Application, Task};
use futures::FutureExt;
use futures::stream::StreamExt;
use matrix_sdk::ruma::OwnedEventId;
use matrix_sdk::ruma::events::room::message::MessageType;
use matrix_sdk_ui::timeline::TimelineDetails;
use std::sync::Arc;

type PinnedOutput =
    std::pin::Pin<Box<dyn Future<Output = (String, Result<Vec<u8>, String>)> + Send + 'static>>;

impl Constellation {
    pub fn restore_scroll_task(&self) -> Task<Action<Message>> {
        if self.active_thread_root.is_some() {
            if self.is_threaded_timeline_at_bottom {
                scrollable::snap_to(
                    THREADED_TIMELINE_ID.clone(),
                    scrollable::RelativeOffset::END.into(),
                )
            } else {
                scrollable::scroll_to(
                    THREADED_TIMELINE_ID.clone(),
                    scrollable::AbsoluteOffset {
                        x: Some(0.0),
                        y: Some(self.last_threaded_timeline_offset),
                    },
                )
            }
        } else {
            if self.is_timeline_at_bottom {
                scrollable::snap_to(TIMELINE_ID.clone(), scrollable::RelativeOffset::END.into())
            } else {
                scrollable::scroll_to(
                    TIMELINE_ID.clone(),
                    scrollable::AbsoluteOffset {
                        x: Some(0.0),
                        y: Some(self.last_timeline_offset),
                    },
                )
            }
        }
    }

    pub fn recompute_timeline_metadata(&mut self) {
        self.thread_counts.clear();
        self.event_id_to_index.clear();
        self.thread_root_to_last_index.clear();

        for (index, item) in self.timeline_items.iter().enumerate() {
            if let Some(inner) = item.item.as_ref()
                && let Some(event) = inner.as_event()
            {
                if let Some(event_id) = event.event_id() {
                    self.event_id_to_index.insert(event_id.into(), index);
                }
                if let Some(root_id) = item.thread_root_id.clone() {
                    *self.thread_counts.entry(root_id.clone()).or_insert(0) += 1;
                    self.thread_root_to_last_index.insert(root_id, index);
                }
            }
        }
    }

    pub fn fetch_missing_media(&mut self) -> Task<Action<Message>> {
        let mut media_fetches: Vec<PinnedOutput> = Vec::new();
        #[cfg(feature = "video-player")]
        let mut autoplay_requests: Vec<(MediaSource, String, String)> = Vec::new();

        let matrix = match &self.matrix {
            Some(m) => m.clone(),
            None => return Task::none(),
        };

        let mut check_item = |item: &Arc<TimelineItem>, fetches: &mut Vec<_>| {
            if let Some(event) = item.as_event() {
                // Fetch avatar
                if let TimelineDetails::Ready(profile) = event.sender_profile()
                    && let Some(avatar_url) = &profile.avatar_url
                {
                    let url_str = avatar_url.to_string();
                    if !self.media_cache.contains_key(&url_str) {
                        let matrix_clone = matrix.clone();
                        let source = MediaSource::Plain(avatar_url.clone());
                        fetches.push(
                            async move {
                                let res = matrix_clone
                                    .fetch_media(source)
                                    .await
                                    .map_err(|e| e.to_string());
                                (url_str, res)
                            }
                            .boxed(),
                        );
                    }
                }

                if !self.user_settings.media_previews_display_policy {
                    return;
                }
                let Some(message) = event.content().as_message() else {
                    return;
                };

                match message.msgtype() {
                    MessageType::Image(image) => {
                        let mxc_url = match &image.source {
                            MediaSource::Plain(uri) => uri.to_string(),
                            MediaSource::Encrypted(file) => file.url.to_string(),
                        };
                        if !self.media_cache.contains_key(&mxc_url) {
                            let matrix_clone = matrix.clone();
                            let source = image.source.clone();
                            fetches.push(
                                async move {
                                    let res = matrix_clone
                                        .fetch_media(source)
                                        .await
                                        .map_err(|e| e.to_string());
                                    (mxc_url, res)
                                }
                                .boxed(),
                            );
                        }
                    }
                    MessageType::Video(video) => {
                        // Fetch the video thumbnail so the play button has a preview.
                        if let Some(info) = &video.info
                            && let Some(thumb_source) = &info.thumbnail_source
                        {
                            let thumb_url = match thumb_source {
                                MediaSource::Plain(uri) => uri.to_string(),
                                MediaSource::Encrypted(file) => file.url.to_string(),
                            };
                            if !self.media_cache.contains_key(&thumb_url) {
                                let matrix_clone = matrix.clone();
                                let source = thumb_source.clone();
                                fetches.push(
                                    async move {
                                        let res = matrix_clone
                                            .fetch_media(source)
                                            .await
                                            .map_err(|e| e.to_string());
                                        (thumb_url, res)
                                    }
                                    .boxed(),
                                );
                            }
                        }
                        // Queue autoplay if enabled.
                        #[cfg(feature = "video-player")]
                        if self.app_settings.autoplay_videos {
                            let mxc_url = match &video.source {
                                MediaSource::Plain(uri) => uri.to_string(),
                                MediaSource::Encrypted(file) => file.url.to_string(),
                            };
                            if !self.video_cache.contains_key(&mxc_url)
                                && !self.loading_videos.contains(&mxc_url)
                            {
                                autoplay_requests.push((
                                    video.source.clone(),
                                    mxc_url,
                                    video.body.clone(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        };

        for item in &self.timeline_items {
            if let Some(t_item) = &item.item {
                check_item(t_item, &mut media_fetches);
            }
        }
        for item in &self.threaded_timeline_items {
            if let Some(t_item) = &item.item {
                check_item(t_item, &mut media_fetches);
            }
        }

        let mut tasks: Vec<Task<Action<Message>>> = Vec::new();
        if !media_fetches.is_empty() {
            tasks.push(Task::perform(
                async move {
                    futures::stream::iter(media_fetches)
                        .buffer_unordered(10)
                        .collect::<Vec<_>>()
                        .await
                },
                |results| Message::MediaFetchedBatch(results).into(),
            ));
        }
        #[cfg(feature = "video-player")]
        for (source, mxc_url, filename) in autoplay_requests {
            tasks.push(Task::done(
                Message::PlayVideo {
                    source,
                    mxc_url,
                    filename,
                    autoplay: true,
                }
                .into(),
            ));
        }

        if let Some(og_task) = self.fetch_missing_og_previews() {
            tasks.push(og_task);
        }
        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }
    pub fn fetch_missing_og_previews(&mut self) -> Option<Task<Action<Message>>> {
        let mut urls_to_fetch = Vec::new();

        let mut check_links = |links: &[(String, String)]| {
            for (_label, url) in links {
                if (url.starts_with("http://") || url.starts_with("https://"))
                    && !url.contains("matrix.to/#/")
                    && !self.og_cache.contains_key(url)
                {
                    self.og_cache
                        .insert(url.clone(), crate::utils::og::OgState::Pending);
                    urls_to_fetch.push(url.clone());
                }
            }
        };

        for item in &self.timeline_items {
            check_links(&item.markdown_links);
            check_links(&item.plain_links);
        }
        for item in &self.threaded_timeline_items {
            check_links(&item.markdown_links);
            check_links(&item.plain_links);
        }

        if self.composer_is_preview {
            check_links(&self.composer_preview_links);
        }

        if urls_to_fetch.is_empty() {
            None
        } else {
            let tasks: Vec<Task<Action<Message>>> = urls_to_fetch
                .into_iter()
                .map(|url| {
                    Task::perform(
                        crate::utils::og::fetch_og_preview(url.clone()),
                        move |res| Action::from(Message::OgPreviewFetched(url, res)),
                    )
                })
                .collect();
            Some(Task::batch(tasks))
        }
    }

    pub(super) fn check_and_perform_initial_scroll(
        &mut self,
    ) -> Option<Task<Action<<Constellation as Application>::Message>>> {
        if self.needs_initial_scroll && !self.is_loading_more && self.is_timeline_initialized {
            self.needs_initial_scroll = false;
            if self.timeline_items.is_empty() {
                return None;
            }
            // Returning to a room we memorized a scroll anchor for overrides
            // default placement; the anchor is resolved once the freshly
            // built timeline has been measured.
            if self.active_event_focus.is_none() && self.pending_room_restore.is_some() {
                self.scroll_main.measure_pending = true;
                return Some(scroll::measure_timeline_task(false, self.scroll_generation));
            }
            return self.default_initial_scroll_task();
        }
        None
    }

    /// Default open placement: bottom for fresh joins / fully-read rooms,
    /// otherwise aligned near the first unread item.
    fn default_initial_scroll_task(
        &self,
    ) -> Option<Task<Action<<Constellation as Application>::Message>>> {
        let room_id = self.selected_room.as_ref()?;
        let unread_count = self.room_by_id(room_id).map_or(0, |room| room.unread_count);

        let offset = if self.is_first_time_joining || unread_count == 0 {
            scrollable::RelativeOffset::END
        } else {
            let total_items = self.timeline_items.len();
            let unread = unread_count as usize;
            if total_items == 0 {
                scrollable::RelativeOffset::END
            } else if unread >= total_items {
                scrollable::RelativeOffset::START
            } else {
                let ratio = (total_items - unread) as f32 / total_items as f32;
                scrollable::RelativeOffset { x: 0.0, y: ratio }
            }
        };

        Some(scrollable::snap_to(TIMELINE_ID.clone(), offset.into()))
    }

    pub fn handle_timeline_diff(
        &mut self,
        diff: eyeball_im::VectorDiff<Arc<TimelineItem>>,
        is_thread: bool,
        root_id: Option<OwnedEventId>,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        let mut tasks = Vec::new();
        let mut media_fetches: Vec<PinnedOutput> = Vec::new();
        #[cfg(feature = "video-player")]
        let mut autoplay_requests: Vec<(MediaSource, String, String)> = Vec::new();
        let mut check_item = |item: &Arc<TimelineItem>, fetches: &mut Vec<_>| {
            if let Some(event) = item.as_event() {
                if let TimelineDetails::Ready(profile) = event.sender_profile()
                    && let Some(avatar_url) = &profile.avatar_url
                {
                    let url_str = avatar_url.to_string();
                    if !self.media_cache.contains_key(&url_str)
                        && let Some(matrix) = &self.matrix
                    {
                        let matrix_clone = matrix.clone();
                        let source = MediaSource::Plain(avatar_url.clone());
                        fetches.push(
                            async move {
                                let res = matrix_clone
                                    .fetch_media(source)
                                    .await
                                    .map_err(|e| e.to_string());
                                (url_str, res)
                            }
                            .boxed(),
                        );
                    }
                }

                if !self.user_settings.media_previews_display_policy {
                    return;
                }
                let Some(message) = event.content().as_message() else {
                    return;
                };

                match message.msgtype() {
                    MessageType::Image(image) => {
                        let mxc_url = match &image.source {
                            MediaSource::Plain(uri) => uri.to_string(),
                            MediaSource::Encrypted(file) => file.url.to_string(),
                        };
                        if !self.media_cache.contains_key(&mxc_url)
                            && let Some(matrix) = &self.matrix
                        {
                            let matrix_clone = matrix.clone();
                            let source = image.source.clone();
                            fetches.push(
                                async move {
                                    let res = matrix_clone
                                        .fetch_media(source)
                                        .await
                                        .map_err(|e| e.to_string());
                                    (mxc_url, res)
                                }
                                .boxed(),
                            );
                        }
                    }
                    MessageType::Video(video) => {
                        if let Some(info) = &video.info
                            && let Some(thumb_source) = &info.thumbnail_source
                        {
                            let thumb_url = match thumb_source {
                                MediaSource::Plain(uri) => uri.to_string(),
                                MediaSource::Encrypted(file) => file.url.to_string(),
                            };
                            if !self.media_cache.contains_key(&thumb_url)
                                && let Some(matrix) = &self.matrix
                            {
                                let matrix_clone = matrix.clone();
                                let source = thumb_source.clone();
                                fetches.push(
                                    async move {
                                        let res = matrix_clone
                                            .fetch_media(source)
                                            .await
                                            .map_err(|e| e.to_string());
                                        (thumb_url, res)
                                    }
                                    .boxed(),
                                );
                            }
                        }
                        #[cfg(feature = "video-player")]
                        if self.app_settings.autoplay_videos {
                            let mxc_url = match &video.source {
                                MediaSource::Plain(uri) => uri.to_string(),
                                MediaSource::Encrypted(file) => file.url.to_string(),
                            };
                            if !self.video_cache.contains_key(&mxc_url)
                                && !self.loading_videos.contains(&mxc_url)
                            {
                                autoplay_requests.push((
                                    video.source.clone(),
                                    mxc_url,
                                    video.body.clone(),
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        };

        match &diff {
            eyeball_im::VectorDiff::Insert { value, .. } => check_item(value, &mut media_fetches),
            eyeball_im::VectorDiff::Set { value, .. } => check_item(value, &mut media_fetches),
            eyeball_im::VectorDiff::PushBack { value } => check_item(value, &mut media_fetches),
            eyeball_im::VectorDiff::PushFront { value } => check_item(value, &mut media_fetches),
            eyeball_im::VectorDiff::Append { values } => values
                .iter()
                .for_each(|v| check_item(v, &mut media_fetches)),
            eyeball_im::VectorDiff::Reset { values } => values
                .iter()
                .for_each(|v| check_item(v, &mut media_fetches)),
            _ => {}
        }

        if !media_fetches.is_empty() {
            tasks.push(cosmic::iced::Task::perform(
                async move {
                    futures::stream::iter(media_fetches)
                        .buffer_unordered(10)
                        .collect::<Vec<_>>()
                        .await
                },
                |results| Message::MediaFetchedBatch(results).into(),
            ));
        }
        #[cfg(feature = "video-player")]
        for (source, mxc_url, filename) in autoplay_requests {
            tasks.push(cosmic::iced::Task::done(
                Message::PlayVideo {
                    source,
                    mxc_url,
                    filename,
                    autoplay: true,
                }
                .into(),
            ));
        }
        if let Some(og_task) = self.fetch_missing_og_previews() {
            tasks.push(og_task);
        }

        let mapped_diff = match diff {
            eyeball_im::VectorDiff::Insert { index, value } => eyeball_im::VectorDiff::Insert {
                index,
                value: ConstellationItem::new(value, self.user_id.as_deref()),
            },
            eyeball_im::VectorDiff::Set { index, value } => eyeball_im::VectorDiff::Set {
                index,
                value: ConstellationItem::new(value, self.user_id.as_deref()),
            },
            eyeball_im::VectorDiff::PushBack { value } => eyeball_im::VectorDiff::PushBack {
                value: ConstellationItem::new(value, self.user_id.as_deref()),
            },
            eyeball_im::VectorDiff::PushFront { value } => eyeball_im::VectorDiff::PushFront {
                value: ConstellationItem::new(value, self.user_id.as_deref()),
            },
            eyeball_im::VectorDiff::Append { values } => eyeball_im::VectorDiff::Append {
                values: values
                    .into_iter()
                    .map(|v| ConstellationItem::new(v, self.user_id.as_deref()))
                    .collect(),
            },
            eyeball_im::VectorDiff::Reset { values } => eyeball_im::VectorDiff::Reset {
                values: values
                    .into_iter()
                    .map(|v| ConstellationItem::new(v, self.user_id.as_deref()))
                    .collect(),
            },
            eyeball_im::VectorDiff::Remove { index } => eyeball_im::VectorDiff::Remove { index },
            eyeball_im::VectorDiff::PopBack => eyeball_im::VectorDiff::PopBack,
            eyeball_im::VectorDiff::PopFront => eyeball_im::VectorDiff::PopFront,
            eyeball_im::VectorDiff::Clear => eyeball_im::VectorDiff::Clear,
            eyeball_im::VectorDiff::Truncate { length } => {
                eyeball_im::VectorDiff::Truncate { length }
            }
        };

        if is_thread {
            if let Some(root_id) = root_id
                && self.active_thread_root == Some(root_id)
            {
                let is_append = match &mapped_diff {
                    eyeball_im::VectorDiff::PushBack { .. } => true,
                    eyeball_im::VectorDiff::Append { .. } => true,
                    eyeball_im::VectorDiff::Insert { index, .. } => {
                        *index >= self.threaded_timeline_items.len()
                    }
                    _ => false,
                };

                let is_prepend = match &mapped_diff {
                    eyeball_im::VectorDiff::PushFront { .. } => true,
                    eyeball_im::VectorDiff::Insert { index, .. } => {
                        *index < self.threaded_timeline_items.len()
                    }
                    eyeball_im::VectorDiff::Reset { .. } => self.is_loading_more,
                    _ => false,
                };

                let is_reset = matches!(
                    &mapped_diff,
                    eyeball_im::VectorDiff::Reset { .. } | eyeball_im::VectorDiff::Clear
                );

                if is_prepend {
                    self.needs_threaded_scroll_adjustment = true;
                }

                self.threaded_timeline_items.apply_diff(mapped_diff);
                if is_reset {
                    // Measured row geometry is meaningless after a reset.
                    scroll::tracker_mut(self, true).reset();
                }

                if is_append && self.is_threaded_timeline_at_bottom {
                    tasks.push(scrollable::snap_to(
                        THREADED_TIMELINE_ID.clone(),
                        scrollable::RelativeOffset::END.into(),
                    ));
                } else if is_reset {
                    if self.is_threaded_timeline_at_bottom {
                        tasks.push(scrollable::snap_to(
                            THREADED_TIMELINE_ID.clone(),
                            scrollable::RelativeOffset::END.into(),
                        ));
                    } else {
                        tasks.push(scrollable::scroll_to(
                            THREADED_TIMELINE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: Some(0.0),
                                y: Some(self.last_threaded_timeline_offset),
                            },
                        ));
                    }
                }
            }
        } else {
            let is_append = match &mapped_diff {
                eyeball_im::VectorDiff::PushBack { .. } => true,
                eyeball_im::VectorDiff::Append { .. } => true,
                eyeball_im::VectorDiff::Insert { index, .. } => *index >= self.timeline_items.len(),
                _ => false,
            };

            let is_prepend = match &mapped_diff {
                eyeball_im::VectorDiff::PushFront { .. } => true,
                eyeball_im::VectorDiff::Insert { index, .. } => *index < self.timeline_items.len(),
                eyeball_im::VectorDiff::Reset { .. } => self.is_loading_more,
                _ => false,
            };

            let is_reset = matches!(
                &mapped_diff,
                eyeball_im::VectorDiff::Reset { .. } | eyeball_im::VectorDiff::Clear
            );

            if is_prepend {
                self.needs_scroll_adjustment = true;
            }

            self.timeline_items.apply_diff(mapped_diff);
            if is_reset {
                // Measured row geometry is meaningless after a reset.
                scroll::tracker_mut(self, false).reset();
            }
            self.recompute_timeline_metadata();

            if let Some(task) = self.check_and_perform_initial_scroll() {
                tasks.push(task);
            } else if is_append && self.is_timeline_at_bottom {
                tasks.push(scrollable::snap_to(
                    TIMELINE_ID.clone(),
                    scrollable::RelativeOffset::END.into(),
                ));
            } else if is_reset {
                if self.is_timeline_at_bottom {
                    tasks.push(scrollable::snap_to(
                        TIMELINE_ID.clone(),
                        scrollable::RelativeOffset::END.into(),
                    ));
                } else {
                    tasks.push(scrollable::scroll_to(
                        TIMELINE_ID.clone(),
                        scrollable::AbsoluteOffset {
                            x: Some(0.0),
                            y: Some(self.last_timeline_offset),
                        },
                    ));
                }
            }
        }

        if !tasks.is_empty() {
            cosmic::iced::Task::batch(tasks)
        } else {
            Task::none()
        }
    }

    pub fn handle_matrix_event(
        &mut self,
        event: matrix::MatrixEvent,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        match event {
            matrix::MatrixEvent::SyncStatusChanged(status) => {
                self.sync_status = status;
                Task::none()
            }
            matrix::MatrixEvent::SyncIndicatorChanged(show) => {
                self.is_sync_indicator_active = show;
                Task::none()
            }
            matrix::MatrixEvent::RoomDiff(diff) => {
                match &*diff {
                    eyeball_im::VectorDiff::Insert { value, .. }
                    | eyeball_im::VectorDiff::PushBack { value }
                    | eyeball_im::VectorDiff::PushFront { value } => {
                        self.joined_room_ids.insert(value.id.clone());
                        if let Some(name) = &value.name {
                            self.room_name_cache.insert(value.id.clone(), name.clone());
                        }
                    }
                    eyeball_im::VectorDiff::Remove { index } => {
                        if let Some(room) = self.room_list.get(*index) {
                            self.joined_room_ids.remove(&room.id);
                            self.room_name_cache.remove(&room.id);
                        }
                    }
                    eyeball_im::VectorDiff::Set { index, value } => {
                        if let Some(old_room) = self.room_list.get(*index) {
                            self.joined_room_ids.remove(&old_room.id);
                            self.room_name_cache.remove(&old_room.id);
                        }
                        self.joined_room_ids.insert(value.id.clone());
                        if let Some(name) = &value.name {
                            self.room_name_cache.insert(value.id.clone(), name.clone());
                        }
                    }
                    eyeball_im::VectorDiff::PopBack => {
                        if let Some(room) = self.room_list.last() {
                            self.joined_room_ids.remove(&room.id);
                            self.room_name_cache.remove(&room.id);
                        }
                    }
                    eyeball_im::VectorDiff::PopFront => {
                        if let Some(room) = self.room_list.first() {
                            self.joined_room_ids.remove(&room.id);
                            self.room_name_cache.remove(&room.id);
                        }
                    }
                    eyeball_im::VectorDiff::Clear => {
                        self.joined_room_ids.clear();
                        self.room_name_cache.clear();
                    }
                    eyeball_im::VectorDiff::Reset { values }
                    | eyeball_im::VectorDiff::Append { values } => {
                        for r in values {
                            if !self.joined_room_ids.contains(&r.id) {
                                self.joined_room_ids.insert(r.id.clone());
                            }
                            if let Some(name) = &r.name {
                                self.room_name_cache.insert(r.id.clone(), name.clone());
                            }
                        }
                    }
                    eyeball_im::VectorDiff::Truncate { length } => {
                        for room in self.room_list.iter().skip(*length) {
                            self.joined_room_ids.remove(&room.id);
                            self.room_name_cache.remove(&room.id);
                        }
                    }
                }

                self.room_list.apply_diff(*diff);
                self.rebuild_room_index();
                self.update_filtered_rooms();
                self.rebuild_space_nav_model();
                self.update_title()
            }
            matrix::MatrixEvent::TimelineDiff(diff) => self.handle_timeline_diff(diff, false, None),
            matrix::MatrixEvent::TimelineReset => {
                let is_background_reset = self.is_timeline_initialized;
                self.timeline_items.clear();
                self.recompute_timeline_metadata();
                self.needs_initial_scroll = !is_background_reset;
                self.needs_scroll_restoration = is_background_reset;
                self.last_content_height = 0.0;
                self.last_viewport_width = 0.0;
                self.last_viewport_height = 0.0;
                self.needs_scroll_adjustment = false;
                // Measured geometry is gone; invalidate in-flight row
                // measurements for this room too.
                self.scroll_main.reset();
                self.scroll_generation += 1;
                if !is_background_reset {
                    self.is_timeline_at_bottom = true;
                    self.is_threaded_timeline_at_bottom = true;
                }
                self.is_timeline_initialized = false;
                Task::none()
            }
            matrix::MatrixEvent::TimelineInitFinished => {
                self.is_timeline_initialized = true;
                // A permalink asked us to focus on a specific event. If it is
                // already in the loaded window we can just scroll to it;
                // otherwise we build an event-focused timeline around it.
                let event_focus_task = self.check_pending_event_focus();
                if self.needs_scroll_restoration {
                    self.needs_scroll_restoration = false;
                    if self.is_timeline_at_bottom {
                        scrollable::snap_to(
                            TIMELINE_ID.clone(),
                            scrollable::RelativeOffset::END.into(),
                        )
                    } else {
                        scrollable::scroll_to(
                            TIMELINE_ID.clone(),
                            scrollable::AbsoluteOffset {
                                x: Some(0.0),
                                y: Some(self.last_timeline_offset),
                            },
                        )
                    }
                } else if let Some(task) = self.check_and_perform_initial_scroll() {
                    task
                } else {
                    event_focus_task
                }
            }
            matrix::MatrixEvent::ReactionAdded { .. } => {
                // For now, we don't do anything specific as reactions are handled via TimelineDiff
                Task::none()
            }
            matrix::MatrixEvent::IgnoredUsersChanged(users) => {
                self.user_settings.ignored_users = users;
                Task::none()
            }
            matrix::MatrixEvent::SpaceHierarchyChanged => {
                let mut tasks = Vec::new();
                if let Some(matrix) = &self.matrix
                    && let Some(sid) = &self.selected_space
                {
                    let matrix_clone = matrix.clone();
                    let sid_clone = sid.clone();
                    tasks.push(Task::perform(
                        async move {
                            let _ = matrix_clone.update_room_list_filter(Some(sid_clone)).await;
                        },
                        |_| Action::from(Message::SpaceFilterUpdated),
                    ));
                }
                self.update_filtered_rooms();
                if tasks.is_empty() {
                    Task::none()
                } else {
                    Task::batch(tasks)
                }
            }
            matrix::MatrixEvent::CallParticipantsChanged {
                room_id,
                participants,
            } => {
                self.call_participants.insert(room_id.into(), participants);
                Task::none()
            }
        }
    }

    pub fn handle_load_more(
        &mut self,
        is_thread: bool,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        if self.is_loading_more {
            return Task::none();
        }

        if let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) {
            self.is_loading_more = true;
            let matrix = matrix.clone();
            let room_id = room_id.clone();
            let root_id = if is_thread {
                self.active_thread_root.clone()
            } else {
                None
            };

            Task::perform(
                async move {
                    if let Some(root_id) = root_id {
                        let timeline = matrix.threaded_timeline(&room_id, &root_id).await?;
                        timeline.paginate_backwards(20).await?;
                    } else {
                        matrix.paginate_backwards(&room_id, 20).await?;
                    }
                    Ok(())
                },
                |res: Result<(), anyhow::Error>| {
                    Action::from(Message::LoadMoreFinished(res.map_err(|e| e.to_string())))
                },
            )
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_timeline_scrolled(
        &mut self,
        viewport: cosmic::iced::widget::scrollable::Viewport,
        is_thread: bool,
    ) -> Task<Action<Message>> {
        let current_offset = viewport.absolute_offset().y;
        let current_height = viewport.content_bounds().height;

        let is_initialized = if is_thread {
            self.is_threaded_timeline_initialized
        } else {
            self.is_timeline_initialized
        };
        if !is_initialized {
            return Task::none();
        }

        // Track observed geometry so in-flight measurements can be judged
        // fresh or stale.
        scroll::tracker_mut(self, is_thread).note_observed(viewport.bounds().width, current_height);

        let prefix = if is_thread {
            "TimelineScrolled (thread)"
        } else {
            "TimelineScrolled"
        };
        let last_content_height = if is_thread {
            self.last_threaded_content_height
        } else {
            self.last_content_height
        };
        let last_viewport_width = if is_thread {
            self.last_threaded_viewport_width
        } else {
            self.last_viewport_width
        };
        let last_viewport_height = if is_thread {
            self.last_threaded_viewport_height
        } else {
            self.last_viewport_height
        };
        let needs_layout_scroll_restoration = if is_thread {
            self.needs_threaded_layout_scroll_restoration
        } else {
            self.needs_layout_scroll_restoration
        };
        let needs_scroll_adjustment = if is_thread {
            self.needs_threaded_scroll_adjustment
        } else {
            self.needs_scroll_adjustment
        };

        tracing::info!(
            "{}: offset={}, content_height={}, viewport_w={}, viewport_h={}, last_h={}, last_w={}, last_vh={}",
            prefix,
            current_offset,
            current_height,
            viewport.bounds().width,
            viewport.bounds().height,
            last_content_height,
            last_viewport_width,
            last_viewport_height
        );

        let mut is_layout_resize = false;
        if (needs_layout_scroll_restoration
            || (last_content_height > 0.0 && current_height != last_content_height)
            || (last_viewport_width > 0.0 && viewport.bounds().width != last_viewport_width)
            || (last_viewport_height > 0.0 && viewport.bounds().height != last_viewport_height))
            && !needs_scroll_adjustment
        {
            is_layout_resize = true;
        }

        if is_thread {
            self.needs_threaded_layout_scroll_restoration = false;
        } else {
            self.needs_layout_scroll_restoration = false;
        }
        let mut measure_tasks: Vec<Task<Action<Message>>> = Vec::new();

        let mut task = Task::none();
        let mut actual_offset = current_offset;
        let timeline_id = if is_thread {
            THREADED_TIMELINE_ID.clone()
        } else {
            TIMELINE_ID.clone()
        };

        let needs_adjustment = if is_thread {
            self.needs_threaded_scroll_adjustment
        } else {
            self.needs_scroll_adjustment
        };

        if needs_adjustment && last_content_height > 0.0 && current_height > last_content_height {
            if is_thread {
                self.needs_threaded_scroll_adjustment = false;
            } else {
                self.needs_scroll_adjustment = false;
            }
            let diff_height = current_height - last_content_height;
            // Rows prepended above shifted every cached anchor down by the
            // same delta; existing row heights are unchanged.
            scroll::tracker_mut(self, is_thread).shift(diff_height);
            actual_offset = current_offset + diff_height;
            task = scrollable::scroll_to(
                timeline_id,
                scrollable::AbsoluteOffset {
                    x: Some(0.0),
                    y: Some(actual_offset),
                },
            );
        } else if is_layout_resize {
            let is_at_bottom = if is_thread {
                self.is_threaded_timeline_at_bottom
            } else {
                self.is_timeline_at_bottom
            };
            if is_at_bottom {
                // Remounts destroy the scrollable's state, and any scroll
                // task issued now would run against the pre-rebuild tree and
                // be lost. Defer a plain END re-snap until the rebuilt
                // layout settles; no row geometry is needed for this.
                let tracker = scroll::tracker_mut(self, is_thread);
                if !tracker.end_snap_scheduled {
                    tracker.end_snap_scheduled = true;
                    tracker.end_snap_deadline =
                        Some(std::time::Instant::now() + std::time::Duration::from_millis(120));
                }
            } else if self.is_search_active && !is_thread {
                // The search-results view owns TIMELINE_ID while active;
                // leave its scroll state alone.
            } else {
                let last_offset = if is_thread {
                    self.last_threaded_timeline_offset
                } else {
                    self.last_timeline_offset
                };
                // Pixels are meaningless after a width change; decode an
                // anchor from the last measured geometry and resolve it once
                // the reflowed layout has been re-measured.
                let mut expect_initial = false;
                {
                    let tracker = scroll::tracker_mut(self, is_thread);
                    let plan = scroll::plan_reflow(
                        last_offset,
                        &tracker.children,
                        tracker.children_content_height,
                        current_height,
                    )
                    .or_else(|| {
                        // No usable snapshot (e.g. the pane was just
                        // re-selected and its bookkeeping reset). Fall back
                        // to a proportional restore against the last known
                        // total height.
                        if last_content_height > 0.0 && current_height > 0.0 && last_offset > 0.0 {
                            Some(scroll::PendingReflow::Ratio(
                                last_offset / last_content_height,
                            ))
                        } else {
                            None
                        }
                    });
                    if let Some(plan) = plan {
                        tracing::debug!(
                            "{} resize armed reflow restore",
                            if is_thread { "thread" } else { "main" }
                        );
                        tracker.reflow_attempts = 0;
                        tracker.pending_reflow = Some(plan);
                        if !tracker.delayed_scheduled {
                            tracker.delayed_scheduled = true;
                            tracker.measure_deadline = Some(
                                std::time::Instant::now() + std::time::Duration::from_millis(90),
                            );
                        }
                    } else if !is_thread && last_offset <= 0.0 && !self.timeline_items.is_empty() {
                        // Bookkeeping was freshly reset (room just opened) so
                        // there is nothing to preserve; after the remount,
                        // apply the default initial placement instead of
                        // staying parked at the top.
                        expect_initial = true;
                    }
                }
                if expect_initial {
                    self.needs_initial_scroll = true;
                    self.scroll_main.expect_relayout = true;
                    if !self.scroll_main.delayed_scheduled {
                        self.scroll_main.delayed_scheduled = true;
                        self.scroll_main.measure_deadline =
                            Some(std::time::Instant::now() + std::time::Duration::from_millis(90));
                    }
                }
            }
        }

        if is_layout_resize {
            tracing::info!("{} layout resize: target_offset={}", prefix, actual_offset);
        }

        let last_offset = if is_thread {
            self.last_threaded_timeline_offset
        } else {
            self.last_timeline_offset
        };
        let should_load = !is_layout_resize && actual_offset < 100.0 && actual_offset < last_offset;
        let is_at_bottom = actual_offset + viewport.bounds().height >= current_height - 20.0;

        if !is_layout_resize {
            if is_thread {
                self.last_threaded_timeline_offset = actual_offset;
                self.last_threaded_content_height = current_height;
                self.last_threaded_viewport_width = viewport.bounds().width;
                self.last_threaded_viewport_height = viewport.bounds().height;
                self.is_threaded_timeline_at_bottom = is_at_bottom;
            } else {
                self.last_timeline_offset = actual_offset;
                self.last_content_height = current_height;
                self.last_viewport_width = viewport.bounds().width;
                self.last_viewport_height = viewport.bounds().height;
                self.is_timeline_at_bottom = is_at_bottom;
            }
            // Keep the measured row snapshot warm so anchors can be decoded
            // whenever a reflow hits.
            let mut request_measure = false;
            if !(self.is_search_active && !is_thread) {
                let tracker = scroll::tracker_mut(self, is_thread);
                if tracker.is_stale() && !tracker.measure_pending {
                    tracker.measure_pending = true;
                    request_measure = true;
                }
            }
            if request_measure {
                measure_tasks.push(scroll::measure_timeline_task(
                    is_thread,
                    self.scroll_generation,
                ));
            }
        } else {
            if is_thread {
                self.last_threaded_content_height = current_height;
                self.last_threaded_viewport_width = viewport.bounds().width;
                self.last_threaded_viewport_height = viewport.bounds().height;
            } else {
                self.last_content_height = current_height;
                self.last_viewport_width = viewport.bounds().width;
                self.last_viewport_height = viewport.bounds().height;
            }
        }

        if should_load {
            measure_tasks.push(self.handle_load_more(is_thread));
        }

        measure_tasks.push(task);
        if measure_tasks.len() == 1 {
            Task::none()
        } else {
            Task::batch(measure_tasks)
        }
    }

    /// Apply a measured row snapshot from the live widget tree: resolve a
    /// pending reflow restore or a room-switch restore against it, or just
    /// refresh the cache used for future anchor decoding.
    pub(super) fn handle_timeline_measured(
        &mut self,
        is_thread: bool,
        generation: u64,
        viewport_width: f32,
        content_height: f32,
        rows: Vec<(String, f32)>,
    ) -> Task<Action<Message>> {
        // The search-results view reuses TIMELINE_ID; never feed its geometry
        // into timeline anchor state. A measurement requested for a room we
        // have already left is equally worthless.
        if (!is_thread && self.is_search_active) || generation != self.scroll_generation {
            tracing::debug!(
                "{} measurement dropped: search={} gen {} != {}",
                if is_thread { "thread" } else { "main" },
                self.is_search_active,
                generation,
                self.scroll_generation
            );
            return Task::none();
        }

        let viewport_height = if is_thread {
            self.last_threaded_viewport_height
        } else {
            self.last_viewport_height
        };

        // The measurement completed: release the in-flight slot. Without
        // this, the first measurement wedges the flag `true` forever and all
        // later restores are silently skipped.
        scroll::tracker_mut(self, is_thread).measure_pending = false;

        let mut tasks: Vec<Task<Action<Message>>> = Vec::new();

        let (fresh, expect_relayout) = {
            let tracker = scroll::tracker_mut(self, is_thread);
            (
                scroll::measurement_is_fresh(tracker, viewport_width, content_height)
                    || tracker.expect_relayout,
                tracker.expect_relayout,
            )
        };
        tracing::debug!(
            "{} measured: w={} ch={} rows={} fresh={} expect_relayout={} pending={}",
            if is_thread { "thread" } else { "main" },
            viewport_width,
            content_height,
            rows.len(),
            fresh,
            expect_relayout,
            scroll::tracker_mut(self, is_thread)
                .pending_reflow
                .is_some(),
        );

        // A reflow is waiting to be resolved against fresh geometry.
        if let Some(pending) = scroll::tracker_mut(self, is_thread).pending_reflow.take() {
            let attempts = scroll::tracker_mut(self, is_thread).reflow_attempts;
            if !fresh && attempts < scroll::MAX_REFLOW_ATTEMPTS {
                // Measured mid-resize; ask again until the layout settles.
                let tracker = scroll::tracker_mut(self, is_thread);
                tracker.reflow_attempts += 1;
                tracker.pending_reflow = Some(pending);
                tracker.measure_pending = true;
                tracing::debug!(
                    "{} measurement stale, retry {}/{}",
                    if is_thread { "thread" } else { "main" },
                    tracker.reflow_attempts,
                    scroll::MAX_REFLOW_ATTEMPTS
                );
                return scroll::measure_timeline_task(is_thread, generation);
            }

            let resolved = {
                let tracker = scroll::tracker_mut(self, is_thread);
                tracker.store(rows, content_height, viewport_width);
                tracker.expect_relayout = false;
                let r = scroll::resolve_reflow(&pending, tracker);
                tracing::debug!(
                    "{} reflow resolve: {:?}",
                    if is_thread { "thread" } else { "main" },
                    r
                );
                r
            };
            if let Some(target) = resolved {
                let max_offset = (content_height - viewport_height).max(0.0);
                let target = target.clamp(0.0, max_offset);
                let at_bottom = target + viewport_height >= content_height - 20.0;
                let id = if is_thread {
                    self.last_threaded_timeline_offset = target;
                    self.is_threaded_timeline_at_bottom = at_bottom;
                    THREADED_TIMELINE_ID.clone()
                } else {
                    self.last_timeline_offset = target;
                    self.is_timeline_at_bottom = at_bottom;
                    TIMELINE_ID.clone()
                };
                tasks.push(scrollable::scroll_to(
                    id,
                    scrollable::AbsoluteOffset {
                        x: Some(0.0),
                        y: Some(target),
                    },
                ));
            }
            return Self::batch_all(tasks);
        }

        // A room switch asked us to resume a memorized position once the
        // rebuilt timeline has been laid out and measured.
        if !is_thread && let Some((key, intra_y)) = self.pending_room_restore.take() {
            let found = rows
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, y)| y + intra_y);
            let tracker = scroll::tracker_mut(self, false);
            tracker.store(rows, content_height, viewport_width);
            tracker.expect_relayout = false;
            if let Some(target) = found {
                let max_offset = (content_height - viewport_height).max(0.0);
                let target = target.clamp(0.0, max_offset);
                self.last_timeline_offset = target;
                self.is_timeline_at_bottom = target + viewport_height >= content_height - 20.0;
                tasks.push(scrollable::scroll_to(
                    TIMELINE_ID.clone(),
                    scrollable::AbsoluteOffset {
                        x: Some(0.0),
                        y: Some(target),
                    },
                ));
            } else if let Some(task) = self.default_initial_scroll_task() {
                // The memorized row is no longer loaded (history reset while
                // away); fall back to default placement.
                tasks.push(task);
            }
            return Self::batch_all(tasks);
        }

        // A remount with freshly reset bookkeeping asked for default
        // placement once the rebuilt layout has been measured.
        let expect_initial =
            !is_thread && self.needs_initial_scroll && self.scroll_main.expect_relayout;
        let was_at_bottom = if is_thread {
            self.is_threaded_timeline_at_bottom
        } else {
            self.is_timeline_at_bottom
        };
        let expect_end_snap = self.scroll_main.expect_relayout && was_at_bottom && !is_thread;
        let tracker = scroll::tracker_mut(self, is_thread);
        tracker.store(rows, content_height, viewport_width);
        tracker.expect_relayout = false;
        if expect_initial {
            self.needs_initial_scroll = false;
            if let Some(task) = self.default_initial_scroll_task() {
                tasks.push(task);
            }
        } else if expect_end_snap {
            // The pane was at the bottom when a remounting toggle fired; the
            // rebuilt scrollable starts at 0, so snap back to the end.
            tasks.push(scrollable::snap_to(
                if is_thread {
                    THREADED_TIMELINE_ID.clone()
                } else {
                    TIMELINE_ID.clone()
                },
                scrollable::RelativeOffset::END.into(),
            ));
        }
        Self::batch_all(tasks)
    }

    /// Run the row-measurement operation now (deduplicated). Emitted by the
    /// deferred task after a layout-affecting toggle has rebuilt the pane.
    /// Fire any deferred scroll restore whose deadline has passed. Driven by
    /// the continuous `RestoreTick` subscription so restores happen without
    /// any user input.
    pub(super) fn handle_restore_tick(&mut self) -> Task<Action<Message>> {
        let now = std::time::Instant::now();
        let mut tasks: Vec<Task<Action<Message>>> = Vec::new();

        if self.scroll_main.end_snap_scheduled
            && self.scroll_main.end_snap_deadline.is_some_and(|d| now >= d)
        {
            self.scroll_main.end_snap_scheduled = false;
            tasks.push(scrollable::snap_to(
                TIMELINE_ID.clone(),
                scrollable::RelativeOffset::END.into(),
            ));
        }
        if self.scroll_thread.end_snap_scheduled
            && self
                .scroll_thread
                .end_snap_deadline
                .is_some_and(|d| now >= d)
        {
            self.scroll_thread.end_snap_scheduled = false;
            tasks.push(scrollable::snap_to(
                THREADED_TIMELINE_ID.clone(),
                scrollable::RelativeOffset::END.into(),
            ));
        }
        if self.scroll_main.delayed_scheduled
            && self.scroll_main.measure_deadline.is_some_and(|d| now >= d)
        {
            self.scroll_main.delayed_scheduled = false;
            self.scroll_main.measure_pending = true;
            tracing::debug!("restore tick: firing main measure");
            tasks.push(scroll::measure_timeline_task(false, self.scroll_generation));
        }
        if self.scroll_thread.delayed_scheduled
            && self
                .scroll_thread
                .measure_deadline
                .is_some_and(|d| now >= d)
        {
            self.scroll_thread.delayed_scheduled = false;
            self.scroll_thread.measure_pending = true;
            tracing::debug!("restore tick: firing thread measure");
            tasks.push(scroll::measure_timeline_task(true, self.scroll_generation));
        }
        Self::batch_all(tasks)
    }

    fn batch_all(tasks: Vec<Task<Action<Message>>>) -> Task<Action<Message>> {
        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    pub(super) fn handle_jump_to_message(
        &mut self,
        event_id: matrix_sdk::ruma::OwnedEventId,
    ) -> Task<Action<Message>> {
        let index = self.timeline_items.iter().position(|item| {
            item.item_id.as_ref().is_some_and(|id| {
                if let crate::matrix::TimelineEventItemId::EventId(eid) = id {
                    eid == &event_id
                } else {
                    false
                }
            })
        });

        if let Some(i) = index
            && !self.timeline_items.is_empty()
            && self.last_content_height > 0.0
        {
            let relative_idx = i as f32 / self.timeline_items.len() as f32;
            let target_y =
                (relative_idx * self.last_content_height) - (self.last_viewport_height / 2.0);
            let target_y =
                target_y.clamp(0.0, self.last_content_height - self.last_viewport_height);

            self.last_timeline_offset = target_y;

            cosmic::iced::widget::scrollable::scroll_to(
                crate::TIMELINE_ID.clone(),
                cosmic::iced::widget::scrollable::AbsoluteOffset {
                    x: Some(0.0),
                    y: Some(target_y),
                },
            )
        } else {
            Task::none()
        }
    }
}
