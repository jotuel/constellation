use crate::constellation::Tab;
use crate::{Constellation, Message, SettingsPanel};
use cosmic::{Action, Task};
use futures::FutureExt;
use futures::stream::StreamExt;

impl Constellation {
    pub(super) fn handle_toggle_search(&mut self) -> Task<Action<Message>> {
        self.is_search_active = !self.is_search_active;
        if !self.is_search_active {
            self.search_query.clear();
            self.room_settings.member_filter.clear();
            self.space_settings.child_filter.clear();
        } else if let Some(panel) = &self.current_settings_panel {
            match panel {
                SettingsPanel::Room => {
                    self.search_query = self.room_settings.member_filter.clone();
                }
                SettingsPanel::Space => {
                    self.search_query = self.space_settings.child_filter.clone();
                }
                _ => {}
            }
        }
        self.update_filtered_rooms();
        self.update_title()
    }

    pub(super) fn handle_submit_search(&mut self) -> Task<Action<Message>> {
        let query = self.search_query.trim().to_string();
        if query.is_empty() {
            return Task::none();
        }

        let room_id = self.selected_room.clone();
        let tab = Tab::Search {
            room_id: room_id.clone(),
            query: query.clone(),
        };

        if !self.open_tabs.contains(&tab) {
            let insert_pos = self
                .active_tab()
                .and_then(|at| self.open_tabs.iter().position(|t| t == &at))
                .map(|pos| pos + 1)
                .unwrap_or(self.open_tabs.len());
            self.open_tabs.insert(insert_pos, tab.clone());
        }

        self.rebuild_tab_model();
        let activate_task = self.activate_tab(tab.clone());
        let search_task = self.start_search_for_tab(tab);

        Task::batch(vec![activate_task, search_task])
    }

    pub(super) fn start_search_for_tab(&mut self, tab: Tab) -> Task<Action<Message>> {
        let Tab::Search { room_id, query } = &tab else {
            return Task::none();
        };

        let mut tasks = Vec::new();
        self.search_generation = self.search_generation.wrapping_add(1);
        let generation = self.search_generation;

        self.public_search_results.clear();
        self.message_search_results.clear();
        self.global_message_search_results.clear();
        self.search_has_more = false;
        self.is_searching_more_messages = false;

        if let Some(matrix) = &self.matrix {
            let query_str = query.trim().to_string();
            let matrix_public = matrix.clone();
            self.is_searching_public = true;

            tasks.push(Task::perform(
                async move { matrix_public.search_public_rooms(query_str, Some(20)).await },
                move |res| {
                    Action::from(Message::PublicSearchResults(
                        generation,
                        res.map_err(|e| e.to_string()),
                    ))
                },
            ));

            let query_str = query.trim().to_string();
            if let Some(room_id) = room_id {
                self.is_searching_messages = true;
                let room_id = room_id.clone();
                let matrix = matrix.clone();

                tasks.push(Task::perform(
                    async move {
                        matrix
                            .search_messages_in_room(&room_id, &query_str, 20)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    move |res| Action::from(Message::MessageSearchResults(generation, res)),
                ));
            } else {
                self.is_searching_global_messages = true;
                let scope = self.global_search_scope;
                let matrix = matrix.clone();
                tasks.push(Task::perform(
                    async move {
                        matrix
                            .search_messages_global(&query_str, 20, scope)
                            .await
                            .map_err(|e| e.to_string())
                    },
                    move |res| Action::from(Message::GlobalMessageSearchResults(generation, res)),
                ));
            }
        }

        Task::batch(tasks)
    }

    pub(super) fn ensure_search_tab(&mut self) {
        if self.active_search.is_none() && !self.search_query.trim().is_empty() {
            let tab = Tab::Search {
                room_id: self.selected_room.clone(),
                query: self.search_query.trim().to_string(),
            };
            if !self.open_tabs.contains(&tab) {
                let insert_pos = self
                    .active_tab()
                    .and_then(|at| self.open_tabs.iter().position(|t| t == &at))
                    .map(|pos| pos + 1)
                    .unwrap_or(self.open_tabs.len());
                self.open_tabs.insert(insert_pos, tab.clone());
            }
            self.active_search = Some(tab);
            self.rebuild_tab_model();
        }
    }

    pub(super) fn sync_active_search_results(&mut self) {
        if let Some(active_search_tab) = &self.active_search {
            let entry = self
                .search_results
                .entry(active_search_tab.clone())
                .or_default();
            entry.public_search_results = self.public_search_results.clone();
            entry.is_searching_public = self.is_searching_public;
            entry.message_search_results = self.message_search_results.clone();
            entry.is_searching_messages = self.is_searching_messages;
            entry.search_has_more = self.search_has_more;
            entry.is_searching_more_messages = self.is_searching_more_messages;
            entry.global_message_search_results = self.global_message_search_results.clone();
            entry.is_searching_global_messages = self.is_searching_global_messages;
            entry.global_search_scope = self.global_search_scope;
        }
    }

    pub(super) fn handle_search_query_changed(&mut self, query: String) -> Task<Action<Message>> {
        self.search_query = query.clone();
        self.search_has_more = false;
        self.is_searching_more_messages = false;
        if let Some(panel) = &self.current_settings_panel {
            match panel {
                SettingsPanel::Room => {
                    self.room_settings.member_filter = query.clone();
                }
                SettingsPanel::Space => {
                    self.space_settings.child_filter = query.clone();
                }
                _ => {}
            }
        }
        self.update_filtered_rooms();
        if let Some(active_search_tab) = self.active_search.clone() {
            let room_id = match active_search_tab {
                Tab::Search { ref room_id, .. } => room_id.clone(),
                _ => None,
            };
            let updated_tab = Tab::Search {
                room_id,
                query: self.search_query.trim().to_string(),
            };
            if let Some(pos) = self.open_tabs.iter().position(|t| t == &active_search_tab) {
                self.open_tabs[pos] = updated_tab.clone();
                self.active_search = Some(updated_tab);
                self.rebuild_tab_model();
            }
        }

        let title_task = self.update_title();
        if self.current_settings_panel.is_none() && !self.search_query.trim().is_empty() {
            let mut tasks = Vec::new();
            self.search_generation = self.search_generation.wrapping_add(1);
            let generation = self.search_generation;

            // Public rooms / spaces directory search (existing).
            if let Some(matrix) = &self.matrix {
                let query_str = self.search_query.trim().to_string();
                let matrix = matrix.clone();
                self.is_searching_public = true;

                tasks.push(Task::perform(
                    async move {
                        // Debounce: wait for typing to settle before
                        // querying the homeserver public room directory.
                        tokio::time::sleep(std::time::Duration::from_millis(350)).await;
                        matrix.search_public_rooms(query_str, Some(20)).await
                    },
                    move |res| {
                        Action::from(Message::PublicSearchResults(
                            generation,
                            res.map_err(|e| e.to_string()),
                        ))
                    },
                ));
            }

            // Message search. Exactly one of two branches fires per keystroke:
            // the in-room search when a room is selected, or the global
            // (cross-room) search when none is. Both are debounced and share
            // `search_generation` so a stale result from either is discarded.
            if let Some(matrix) = &self.matrix {
                let query_str = self.search_query.trim().to_string();

                if let Some(room_id) = &self.selected_room {
                    // In-room search.
                    self.is_searching_messages = true;
                    let room_id = room_id.clone();
                    let matrix = matrix.clone();

                    tasks.push(Task::perform(
                        async move {
                            // Debounce: wait for typing to settle before
                            // querying the homeserver search index.
                            tokio::time::sleep(std::time::Duration::from_millis(350)).await;
                            matrix
                                .search_messages_in_room(&room_id, &query_str, 20)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| Action::from(Message::MessageSearchResults(generation, res)),
                    ));
                } else {
                    // Global search across all joined rooms (local seshat
                    // index). The scope (All/DMs/Groups) is captured by copy;
                    // changing it re-fires the query via `SetGlobalSearchScope`.
                    self.is_searching_global_messages = true;
                    let scope = self.global_search_scope;
                    let matrix = matrix.clone();
                    tasks.push(Task::perform(
                        async move {
                            tokio::time::sleep(std::time::Duration::from_millis(350)).await;
                            matrix
                                .search_messages_global(&query_str, 20, scope)
                                .await
                                .map_err(|e| e.to_string())
                        },
                        move |res| {
                            Action::from(Message::GlobalMessageSearchResults(generation, res))
                        },
                    ));
                }
            }

            if tasks.is_empty() {
                title_task
            } else {
                tasks.push(title_task);
                Task::batch(tasks)
            }
        } else {
            self.public_search_results.clear();
            self.is_searching_public = false;
            self.message_search_results.clear();
            self.is_searching_messages = false;
            self.search_has_more = false;
            self.is_searching_more_messages = false;
            self.global_message_search_results.clear();
            self.is_searching_global_messages = false;
            // Invalidate any in-flight message search so a late result
            // doesn't repopulate stale hits for the cleared query.
            self.search_generation = self.search_generation.wrapping_add(1);
            title_task
        }
    }

    pub(super) fn handle_public_search_results(
        &mut self,
        generation: u64,
        res: Result<Vec<crate::matrix::PublicRoom>, String>,
    ) -> Task<Action<Message>> {
        // Discard stale results from a query the user has since edited.
        if generation != self.search_generation {
            return Task::none();
        }
        self.is_searching_public = false;
        self.ensure_search_tab();
        match res {
            Ok(results) => {
                self.public_search_results = results;

                let mut missing_sources = Vec::new();
                for room in &self.public_search_results {
                    if let Some(avatar_url) = &room.avatar_url
                        && !self.media_cache.contains_key(avatar_url)
                    {
                        // Performance optimization: parse the URL directly from the borrowed &str
                        // to avoid cloning the underlying String before creating the MediaSource.
                        missing_sources.push(crate::MediaSource::Plain(
                            matrix_sdk::ruma::OwnedMxcUri::from(avatar_url.as_str()),
                        ));
                    }
                }

                if !missing_sources.is_empty()
                    && let Some(matrix) = &self.matrix
                {
                    let matrix = matrix.clone();
                    let mut fetches = Vec::new();
                    for source in missing_sources {
                        let mxc_url = match &source {
                            crate::MediaSource::Plain(uri) => uri.to_string(),
                            crate::MediaSource::Encrypted(file) => file.url.to_string(),
                        };
                        let matrix_clone = matrix.clone();
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

                    return Task::perform(
                        async move {
                            futures::stream::iter(fetches)
                                .buffer_unordered(10)
                                .collect::<Vec<_>>()
                                .await
                        },
                        |results| Action::from(Message::MediaFetchedBatch(results)),
                    );
                }
            }
            Err(e) => {
                self.set_error(
                    crate::fl!("error-failed-search-public-rooms", error = e.to_string())
                        .to_string(),
                );
            }
        }
        self.sync_active_search_results();
        Task::none()
    }

    pub(super) fn handle_message_search_results(
        &mut self,
        generation: u64,
        res: Result<(Vec<crate::matrix::MessageSearchResult>, bool), String>,
    ) -> Task<Action<Message>> {
        // Discard stale results from a query the user has since edited.
        if generation != self.search_generation {
            return Task::none();
        }
        self.is_searching_messages = false;
        self.ensure_search_tab();
        match res {
            Ok((results, has_more)) => {
                self.message_search_results = results;
                self.search_has_more = has_more;
            }
            Err(e) => {
                self.message_search_results.clear();
                self.search_has_more = false;
                self.set_error(crate::fl!("search-server-failed", error = e).to_string());
            }
        }
        self.sync_active_search_results();
        Task::none()
    }

    pub(super) fn handle_load_more_message_search(&mut self) -> Task<Action<Message>> {
        if self.is_searching_more_messages {
            return Task::none();
        }
        if let Some(matrix) = &self.matrix {
            self.is_searching_more_messages = true;
            let matrix = matrix.clone();
            Task::perform(
                async move {
                    matrix
                        .search_messages_in_room_next_batch(20)
                        .await
                        .map_err(|e| e.to_string())
                },
                |res| Action::from(Message::MessageSearchMoreResults(res)),
            )
        } else {
            Task::none()
        }
    }

    pub(super) fn handle_message_search_more_results(
        &mut self,
        res: Result<(Vec<crate::matrix::MessageSearchResult>, bool), String>,
    ) -> Task<Action<Message>> {
        self.is_searching_more_messages = false;
        match res {
            Ok((results, has_more)) => {
                self.message_search_results.extend(results);
                self.search_has_more = has_more;
            }
            Err(e) => {
                self.set_error(crate::fl!("search-server-failed", error = e).to_string());
            }
        }
        self.sync_active_search_results();
        Task::none()
    }

    pub(super) fn handle_global_message_search_results(
        &mut self,
        generation: u64,
        res: Result<Vec<crate::matrix::MessageSearchResult>, String>,
    ) -> Task<Action<Message>> {
        // Same stale-discard guard as the in-room search; both share
        // `search_generation`.
        if generation != self.search_generation {
            return Task::none();
        }
        self.is_searching_global_messages = false;
        self.ensure_search_tab();
        match res {
            Ok(results) => {
                self.global_message_search_results = results;
            }
            Err(e) => {
                self.global_message_search_results.clear();
                self.set_error(crate::fl!("search-server-failed", error = e).to_string());
            }
        }
        self.sync_active_search_results();
        Task::none()
    }

    pub(super) fn handle_set_global_search_scope(
        &mut self,
        scope: crate::matrix::GlobalSearchScope,
    ) -> Task<Action<Message>> {
        self.global_search_scope = scope;
        // Clear stale hits immediately; the re-fired query repopulates.
        self.global_message_search_results.clear();
        // Re-run the current query under the new scope by re-entering
        // the search dispatch. This reuses the debounce so toggling
        // the filter isn't an instant DoS.
        self.handle_update(Message::SearchQueryChanged(self.search_query.clone()))
    }
}
