use super::Constellation;
use crate::matrix;
use crate::utils::contains_ignore_ascii_case;

fn build_error_notification(body: &str) -> notify_rust::Notification {
    let mut notification = notify_rust::Notification::new();
    notification
        .appname("Constellation")
        .summary("Constellation Error")
        .body(body)
        .icon("dialog-error");
    notification
}

impl Constellation {
    pub fn set_error(&mut self, error: String) {
        tracing::error!("Error occurred: {}", error);
        let error_clone = error.clone();
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            handle.spawn(async move {
                let _ = build_error_notification(&error_clone).show_async().await;
            });
        } else {
            let _ = build_error_notification(&error_clone).show();
        }
        self.error = Some(error);
    }

    pub fn update_filtered_rooms(&mut self) {
        let is_search_empty = self.search_query.is_empty();

        let is_query_ascii = self.search_query.is_ascii();
        let search_query_lower_fallback =
            (!is_query_ascii).then(|| self.search_query.to_lowercase());

        let filter_by_search = |room: &matrix::RoomData| {
            if is_search_empty {
                true
            } else {
                room.name
                    .as_ref()
                    .map(|n| {
                        contains_ignore_ascii_case(
                            n,
                            &self.search_query,
                            search_query_lower_fallback.as_deref(),
                        )
                    })
                    .unwrap_or(false)
                    || contains_ignore_ascii_case(
                        &room.id,
                        &self.search_query,
                        search_query_lower_fallback.as_deref(),
                    )
            }
        };

        if let Some(selected_space) = &self.selected_space {
            if let Some(matrix) = &self.matrix {
                // ⚡ Bolt Optimization: Reuse the existing vector allocation to avoid O(N) allocation on every keystroke
                let mut rooms = std::mem::take(&mut self.filtered_room_list);

                if matrix.filter_in_space_bulk_sync(
                    self.room_list
                        .iter()
                        .enumerate()
                        .filter(|(_, r)| !r.is_space),
                    selected_space,
                    &mut rooms,
                    filter_by_search,
                ) {
                    rooms.sort_by(|&a, &b| {
                        let ra = &self.room_list[a];
                        let rb = &self.room_list[b];
                        match (&ra.order, &rb.order) {
                            (Some(oa), Some(ob)) => oa.cmp(ob).then_with(|| ra.id.cmp(&rb.id)),
                            (Some(_), None) => std::cmp::Ordering::Less,
                            (None, Some(_)) => std::cmp::Ordering::Greater,
                            (None, None) => ra.id.cmp(&rb.id),
                        }
                    });
                    self.filtered_room_list = rooms;
                } else {
                    // If we couldn't get the lock, just return and keep the old list
                    self.filtered_room_list = rooms;
                    return;
                }
            }

            // Re-filter other_rooms to remove any that we've now joined
            self.other_rooms
                .retain(|r| !self.joined_room_ids.contains(r.id.as_ref()));

            let mut filtered_other = std::mem::take(&mut self.filtered_other_rooms);
            filtered_other.clear();
            filtered_other.extend(
                self.other_rooms
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| filter_by_search(r))
                    .map(|(i, _)| i),
            );
            self.filtered_other_rooms = filtered_other;
        } else {
            let mut rooms = std::mem::take(&mut self.filtered_room_list);
            rooms.clear();
            rooms.extend(
                self.room_list
                    .iter()
                    .enumerate()
                    .filter(|(_, r)| !r.is_space && filter_by_search(r))
                    .map(|(i, _)| i),
            );
            rooms.sort_by(|&a, &b| self.room_list[a].id.cmp(&self.room_list[b].id));
            self.filtered_room_list = rooms;
            self.other_rooms.clear();
            self.filtered_other_rooms.clear();
        }
    }

    /// Rebuild `room_index` from the current `room_list`. Call after any
    /// mutation of `room_list` (diff apply, clear).
    pub fn rebuild_room_index(&mut self) {
        self.room_index.clear();
        self.room_index.extend(
            self.room_list
                .iter()
                .enumerate()
                .map(|(i, r)| (r.id.clone(), i)),
        );
    }

    /// O(1) lookup of a room by id via `room_index`.
    pub fn room_by_id(&self, id: &str) -> Option<&matrix::RoomData> {
        self.room_index.get(id).and_then(|&i| self.room_list.get(i))
    }
}

/// Icon size (px) used for space avatars in the nav bar.
const SPACE_NAV_ICON_SIZE: u16 = 24;

impl Constellation {
    /// Rebuild the space nav bar model from `room_list` and `media_cache`.
    ///
    /// Position 0 is the "All rooms" pseudo-entry; each joined space follows
    /// with its name, avatar icon (once loaded) and room id attached as
    /// entity data. Gated by a fingerprint over the visible space data, so
    /// routine sync churn keeps stable entity ids (and the widget's scroll
    /// state); call after anything that changes `room_list` or a space
    /// avatar image.
    pub fn rebuild_space_nav_model(&mut self) {
        use std::hash::{Hash as _, Hasher as _};

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        for room in self.room_list.iter().filter(|r| r.is_space) {
            if matrix_sdk::ruma::RoomId::parse(&*room.id).is_err() {
                continue;
            }
            room.id.hash(&mut hasher);
            room.name.hash(&mut hasher);
            room.avatar_url.hash(&mut hasher);
            room.avatar_url
                .as_ref()
                .is_some_and(|url| self.media_cache.contains_key(url))
                .hash(&mut hasher);
        }
        let fingerprint = hasher.finish();
        if Some(fingerprint) == self.space_nav_fingerprint {
            return;
        }
        self.space_nav_fingerprint = Some(fingerprint);

        let mut model = cosmic::widget::nav_bar::Model::default();
        model.insert().text(crate::fl!("all-rooms")).icon(
            cosmic::widget::icon::Named::new("web-browser")
                .size(SPACE_NAV_ICON_SIZE)
                .icon(),
        );

        for room in self.room_list.iter().filter(|r| r.is_space) {
            if matrix_sdk::ruma::RoomId::parse(&*room.id).is_err() {
                continue;
            }
            let name = room
                .name
                .clone()
                .unwrap_or_else(|| crate::fl!("unknown-space"));
            let icon = match room
                .avatar_url
                .as_ref()
                .and_then(|url| self.media_cache.get(url))
            {
                Some(handle) => cosmic::widget::icon::Handle {
                    symbolic: false,
                    data: cosmic::widget::icon::Data::Image(handle.clone()),
                }
                .icon()
                .size(SPACE_NAV_ICON_SIZE),
                None => cosmic::widget::icon::Named::new("network-workgroup-symbolic")
                    .size(SPACE_NAV_ICON_SIZE)
                    .icon(),
            };
            model.insert().text(name).icon(icon).data(room.id.clone());
        }

        self.space_nav_model = model;
        self.sync_space_nav_activation();
    }

    /// Sync the active nav bar entry with `selected_space` (`None` activates
    /// the "All rooms" entry).
    pub fn sync_space_nav_activation(&mut self) {
        let entities: Vec<_> = self.space_nav_model.iter().collect();
        let mut target = None;
        for entity in entities {
            let space_id = self
                .space_nav_model
                .data::<std::sync::Arc<str>>(entity)
                .cloned();
            let matches = match (self.selected_space.as_deref(), space_id.as_deref()) {
                (None, None) => true,
                (Some(selected), Some(id)) => selected.as_str() == id,
                _ => false,
            };
            if matches {
                target = Some(entity);
                break;
            }
        }
        self.space_nav_model.deactivate();
        if let Some(entity) = target {
            self.space_nav_model.activate(entity);
        }
    }

    /// Whether `url` is the avatar of a joined space, i.e. whether its media
    /// finishing should refresh the space nav bar icons.
    pub fn is_space_avatar_url(&self, url: &str) -> bool {
        self.room_list
            .iter()
            .any(|r| r.is_space && r.avatar_url.as_deref() == Some(url))
    }
}
