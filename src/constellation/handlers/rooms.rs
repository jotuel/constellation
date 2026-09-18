use crate::constellation::{Tab, scroll};
use crate::matrix;
use crate::settings;
use crate::{
    Constellation, MediaSource, Message, OwnedRoomId, SettingsPanel, THREADED_TIMELINE_ID,
};
use cosmic::iced::widget::scrollable;
use cosmic::{Action, Application, Task};
use futures::stream::StreamExt;
use matrix_sdk::ruma::OwnedEventId;
use std::sync::Arc;

impl Constellation {
    pub fn handle_join_call(&mut self) -> Task<Action<Message>> {
        if let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) {
            let matrix = matrix.clone();
            let room_id = room_id.clone();
            Task::perform(
                async move { matrix.join_call(&room_id).await.map_err(|e| e.to_string()) },
                |res| Action::from(Message::CallJoined(res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_leave_call(&mut self) -> Task<Action<Message>> {
        if let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) {
            let matrix = matrix.clone();
            let room_id = room_id.to_string();
            Task::perform(
                async move { matrix.leave_call(&room_id).await.map_err(|e| e.to_string()) },
                |res| Action::from(Message::CallLeft(res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_create_room(
        &mut self,
        name: String,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix = matrix.clone();
            let is_video = self.new_room_is_video;
            Task::perform(
                async move {
                    matrix
                        .create_room(&name, is_video)
                        .await
                        .map(|id| id.to_string())
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::RoomCreated(res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_create_space(
        &mut self,
        name: String,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix = matrix.clone();
            Task::perform(
                async move {
                    matrix
                        .create_space(&name)
                        .await
                        .map(|id| id.to_string())
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::SpaceCreated(res)),
            )
        } else {
            Task::none()
        }
    }

    pub fn handle_select_space(
        &mut self,
        space_id: Option<std::sync::Arc<str>>,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        self.needs_layout_scroll_restoration = true;
        self.needs_threaded_layout_scroll_restoration = true;
        self.is_room_list_open = true;
        let parsed = space_id
            .as_deref()
            .and_then(|id| matrix_sdk::ruma::RoomId::parse(id).ok());
        if parsed.is_none()
            && matches!(
                self.list_selection,
                Some(crate::constellation::ListSelection::Rooms { .. })
            )
        {
            self.list_selection = None;
        }
        self.selected_space = parsed.clone();
        // Keep the nav bar's highlighted entry in sync with the selection.
        self.sync_space_nav_activation();
        // Clear other_rooms immediately when switching to avoid stale data from previous space
        self.other_rooms.clear();

        let mut tasks = Vec::new();

        if let Some(matrix) = &self.matrix {
            let matrix_clone = matrix.clone();
            let sid = parsed.clone();
            tasks.push(Task::perform(
                async move {
                    let _ = matrix_clone.update_room_list_filter(sid).await;
                },
                |_| Action::from(Message::SpaceFilterUpdated),
            ));
            if let Some(space_id) = parsed {
                let matrix_clone = matrix.clone();
                tasks.push(Task::perform(
                    async move {
                        let res = matrix_clone
                            .get_space_children(space_id.as_str())
                            .await
                            .map_err(|e| e.to_string());
                        (space_id, res)
                    },
                    move |(space_id, res)| {
                        Action::from(Message::SpaceChildrenFetched(space_id, res))
                    },
                ));
            } else {
                self.other_rooms.clear();
            }
        }

        self.update_filtered_rooms();
        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    pub fn handle_close_space_switcher(
        &mut self,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        self.needs_layout_scroll_restoration = true;
        self.needs_threaded_layout_scroll_restoration = true;
        self.is_room_list_open = false;
        self.selected_space = None;
        self.sync_space_nav_activation();
        self.other_rooms.clear();
        if matches!(
            self.list_selection,
            Some(crate::constellation::ListSelection::Rooms { .. })
        ) {
            self.list_selection = None;
        }

        let mut tasks = Vec::new();
        if let Some(matrix) = &self.matrix {
            let matrix_clone = matrix.clone();
            tasks.push(Task::perform(
                async move {
                    let _ = matrix_clone.update_room_list_filter(None).await;
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

    fn fetch_space_child_avatars(
        &self,
        children: &[matrix::RoomData],
    ) -> Task<Action<<Constellation as Application>::Message>> {
        let Some(matrix) = &self.matrix else {
            return Task::none();
        };

        if !self.user_settings.invite_avatars_display_policy {
            return Task::none();
        }

        let mut urls_to_fetch = Vec::new();
        for child in children {
            if let Some(avatar_url) = &child.avatar_url
                && !self.media_cache.contains_key(avatar_url)
            {
                let uri = matrix_sdk::ruma::OwnedMxcUri::from(avatar_url.as_str());
                let source = MediaSource::Plain(uri);
                urls_to_fetch.push((avatar_url.clone(), source));
            }
        }

        if urls_to_fetch.is_empty() {
            return Task::none();
        }

        let matrix_clone = matrix.clone();
        Task::perform(
            async move {
                futures::stream::iter(urls_to_fetch)
                    .map(|(url_str, source)| {
                        let matrix = matrix_clone.clone();
                        async move {
                            let res = matrix.fetch_media(source).await.map_err(|e| e.to_string());
                            (url_str, res)
                        }
                    })
                    .buffer_unordered(10)
                    .collect::<Vec<_>>()
                    .await
            },
            |batch| Message::MediaFetchedBatch(batch).into(),
        )
    }

    pub fn handle_space_children_fetched(
        &mut self,
        space_id: OwnedRoomId,
        res: Result<Vec<matrix::RoomData>, String>,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        // Only update if the fetched children are for the currently selected space
        if Some(&space_id) != self.selected_space.as_ref() {
            return Task::none();
        }

        let mut tasks = Vec::new();

        match res {
            Ok(children) => {
                // First, update the filtered_room_list because the hierarchy in matrix engine was updated
                self.update_filtered_rooms();

                // Re-trigger the SDK filter with the new hierarchy data
                if let Some(matrix) = &self.matrix {
                    let matrix_clone = matrix.clone();
                    let sid = space_id.clone();
                    tasks.push(Task::perform(
                        async move {
                            let _ = matrix_clone.update_room_list_filter(Some(sid)).await;
                        },
                        |_| Action::from(Message::SpaceFilterUpdated),
                    ));
                }

                let fetch_avatars_task = self.fetch_space_child_avatars(&children);
                tasks.push(fetch_avatars_task);

                let mut other_rooms: Vec<_> = children
                    .into_iter()
                    .filter(|r| !self.joined_room_ids.contains(r.id.as_ref()) && !r.is_space)
                    .collect();

                other_rooms.sort_by(|a, b| match (&a.order, &b.order) {
                    (Some(oa), Some(ob)) => oa.cmp(ob).then_with(|| a.id.cmp(&b.id)),
                    (Some(_), None) => std::cmp::Ordering::Less,
                    (None, Some(_)) => std::cmp::Ordering::Greater,
                    (None, None) => a.id.cmp(&b.id),
                });

                self.other_rooms = other_rooms;
            }
            Err(e) => {
                self.set_error(
                    crate::fl!("error-failed-fetch-space-children", error = e.to_string())
                        .to_string(),
                );
            }
        }

        if tasks.is_empty() {
            Task::none()
        } else {
            Task::batch(tasks)
        }
    }

    pub fn handle_unpin_message(
        &mut self,
        event_id: matrix_sdk::ruma::OwnedEventId,
    ) -> Task<Action<<Constellation as Application>::Message>> {
        self.is_loading_pinned = true;
        self.unpin_message_task(event_id)
    }

    pub(super) fn fetch_members_task(&self) -> Task<Action<Message>> {
        let Some(room_id) = self.selected_room.clone() else {
            return Task::none();
        };
        let Some(matrix) = self.matrix.clone() else {
            return Task::none();
        };
        Task::perform(
            async move {
                matrix
                    .get_room_members(&room_id)
                    .await
                    .map_err(|e| e.to_string())
            },
            |res| Action::from(Message::MembersFetched(res)),
        )
    }

    pub(super) fn fetch_pinned_events_task(&self) -> Task<Action<Message>> {
        let Some(room_id) = self.selected_room.clone() else {
            return Task::none();
        };
        let Some(matrix) = self.matrix.clone() else {
            return Task::none();
        };
        Task::perform(
            async move {
                let ids = matrix
                    .get_pinned_events(&room_id)
                    .await
                    .map_err(|e| e.to_string())?;
                let futures = ids.into_iter().map(|id| {
                    let matrix = matrix.clone();
                    let room_id = room_id.clone();
                    async move {
                        match matrix.fetch_pinned_event_details(&room_id, &id).await {
                            Ok(detail) => detail,
                            Err(e) => {
                                tracing::error!(
                                    "Failed to fetch details for pinned event {}: {}",
                                    id,
                                    e
                                );
                                matrix::PinnedEventInfo {
                                    event_id: id.to_string(),
                                    sender_id: "@unknown:example.com".to_string(),
                                    sender_name: crate::fl!("unknown-sender").to_string(),
                                    avatar_url: None,
                                    timestamp: crate::fl!("unknown-time").to_string(),
                                    body: crate::fl!(
                                        "error-failed-load-message-content",
                                        error = e.to_string()
                                    )
                                    .to_string(),
                                }
                            }
                        }
                    }
                });
                let details = futures::future::join_all(futures).await;
                Ok(details)
            },
            |res| Action::from(Message::PinnedEventsFetched(res)),
        )
    }

    pub(super) fn fetch_active_threads_task(&self) -> Task<Action<Message>> {
        let Some(room_id) = self.selected_room.clone() else {
            return Task::none();
        };
        let Some(matrix) = self.matrix.clone() else {
            return Task::none();
        };
        Task::perform(
            async move {
                matrix
                    .fetch_active_threads(&room_id)
                    .await
                    .map_err(|e| e.to_string())
            },
            |res| Action::from(Message::ActiveThreadsFetched(res)),
        )
    }

    /// Removes an event from the room's pinned list, then refreshes the panel.
    fn unpin_message_task(
        &self,
        event_id: matrix_sdk::ruma::OwnedEventId,
    ) -> Task<Action<Message>> {
        let Some(room_id) = self.selected_room.clone() else {
            return Task::none();
        };
        let Some(matrix) = self.matrix.clone() else {
            return Task::none();
        };
        let mut details = self.pinned_events_details.clone();
        details.retain(|info| info.event_id != event_id.as_str());

        Task::perform(
            async move {
                let current = matrix
                    .get_pinned_events(&room_id)
                    .await
                    .map_err(|e| e.to_string())?;
                let updated: Vec<_> = current.into_iter().filter(|id| id != &event_id).collect();
                matrix
                    .set_pinned_events(&room_id, updated)
                    .await
                    .map_err(|e| e.to_string())?;

                Ok(details)
            },
            |res| Action::from(Message::PinnedEventsFetched(res)),
        )
    }

    pub(super) fn handle_room_selected(
        &mut self,
        room_id: std::sync::Arc<str>,
    ) -> Task<Action<Message>> {
        let tab = Tab::Room(room_id.clone());
        if !self.open_tabs.contains(&tab) {
            self.open_tabs.push(tab.clone());
        }
        self.activate_tab(tab)
    }

    pub(super) fn activate_tab(&mut self, tab: Tab) -> Task<Action<Message>> {
        if let Some(active_search_tab) = self.active_search.take() {
            let state = crate::constellation::SearchTabState {
                public_search_results: std::mem::take(&mut self.public_search_results),
                is_searching_public: self.is_searching_public,
                message_search_results: std::mem::take(&mut self.message_search_results),
                is_searching_messages: self.is_searching_messages,
                search_has_more: self.search_has_more,
                is_searching_more_messages: self.is_searching_more_messages,
                global_message_search_results: std::mem::take(
                    &mut self.global_message_search_results,
                ),
                is_searching_global_messages: self.is_searching_global_messages,
                global_search_scope: self.global_search_scope,
            };
            self.search_results.insert(active_search_tab, state);
            self.is_searching_public = false;
            self.is_searching_messages = false;
            self.search_has_more = false;
            self.is_searching_more_messages = false;
            self.is_searching_global_messages = false;
        }

        match tab {
            Tab::Room(room_id) => {
                self.active_search = None;
                self.search_query.clear();
                self.update_filtered_rooms();
                if self.selected_room.as_ref() == Some(&room_id) {
                    if self.active_thread_root.is_some() {
                        self.active_thread_root = None;
                        self.needs_layout_scroll_restoration = true;
                        self.sync_tab_activation();
                        Task::batch(vec![self.update_title(), self.restore_scroll_task()])
                    } else {
                        self.sync_tab_activation();
                        Task::none()
                    }
                } else {
                    self.save_current_room_scroll();
                    self.selected_room = Some(room_id.clone());
                    self.active_thread_root = None;
                    self.rebuild_tab_model();
                    self.load_room_data(room_id)
                }
            }
            Tab::Thread { room_id, root_id } => {
                self.active_search = None;
                self.search_query.clear();
                self.update_filtered_rooms();
                if self.selected_room.as_ref() == Some(&room_id) {
                    self.active_thread_root = Some(root_id.clone());
                    self.sync_tab_activation();
                    self.setup_thread_timeline(root_id)
                } else {
                    self.save_current_room_scroll();
                    self.selected_room = Some(room_id.clone());
                    self.active_thread_root = Some(root_id.clone());
                    self.rebuild_tab_model();
                    let room_task = self.load_room_data(room_id);
                    let thread_task = self.setup_thread_timeline(root_id);
                    Task::batch(vec![room_task, thread_task])
                }
            }
            Tab::Search { room_id, query } => {
                self.active_thread_root = None;
                let search_tab = Tab::Search {
                    room_id: room_id.clone(),
                    query: query.clone(),
                };
                self.active_search = Some(search_tab.clone());
                self.selected_room = room_id;
                self.search_query = query.clone();
                if let Some(saved) = self.search_results.get(&search_tab) {
                    self.public_search_results = saved.public_search_results.clone();
                    self.is_searching_public = saved.is_searching_public;
                    self.message_search_results = saved.message_search_results.clone();
                    self.is_searching_messages = saved.is_searching_messages;
                    self.search_has_more = saved.search_has_more;
                    self.is_searching_more_messages = saved.is_searching_more_messages;
                    self.global_message_search_results =
                        saved.global_message_search_results.clone();
                    self.is_searching_global_messages = saved.is_searching_global_messages;
                    self.global_search_scope = saved.global_search_scope;
                }
                self.update_filtered_rooms();
                self.sync_tab_activation();
                self.update_title()
            }
        }
    }

    pub(super) fn save_current_room_scroll(&mut self) {
        if let Some(prev_room) = self.selected_room.clone() {
            self.room_scroll_memory.remove(&prev_room);
            if !self.is_timeline_at_bottom && self.active_event_focus.is_none() {
                let anchor =
                    scroll::decode_anchor(self.last_timeline_offset, &self.scroll_main.children);
                if let Some(anchor) = anchor {
                    self.room_scroll_memory.insert(prev_room, anchor);
                }
            }
        }
    }

    pub(super) fn load_room_data(&mut self, room_id: std::sync::Arc<str>) -> Task<Action<Message>> {
        if let Some(room) = self.room_by_id(&room_id)
            && let Some(name) = &room.name
        {
            self.room_name_cache.insert(room_id.clone(), name.clone());
        }
        // Drop in-app video players from the previous room; this stops their
        // GStreamer pipelines and removes the backing temp files.
        #[cfg(feature = "video-player")]
        {
            self.video_cache.clear();
            self.loading_videos.clear();
        }
        self.timeline_items.clear();
        self.room_members.clear();
        self.pinned_events.clear();
        self.pinned_events_details.clear();
        self.active_threads.clear();
        // Message search results are scoped to the previous room;
        // clear them so stale hits don't bleed into the new room. Global
        // search results are also room-context-sensitive (they only run when
        // no room is selected), so clear them too.
        self.message_search_results.clear();
        self.is_searching_messages = false;
        self.search_has_more = false;
        self.is_searching_more_messages = false;
        self.global_message_search_results.clear();
        self.is_searching_global_messages = false;
        self.inviting_to_room = false;
        self.invite_to_room_id.clear();
        // A room switch always leaves the event-focused (permalink
        // context) view: clear any pending/active event focus so the
        // new room opens on its live timeline and the banner hides.
        self.pending_event_focus = None;
        self.active_event_focus = None;
        let fetch_members_task = if self.show_members_panel {
            self.is_loading_members = true;
            self.fetch_members_task()
        } else {
            Task::none()
        };
        let fetch_pinned_task = self.fetch_pinned_events_task();
        let fetch_active_threads_task = self.fetch_active_threads_task();
        let fetch_packs_task = if let Ok(parsed_id) = matrix_sdk::ruma::RoomId::parse(&*room_id) {
            Task::done(Action::from(Message::LoadRoomImagePacks(parsed_id)))
        } else {
            Task::none()
        };
        self.recompute_timeline_metadata();
        self.last_timeline_offset = 0.0;
        self.last_content_height = 0.0;
        self.last_viewport_width = 0.0;
        self.last_viewport_height = 0.0;
        self.needs_scroll_adjustment = false;
        // Fresh timeline context: drop measured geometry and any in-flight
        // row measurements from the previous room.
        self.scroll_generation += 1;
        self.scroll_main.reset();
        self.scroll_thread.reset();
        self.is_timeline_at_bottom = true;
        self.is_threaded_timeline_at_bottom = true;
        self.is_timeline_initialized = false;
        self.is_first_time_joining = false;
        self.visited_room_ids.insert(room_id.clone());
        self.needs_initial_scroll = true;
        // If we have a memorized position for this room, resume it instead of
        // the default placement once the timeline has been measured.
        self.pending_room_restore = self.room_scroll_memory.get(&room_id).cloned();

        Task::batch(vec![
            self.update_title(),
            self.handle_load_more(false),
            fetch_members_task,
            fetch_pinned_task,
            fetch_packs_task,
            fetch_active_threads_task,
        ])
    }

    pub(super) fn mark_thread_read_task(&self, root_id: OwnedEventId) -> Task<Action<Message>> {
        let (Some(matrix), Some(room_id)) = (&self.matrix, &self.selected_room) else {
            return Task::none();
        };
        let matrix = matrix.clone();
        let room_id = room_id.clone();
        Task::perform(
            async move {
                let _ = matrix
                    .mark_threaded_timeline_as_read(&room_id, &root_id)
                    .await;
            },
            |_| Action::None,
        )
    }

    pub(super) fn setup_thread_timeline(&mut self, root_id: OwnedEventId) -> Task<Action<Message>> {
        self.needs_layout_scroll_restoration = true;
        self.threaded_timeline_items.clear();
        self.last_threaded_timeline_offset = 0.0;
        self.last_threaded_content_height = 0.0;
        self.last_threaded_viewport_width = 0.0;
        self.last_threaded_viewport_height = 0.0;
        self.needs_threaded_scroll_adjustment = false;
        self.scroll_thread.reset();
        self.is_threaded_timeline_initialized = false;

        // Clear local unread counts for the active thread
        self.thread_unreads
            .insert(root_id.clone(), matrix::ThreadUnread::default());
        let root_id_str = root_id.as_str();
        for item in &mut self.active_threads {
            if item.event_id == root_id_str {
                item.num_unread_messages = 0;
                item.num_unread_notifications = 0;
                item.num_unread_mentions = 0;
            }
        }
        self.rebuild_tab_model();

        let mark_read_task = self.mark_thread_read_task(root_id);

        Task::batch(vec![
            self.update_title(),
            self.handle_load_more(true),
            mark_read_task,
            scrollable::snap_to(
                THREADED_TIMELINE_ID.clone(),
                scrollable::RelativeOffset::END.into(),
            ),
        ])
    }

    pub fn rebuild_tab_model(&mut self) {
        let mut model = cosmic::widget::segmented_button::SingleSelectModel::default();
        let active_tab = self.active_tab();
        for tab in &self.open_tabs {
            let label = match tab {
                Tab::Room(room_id) => self
                    .get_room_name(room_id)
                    .unwrap_or_else(|| crate::view::UNKNOWN_ROOM.as_str())
                    .to_string(),
                Tab::Thread { room_id, root_id } => {
                    let room_name = self
                        .get_room_name(room_id)
                        .unwrap_or_else(|| crate::view::UNKNOWN_ROOM.as_str());
                    let mut text = format!("{}: {}", crate::fl!("thread"), room_name);
                    if active_tab.as_ref() != Some(tab)
                        && let Some(unread) = self.thread_unreads.get(root_id)
                        && unread.is_unread()
                    {
                        let count = unread.display_count();
                        if unread.num_unread_mentions > 0 {
                            text.push_str(&format!(" (@{count})"));
                        } else {
                            text.push_str(&format!(" ({count})"));
                        }
                    }
                    text
                }
                Tab::Search { query, .. } => {
                    format!("{}: {}", crate::fl!("search"), query)
                }
            };

            let mut entity = model.insert().text(label).closable().data(tab.clone());

            if active_tab.as_ref() == Some(tab) {
                entity = entity.activate();
            }
        }
        self.tab_model = model;
    }

    pub fn sync_tab_activation(&mut self) {
        let active_tab = self.active_tab();
        let entities: Vec<_> = self.tab_model.iter().collect();
        let mut target = None;
        for entity in entities {
            let tab = self.tab_model.data::<Tab>(entity);
            if tab == active_tab.as_ref() {
                target = Some(entity);
                break;
            }
        }
        self.tab_model.deactivate();
        if let Some(entity) = target {
            self.tab_model.activate(entity);
        }
    }

    pub fn handle_close_tab(&mut self, tab: &Tab) -> Task<Action<Message>> {
        let Some(pos) = self.open_tabs.iter().position(|t| t == tab) else {
            return Task::none();
        };

        let is_active = self.active_tab().as_ref() == Some(tab);
        let tab_to_close = self.open_tabs.remove(pos);

        if is_active {
            if let Tab::Thread { .. } = tab_to_close {
                self.active_thread_root = None;
                self.threaded_timeline_items.clear();
                self.last_threaded_timeline_offset = 0.0;
                self.last_threaded_content_height = 0.0;
                self.last_threaded_viewport_width = 0.0;
                self.last_threaded_viewport_height = 0.0;
                self.needs_threaded_scroll_adjustment = false;
                self.scroll_thread.reset();
                self.is_threaded_timeline_initialized = false;
            }
            if let Tab::Search { .. } = &tab_to_close {
                self.active_search = None;
                self.search_results.remove(&tab_to_close);
                self.public_search_results.clear();
                self.is_searching_public = false;
                self.message_search_results.clear();
                self.is_searching_messages = false;
                self.search_has_more = false;
                self.is_searching_more_messages = false;
                self.global_message_search_results.clear();
                self.is_searching_global_messages = false;
            }

            if !self.open_tabs.is_empty() {
                let next_idx = if pos < self.open_tabs.len() {
                    pos
                } else {
                    self.open_tabs.len() - 1
                };
                let next_tab = self.open_tabs[next_idx].clone();
                self.rebuild_tab_model();
                self.activate_tab(next_tab)
            } else {
                self.open_tabs.clear();
                self.selected_room = None;
                self.active_thread_root = None;
                self.rebuild_tab_model();
                self.active_search = None;
                self.search_results.clear();
                #[cfg(feature = "video-player")]
                {
                    self.video_cache.clear();
                    self.loading_videos.clear();
                }
                self.timeline_items.clear();
                self.room_members.clear();
                self.pinned_events.clear();
                self.pinned_events_details.clear();
                self.active_threads.clear();
                self.message_search_results.clear();
                self.is_searching_messages = false;
                self.search_has_more = false;
                self.is_searching_more_messages = false;
                self.global_message_search_results.clear();
                self.is_searching_global_messages = false;
                self.inviting_to_room = false;
                self.invite_to_room_id.clear();
                self.pending_event_focus = None;
                self.active_event_focus = None;
                self.recompute_timeline_metadata();
                self.search_query.clear();
                self.update_filtered_rooms();
                self.update_title()
            }
        } else {
            if let Tab::Search { .. } = &tab_to_close {
                self.search_results.remove(&tab_to_close);
            }
            self.rebuild_tab_model();
            Task::none()
        }
    }

    pub fn handle_close_room(&mut self, room_id: std::sync::Arc<str>) -> Task<Action<Message>> {
        self.handle_close_tab(&Tab::Room(room_id))
    }

    pub(super) fn handle_close_thread(&mut self) -> Task<Action<Message>> {
        if let (Some(room_id), Some(root_id)) =
            (self.selected_room.clone(), self.active_thread_root.clone())
        {
            self.handle_close_tab(&Tab::Thread { room_id, root_id })
        } else {
            self.active_thread_root = None;
            self.threaded_timeline_items.clear();
            Task::none()
        }
    }

    pub(super) fn handle_open_settings(&mut self, panel: SettingsPanel) -> Task<Action<Message>> {
        self.needs_layout_scroll_restoration = true;
        self.needs_threaded_layout_scroll_restoration = true;
        self.show_members_panel = false;
        self.show_pinned_panel = false;
        self.show_active_threads_panel = false;
        self.creating_room = false;
        self.creating_space = false;
        self.current_settings_panel = Some(panel.clone());
        self.core.set_show_context(true);

        if self.is_search_active {
            match panel {
                SettingsPanel::Room => {
                    self.room_settings.member_filter = self.search_query.clone();
                }
                SettingsPanel::Space => {
                    self.space_settings.child_filter = self.search_query.clone();
                }
                _ => {}
            }
        }

        let task = if panel == SettingsPanel::User {
            self.user_settings
                .update(settings::user::Message::LoadProfile, &self.matrix)
        } else if matches!(
            panel,
            SettingsPanel::Room | SettingsPanel::Permissions | SettingsPanel::ManageRoomMembers
        ) {
            if let Some(room_id) = &self.selected_room {
                self.room_settings.update(
                    settings::room::Message::LoadRoom(room_id.clone()),
                    &self.matrix,
                )
            } else {
                Task::none()
            }
        } else if panel == SettingsPanel::Space
            && let Some(space_id) = &self.selected_space
        {
            self.space_settings.update(
                settings::space::Message::LoadSpace(Arc::from(space_id.as_str())),
                &self.matrix,
            )
        } else {
            Task::none()
        };
        Task::batch(vec![task, self.update_title(), self.restore_scroll_task()])
    }

    pub(super) fn handle_room_joined(
        &mut self,
        res: Result<matrix_sdk::ruma::OwnedRoomId, String>,
    ) -> Task<Action<Message>> {
        match res {
            Ok(room_id) => {
                self.selected_room = Some(room_id.as_str().into());
                self.is_first_time_joining = true;
                self.visited_room_ids.insert(room_id.as_str().into());
                // Refresh both lists
                self.update_filtered_rooms();
                if let (Some(matrix), Some(space_id)) = (&self.matrix, &self.selected_space) {
                    let matrix = matrix.clone();
                    let sid = space_id.clone();
                    let sid_clone = sid.clone();
                    return Task::perform(
                        async move {
                            matrix
                                .get_space_children(sid_clone.as_str())
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| Message::SpaceChildrenFetched(sid, res).into(),
                    );
                }
            }
            Err(e) => {
                self.set_error(
                    crate::fl!("error-failed-join-room", error = e.to_string()).to_string(),
                );
            }
        }
        Task::none()
    }

    pub(super) fn handle_tab_activated(
        &mut self,
        entity: cosmic::widget::segmented_button::Entity,
    ) -> Task<Action<Message>> {
        if let Some(tab) = self.tab_model.data::<Tab>(entity).cloned() {
            if self.active_tab().as_ref() != Some(&tab) {
                self.activate_tab(tab)
            } else {
                Task::none()
            }
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_tab_closed(
        &mut self,
        entity: cosmic::widget::segmented_button::Entity,
    ) -> Task<Action<Message>> {
        if let Some(tab) = self.tab_model.data::<Tab>(entity).cloned() {
            self.handle_close_tab(&tab)
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_copy_active_room_link(&mut self) -> Task<Action<Message>> {
        if let Some(room_id) = self.selected_room.clone() {
            self.handle_copy_room_link(room_id)
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_close_active_tab(&mut self) -> Task<Action<Message>> {
        if let Some(tab) = self.active_tab() {
            self.handle_close_tab(&tab)
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_close_active_room(&mut self) -> Task<Action<Message>> {
        self.handle_close_active_tab()
    }

    pub(super) fn handle_toggle_create_room(&mut self) -> Task<Action<Message>> {
        self.creating_room = !self.creating_room;
        self.creating_space = false;
        self.new_room_name.clear();
        self.current_settings_panel = None;
        self.core.set_show_context(self.creating_room);
        Task::none()
    }

    pub(super) fn handle_toggle_create_space(&mut self) -> Task<Action<Message>> {
        self.creating_space = !self.creating_space;
        self.creating_room = false;
        self.new_room_name.clear();
        self.current_settings_panel = None;
        self.core.set_show_context(self.creating_space);
        Task::none()
    }

    pub(super) fn handle_toggle_invite_to_space(&mut self) -> Task<Action<Message>> {
        self.inviting_to_space = !self.inviting_to_space;
        if self.inviting_to_space {
            self.creating_room = false;
            self.creating_space = false;
        }
        self.invite_to_space_id.clear();
        Task::none()
    }

    pub(super) fn handle_invite_to_space_id_changed(
        &mut self,
        id: String,
    ) -> Task<Action<Message>> {
        self.invite_to_space_id = id;
        Task::none()
    }

    pub(super) fn handle_invite_to_space(&mut self) -> Task<Action<Message>> {
        if let Some(matrix) = &self.matrix
            && let Some(space_id) = &self.selected_space
        {
            let matrix = matrix.clone();
            let space_id = space_id.to_string();
            let user_id = self.invite_to_space_id.clone();
            Task::perform(
                async move {
                    matrix
                        .invite_user(&space_id, &user_id)
                        .await
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::SpaceUserInvited(res)),
            )
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_space_user_invited(
        &mut self,
        res: Result<(), String>,
    ) -> Task<Action<Message>> {
        match res {
            Ok(_) => {
                self.inviting_to_space = false;
                self.invite_to_space_id.clear();
            }
            Err(e) => {
                self.set_error(
                    crate::fl!("error-failed-invite", error = e.to_string()).to_string(),
                );
            }
        }
        Task::none()
    }

    pub(super) fn handle_toggle_invite_to_room(&mut self) -> Task<Action<Message>> {
        self.inviting_to_room = !self.inviting_to_room;
        self.invite_to_room_id.clear();
        Task::none()
    }

    pub(super) fn handle_invite_to_room_id_changed(&mut self, id: String) -> Task<Action<Message>> {
        self.invite_to_room_id = id;
        Task::none()
    }

    pub(super) fn handle_invite_to_room(&mut self) -> Task<Action<Message>> {
        if let Some(matrix) = &self.matrix
            && let Some(room_id) = &self.selected_room
        {
            let matrix = matrix.clone();
            let room_id = room_id.to_string();
            let user_id = self.invite_to_room_id.clone();
            Task::perform(
                async move {
                    matrix
                        .invite_user(&room_id, &user_id)
                        .await
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::RoomUserInvited(res)),
            )
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_room_user_invited(
        &mut self,
        res: Result<(), String>,
    ) -> Task<Action<Message>> {
        match res {
            Ok(_) => {
                self.inviting_to_room = false;
                self.invite_to_room_id.clear();
            }
            Err(e) => {
                self.set_error(
                    crate::fl!("error-failed-invite", error = e.to_string()).to_string(),
                );
            }
        }
        Task::none()
    }

    pub(super) fn handle_new_room_name_changed(&mut self, name: String) -> Task<Action<Message>> {
        self.new_room_name = name;
        Task::none()
    }

    pub(super) fn handle_room_created(
        &mut self,
        res: Result<String, String>,
    ) -> Task<Action<Message>> {
        match res {
            Ok(room_id) => {
                self.creating_room = false;
                self.new_room_name.clear();
                self.selected_room = Some(room_id.as_str().into());
                self.core.set_show_context(false);
            }
            Err(e) => {
                self.set_error(
                    crate::fl!("error-failed-create-room", error = e.to_string()).to_string(),
                );
            }
        }
        Task::none()
    }

    pub(super) fn handle_space_created(
        &mut self,
        res: Result<String, String>,
    ) -> Task<Action<Message>> {
        match res {
            Ok(space_id) => {
                self.creating_space = false;
                self.new_room_name.clear();
                self.core.set_show_context(false);
                return self.handle_select_space(Some(space_id.as_str().into()));
            }
            Err(e) => {
                self.set_error(
                    crate::fl!("error-failed-create-space", error = e.to_string()).to_string(),
                );
            }
        }
        Task::none()
    }

    pub(super) fn handle_join_room(
        &mut self,
        room_id: std::sync::Arc<str>,
    ) -> Task<Action<Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix = matrix.clone();
            return Task::perform(
                async move {
                    let rid =
                        matrix_sdk::ruma::RoomId::parse(&*room_id).map_err(|e| e.to_string())?;
                    matrix
                        .join_room(&rid)
                        .await
                        .map(|_| rid)
                        .map_err(|e| e.to_string())
                },
                |res| Message::RoomJoined(res).into(),
            );
        }
        Task::none()
    }

    pub(super) fn handle_call_joined(&mut self, res: Result<(), String>) -> Task<Action<Message>> {
        if let Err(e) = res {
            self.set_error(crate::fl!("error-failed-join-call", error = e.to_string()).to_string());
        }
        Task::none()
    }

    pub(super) fn handle_call_left(&mut self, res: Result<(), String>) -> Task<Action<Message>> {
        if let Err(e) = res {
            self.set_error(
                crate::fl!("error-failed-leave-call", error = e.to_string()).to_string(),
            );
        }
        Task::none()
    }

    pub(super) fn handle_load_room_image_packs(
        &self,
        room_id: matrix_sdk::ruma::OwnedRoomId,
    ) -> Task<Action<Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix_clone = matrix.clone();
            let rid = room_id.clone();
            Task::perform(
                async move {
                    matrix_clone
                        .get_room_image_packs(&rid)
                        .await
                        .map_err(|e| e.to_string())
                },
                move |res| Action::from(Message::RoomImagePacksLoaded(room_id, res)),
            )
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_room_image_packs_loaded(
        &mut self,
        room_id: matrix_sdk::ruma::OwnedRoomId,
        res: Result<Vec<matrix::ImagePack>, String>,
    ) -> Task<Action<Message>> {
        match res {
            Ok(packs) => {
                let mut updated_packs = packs;
                if let Some(state_keys) = self.global_pack_rooms.get(&room_id) {
                    for pack in &mut updated_packs {
                        if state_keys.contains_key(&pack.state_key) {
                            pack.is_globally_enabled = true;
                        }
                    }
                }
                self.room_image_packs.insert(room_id, updated_packs);
                self.update_active_emojis_and_stickers();

                // Prefetch thumbnails of custom emojis and stickers so they render fast
                let mut media_fetches = Vec::new();
                for emoji in &self.active_custom_emojis {
                    if !self.media_cache.contains_key(&emoji.url) && emoji.url.starts_with("mxc://")
                    {
                        let mxc_uri = matrix_sdk::ruma::OwnedMxcUri::from(emoji.url.as_str());
                        media_fetches.push(crate::MediaSource::Plain(mxc_uri));
                    }
                }
                for sticker in &self.active_stickers {
                    if !self.media_cache.contains_key(&sticker.url)
                        && sticker.url.starts_with("mxc://")
                    {
                        let mxc_uri = matrix_sdk::ruma::OwnedMxcUri::from(sticker.url.as_str());
                        media_fetches.push(crate::MediaSource::Plain(mxc_uri));
                    }
                }
                if !media_fetches.is_empty()
                    && let Some(matrix) = &self.matrix
                {
                    let matrix = matrix.clone();
                    return Task::perform(
                        async move {
                            let mut results = Vec::new();
                            for source in media_fetches {
                                let mxc_url = match &source {
                                    crate::MediaSource::Plain(uri) => uri.to_string(),
                                    crate::MediaSource::Encrypted(file) => file.url.to_string(),
                                };
                                let res =
                                    matrix.fetch_media(source).await.map_err(|e| e.to_string());
                                results.push((mxc_url, res));
                            }
                            results
                        },
                        |batch| Action::from(Message::MediaFetchedBatch(batch)),
                    );
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load room image packs for {room_id}: {e}");
            }
        }
        Task::none()
    }

    pub(super) fn handle_load_account_image_packs(&self) -> Task<Action<Message>> {
        if let Some(matrix) = &self.matrix {
            let matrix_clone = matrix.clone();
            Task::perform(
                async move {
                    matrix_clone
                        .get_account_image_packs()
                        .await
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::AccountImagePacksLoaded(res)),
            )
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_account_image_packs_loaded(
        &mut self,
        res: Result<matrix::AccountImagePacksData, String>,
    ) -> Task<Action<Message>> {
        match res {
            Ok((personal_packs, rooms_map)) => {
                self.user_image_packs = personal_packs;
                self.global_pack_rooms = rooms_map;

                let mut tasks = Vec::new();
                for room_id in self.global_pack_rooms.keys() {
                    if !self.room_image_packs.contains_key(room_id) {
                        tasks.push(Task::done(Action::from(Message::LoadRoomImagePacks(
                            room_id.clone(),
                        ))));
                    }
                }
                self.update_active_emojis_and_stickers();
                if !tasks.is_empty() {
                    return Task::batch(tasks);
                }
            }
            Err(e) => {
                tracing::warn!("Failed to load account image packs: {e}");
            }
        }
        Task::none()
    }
}
