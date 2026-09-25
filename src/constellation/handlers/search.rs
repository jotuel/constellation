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
            self.show_search_suggestions = false;
            self.search_suggestions.clear();
        } else if let Some(panel) = self.settings_stack.last() {
            match panel {
                SettingsPanel::Room => {
                    self.search_query = self.room_settings.member_filter.clone();
                }
                SettingsPanel::Space => {
                    self.search_query = self.space_settings.child_filter.clone();
                }
                _ => {}
            }
        } else if let Some(Tab::Search { query, .. }) = self.active_tab() {
            self.search_query = query.clone();
        }
        self.update_filtered_rooms();
        self.update_title()
    }

    pub(super) fn handle_submit_search(&mut self) -> Task<Action<Message>> {
        let query = self.search_query.trim().to_string();
        if query.is_empty() {
            return Task::none();
        }
        self.show_search_suggestions = false;
        self.search_suggestions.clear();

        let parsed = crate::utils::search_query::parse_search_query(&query);
        if let Some(scope) = parsed.scope {
            self.global_search_scope = scope;
        }

        let target_room_id = if let Some(room_filter) = &parsed.room_filter {
            self.resolve_room_filter(room_filter)
        } else if parsed.scope.is_some() {
            None
        } else if let Some(Tab::Search { room_id, .. }) = &self.active_search {
            room_id.clone()
        } else {
            self.selected_room.clone()
        };

        if let Some(active_search_tab) = self.active_search.clone() {
            let Tab::Search { .. } = active_search_tab.clone() else {
                return Task::none();
            };
            let updated_tab = Tab::Search {
                room_id: target_room_id,
                query: query.clone(),
            };
            if active_search_tab == updated_tab {
                self.update_filtered_rooms();
                return self.start_search_for_tab(updated_tab);
            }

            self.search_results.remove(&active_search_tab);
            if self.open_tabs.contains(&updated_tab) {
                if let Some(pos) = self.open_tabs.iter().position(|t| t == &active_search_tab) {
                    self.open_tabs.remove(pos);
                }
            } else if let Some(pos) = self.open_tabs.iter().position(|t| t == &active_search_tab) {
                self.open_tabs[pos] = updated_tab.clone();
            } else {
                self.open_tabs.push(updated_tab.clone());
            }

            self.active_search = None;
            self.rebuild_tab_model();
            let activate_task = self.activate_tab(updated_tab.clone());
            let search_task = self.start_search_for_tab(updated_tab);
            return Task::batch(vec![activate_task, search_task]);
        }

        let tab = Tab::Search {
            room_id: target_room_id,
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
            let parsed = crate::utils::search_query::parse_search_query(query);
            let public_query = if !parsed.sanitized_query.is_empty() {
                parsed.sanitized_query.clone()
            } else {
                query.trim().to_string()
            };
            let matrix_public = matrix.clone();
            self.is_searching_public = true;

            tasks.push(Task::perform(
                async move {
                    matrix_public
                        .search_public_rooms(public_query, Some(20))
                        .await
                },
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
                let scope = parsed.scope.unwrap_or(self.global_search_scope);
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
        if let Some(panel) = self.settings_stack.last() {
            match panel {
                SettingsPanel::Room => {
                    self.room_settings.member_filter = query;
                }
                SettingsPanel::Space => {
                    self.space_settings.child_filter = query;
                }
                _ => {}
            }
        }
        self.update_search_suggestions();
        self.update_filtered_rooms();
        Task::none()
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
        if let Some(active_search_tab) = self.active_search.clone() {
            self.start_search_for_tab(active_search_tab)
        } else {
            Task::none()
        }
    }

    /// Resolves a user-typed room name, alias, or ID against joined and known rooms.
    pub(crate) fn resolve_room_filter(&self, room_filter: &str) -> Option<std::sync::Arc<str>> {
        let trimmed = room_filter.trim();
        if trimmed.is_empty() {
            return None;
        }

        // 1. Full Matrix room ID format (!...:...)
        if trimmed.starts_with('!') && trimmed.contains(':') {
            return Some(std::sync::Arc::from(trimmed));
        }

        let needle = trimmed.trim_start_matches(['#', '!']).trim();

        // 2. Exact match (case-insensitive) on room name or id
        for room in self.room_list.iter().chain(self.other_rooms.iter()) {
            if let Some(name) = &room.name
                && name.eq_ignore_ascii_case(needle)
            {
                return Some(room.id.clone());
            }
            if room.id.as_ref().eq_ignore_ascii_case(trimmed) {
                return Some(room.id.clone());
            }
        }

        // 3. Prefix match on room name
        let needle_lower = needle.to_lowercase();
        for room in self.room_list.iter().chain(self.other_rooms.iter()) {
            if let Some(name) = &room.name
                && name.to_lowercase().starts_with(&needle_lower)
            {
                return Some(room.id.clone());
            }
        }

        // 4. Substring match on room name
        for room in self.room_list.iter().chain(self.other_rooms.iter()) {
            if let Some(name) = &room.name
                && name.to_lowercase().contains(&needle_lower)
            {
                return Some(room.id.clone());
            }
        }

        if trimmed.starts_with('!') {
            Some(std::sync::Arc::from(trimmed))
        } else {
            None
        }
    }

    /// Updates the active autocomplete suggestions based on the trailing search input token.
    pub(crate) fn update_search_suggestions(&mut self) {
        use crate::constellation::SearchSuggestion;
        use crate::utils::search_query::{AutocompleteTrigger, extract_autocomplete_trigger};

        self.search_suggestions.clear();

        let Some(trigger) = extract_autocomplete_trigger(&self.search_query) else {
            self.show_search_suggestions = false;
            return;
        };

        match trigger {
            AutocompleteTrigger::Room { needle } => {
                let needle_lower = needle.to_lowercase();
                for room in self.room_list.iter().chain(self.other_rooms.iter()) {
                    let matches = if needle.is_empty() {
                        true
                    } else {
                        let name_match = room
                            .name
                            .as_deref()
                            .is_some_and(|n| n.to_lowercase().contains(&needle_lower));
                        let id_match = room.id.to_lowercase().contains(&needle_lower);
                        name_match || id_match
                    };

                    if matches {
                        let display_text = room.name.clone().unwrap_or_else(|| room.id.to_string());
                        let secondary_text = if room.name.is_some() {
                            Some(room.id.to_string())
                        } else {
                            None
                        };
                        let replacement = if let Some(name) = &room.name {
                            if name.contains(char::is_whitespace) {
                                format!("#\"{name}\" ")
                            } else {
                                format!("#{name} ")
                            }
                        } else {
                            format!("#{} ", room.id)
                        };

                        self.search_suggestions.push(SearchSuggestion {
                            display_text,
                            secondary_text,
                            replacement,
                            is_room: true,
                        });

                        if self.search_suggestions.len() >= 8 {
                            break;
                        }
                    }
                }
            }
            AutocompleteTrigger::Member { needle } => {
                let needle_lower = needle.to_lowercase();
                for member in &self.room_members {
                    let matches = if needle.is_empty() {
                        true
                    } else {
                        let name_match = member
                            .display_name
                            .as_deref()
                            .is_some_and(|d| d.to_lowercase().contains(&needle_lower));
                        let id_match = member.user_id.to_lowercase().contains(&needle_lower);
                        name_match || id_match
                    };

                    if matches {
                        let display_text = member
                            .display_name
                            .clone()
                            .unwrap_or_else(|| member.user_id.clone());
                        let secondary_text = Some(member.user_id.clone());
                        let clean_id = member.user_id.trim_start_matches('@');
                        let replacement = format!("@{clean_id} ");

                        self.search_suggestions.push(SearchSuggestion {
                            display_text,
                            secondary_text,
                            replacement,
                            is_room: false,
                        });

                        if self.search_suggestions.len() >= 8 {
                            break;
                        }
                    }
                }
            }
        }

        self.show_search_suggestions = !self.search_suggestions.is_empty();
    }

    pub(super) fn handle_search_apply_suggestion(
        &mut self,
        replacement: String,
    ) -> Task<Action<Message>> {
        use crate::utils::search_query::apply_autocomplete_replacement;
        self.search_query = apply_autocomplete_replacement(&self.search_query, &replacement);
        self.show_search_suggestions = false;
        self.search_suggestions.clear();
        self.update_filtered_rooms();
        cosmic::widget::text_input::focus(crate::SEARCH_INPUT_ID.clone())
    }

    pub(super) fn handle_search_dismiss_suggestions(&mut self) -> Task<Action<Message>> {
        self.show_search_suggestions = false;
        self.search_suggestions.clear();
        Task::none()
    }
}
