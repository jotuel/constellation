//! Handlers for global keyboard shortcuts and keyboard list-selection.

use crate::constellation::ListSelection;
use crate::constellation::keybind::{Bindings, ShortcutAction};
use crate::{Constellation, Message, THREADED_TIMELINE_ID, TIMELINE_ID};
use cosmic::iced::widget::scrollable;
use cosmic::widget::text_input;
use cosmic::{Action, Task};
use std::collections::HashSet;
use std::sync::Arc;

impl Constellation {
    /// Dispatch a triggered global shortcut.
    pub(super) fn handle_shortcut_triggered(
        &mut self,
        action: ShortcutAction,
    ) -> Task<Action<Message>> {
        // Shortcuts are inert on the login screen except quitting.
        if self.user_id.is_none() && action != ShortcutAction::Quit {
            return Task::none();
        }

        match action {
            ShortcutAction::ToggleAppSettings => {
                self.handle_toggle_settings_panel(crate::SettingsPanel::App)
            }
            ShortcutAction::ToggleUserSettings => {
                self.handle_toggle_settings_panel(crate::SettingsPanel::User)
            }
            ShortcutAction::ToggleRoomSettings => {
                self.handle_toggle_settings_panel(crate::SettingsPanel::Room)
            }
            ShortcutAction::ToggleSpaceSettings => {
                self.handle_toggle_settings_panel(crate::SettingsPanel::Space)
            }
            ShortcutAction::ToggleSpaceSwitcher => {
                // The space switcher is the libcosmic navigation bar listing
                // joined spaces; toggling shows/hides it.
                self.core.nav_bar_toggle();
                Task::none()
            }
            ShortcutAction::CloseThread => {
                if self.active_thread_root.is_some() {
                    self.handle_close_thread()
                } else {
                    Task::none()
                }
            }
            ShortcutAction::Quit => cosmic::iced::exit(),
            ShortcutAction::Search => self.handle_search_shortcut(),
            // Copy is performed natively by the focused text widget; the
            // subscription's Ignored gate guarantees we only get here when no
            // text area holds the key, i.e. there is nothing to copy.
            ShortcutAction::CopyText => Task::none(),
            ShortcutAction::SelectRoomList => self.enter_room_selection(),
            ShortcutAction::SelectSpaceSwitcher => self.enter_space_selection(),
            ShortcutAction::ScrollUp => self.handle_scroll_shortcut(true),
            ShortcutAction::ScrollDown => self.handle_scroll_shortcut(false),
        }
    }

    pub(super) fn handle_toggle_settings_panel(
        &mut self,
        panel: crate::SettingsPanel,
    ) -> Task<Action<Message>> {
        if self.current_settings_panel.as_ref() == Some(&panel) {
            self.handle_close_settings()
        } else {
            self.handle_open_settings(panel)
        }
    }

    /// Close the context drawer; shared by `Message::CloseSettings` and the
    /// "Toggle … Settings" shortcuts.
    pub(super) fn handle_close_settings(&mut self) -> Task<Action<Message>> {
        self.needs_layout_scroll_restoration = true;
        self.needs_threaded_layout_scroll_restoration = true;
        self.current_settings_panel = None;
        self.core.set_show_context(false);
        self.show_members_panel = false;
        self.show_pinned_panel = false;
        self.room_members.clear();
        self.pinned_events_details.clear();
        self.restore_scroll_task()
    }

    /// Ctrl+F / Search: open the header search bar when closed, then move
    /// input focus to its field.
    fn handle_search_shortcut(&mut self) -> Task<Action<Message>> {
        if !self.is_search_active {
            // Flip the bar open through the normal path (clears stale query
            // state) so only the focus task remains for the runtime.
            let _ = self.handle_toggle_search();
        }
        text_input::focus(crate::SEARCH_INPUT_ID.clone())
    }

    /// PageUp/PageDown: page the visible timeline by ~90% of its viewport.
    fn handle_scroll_shortcut(&self, up: bool) -> Task<Action<Message>> {
        let (id, offset, viewport_height) = if self.active_thread_root.is_some() {
            (
                THREADED_TIMELINE_ID.clone(),
                self.last_threaded_timeline_offset,
                self.last_threaded_viewport_height,
            )
        } else {
            (
                TIMELINE_ID.clone(),
                self.last_timeline_offset,
                self.last_viewport_height,
            )
        };

        let page = if viewport_height > 1.0 {
            viewport_height * 0.9
        } else {
            400.0
        };
        let y = if up {
            (offset - page).max(0.0)
        } else {
            offset + page
        };
        scrollable::scroll_to(
            id,
            scrollable::AbsoluteOffset {
                x: Some(0.0),
                y: Some(y),
            },
        )
    }

    /// "Selection to Rooms": start keyboard-selection over the sidebar's room
    /// list, starting from the currently selected room.
    fn enter_room_selection(&mut self) -> Task<Action<Message>> {
        if matches!(self.list_selection, Some(ListSelection::Rooms { .. })) {
            return Task::none();
        }
        let ids = self.selectable_sidebar_rooms();
        if ids.is_empty() {
            return Task::none();
        }
        let index = self
            .selected_room
            .as_ref()
            .and_then(|selected| ids.iter().position(|id| id == selected))
            .unwrap_or(0);
        self.list_selection = Some(ListSelection::Rooms { ids, index });
        Task::none()
    }

    /// "Selection to Space Switcher": start keyboard-selection over the space
    /// nav bar, starting from the active entry.
    fn enter_space_selection(&mut self) -> Task<Action<Message>> {
        if matches!(self.list_selection, Some(ListSelection::Spaces { .. })) {
            return Task::none();
        }
        let count = self.space_nav_model.len();
        if count == 0 {
            return Task::none();
        }
        let position = self
            .space_nav_model
            .active_data::<Arc<str>>()
            .and_then(|id| {
                self.space_nav_model
                    .iter()
                    .position(|entity| self.space_nav_model.data::<Arc<str>>(entity) == Some(id))
            });
        let position = position.map(|p| p as u16).unwrap_or(0);
        self.list_selection = Some(ListSelection::Spaces { position });
        // Visual feedback: highlight the entry without switching spaces.
        self.space_nav_model.activate_position(position);
        Task::none()
    }

    /// Arrow/Home/End navigation within an active list selection.
    pub(super) fn handle_selection_move(&mut self, delta: i32) -> Task<Action<Message>> {
        match &mut self.list_selection {
            Some(ListSelection::Rooms { ids, index }) => {
                let len = ids.len();
                let new_index = if delta == i32::MIN {
                    0
                } else if delta == i32::MAX {
                    len - 1
                } else {
                    (*index as i64 + delta as i64).clamp(0, len as i64 - 1) as usize
                };
                *index = new_index;
                Task::none()
            }
            Some(ListSelection::Spaces { position }) => {
                let max = self.space_nav_model.len().saturating_sub(1) as u16;
                let new_position = if delta == i32::MIN {
                    0
                } else if delta == i32::MAX {
                    max
                } else {
                    (*position as i64 + delta as i64).clamp(0, max as i64) as u16
                };
                *position = new_position;
                self.space_nav_model.activate_position(new_position);
                Task::none()
            }
            None => Task::none(),
        }
    }

    /// Enter: activate the keyboard-selected entry.
    pub(super) fn handle_selection_commit(&mut self) -> Task<Action<Message>> {
        match self.list_selection.take() {
            Some(ListSelection::Rooms { ids, index }) => match ids.get(index).cloned() {
                Some(id) => self.handle_room_selected(id),
                None => Task::none(),
            },
            Some(ListSelection::Spaces { position }) => {
                let space_id = self
                    .space_nav_model
                    .entity_at(position)
                    .and_then(|entity| self.space_nav_model.data::<Arc<str>>(entity).cloned());
                self.sync_space_nav_activation();
                self.handle_select_space(space_id)
            }
            None => Task::none(),
        }
    }

    /// Escape: leave selection mode and restore the real active nav entry.
    pub(super) fn handle_selection_cancel(&mut self) -> Task<Action<Message>> {
        if self.list_selection.take().is_some() {
            self.sync_space_nav_activation();
        }
        Task::none()
    }

    /// Commit the shortcuts page draft: persist overrides into config and
    /// rebuild the live bindings so rebinds apply without a restart.
    pub(super) fn handle_shortcuts_saved(&mut self) -> Task<Action<Message>> {
        let overrides = self.shortcuts.compute_overrides();
        self.shortcuts.overrides = overrides.clone();
        self.keybinds = Bindings::with_overrides(&overrides);
        self.shortcuts.dirty = false;

        let config = self.build_config();
        Task::perform(async move { config.save() }, |_| {
            Action::from(Message::NoOp)
        })
    }

    /// Close an open thread and restore the main timeline scroll position.
    pub(super) fn handle_close_thread(&mut self) -> Task<Action<Message>> {
        self.needs_layout_scroll_restoration = true;
        self.active_thread_root = None;
        self.threaded_timeline_items.clear();
        self.last_threaded_timeline_offset = 0.0;
        self.last_threaded_content_height = 0.0;
        self.last_threaded_viewport_width = 0.0;
        self.last_threaded_viewport_height = 0.0;
        self.needs_threaded_scroll_adjustment = false;
        self.is_threaded_timeline_initialized = false;
        self.restore_scroll_task()
    }

    /// Room ids displayed as subspaces under the selected space, in sidebar
    /// display order. Shared by the sidebar view and room-list selection.
    pub(crate) fn sidebar_subspaces(&self) -> Vec<Arc<str>> {
        let mut out = Vec::new();
        if let Some(selected_space) = &self.selected_space
            && let Some(matrix) = &self.matrix
            && let Ok(selected_space_id) = matrix_sdk::ruma::RoomId::parse(selected_space.as_str())
        {
            for room in &self.room_list {
                if room.is_space
                    && room.id.as_ref() != selected_space.as_str()
                    && let Ok(room_id) = matrix_sdk::ruma::RoomId::parse(room.id.as_ref())
                    && matrix.is_in_space_sync(&room_id, &selected_space_id)
                {
                    out.push(room.id.clone());
                }
            }
        }
        out
    }

    /// Every selectable sidebar room id in visual order: subspaces first (when
    /// a space is selected), then filtered rooms, then suggested others, then
    /// remaining others.
    pub(crate) fn selectable_sidebar_rooms(&self) -> Vec<Arc<str>> {
        let subspace_ids: HashSet<Arc<str>> = self.sidebar_subspaces().into_iter().collect();

        let mut ids: Vec<Arc<str>> = self
            .filtered_room_list
            .iter()
            .map(|&idx| self.room_list[idx].id.clone())
            .filter(|id| !subspace_ids.contains(id))
            .collect();

        for &idx in &self.filtered_other_rooms {
            if self.other_rooms[idx].suggested {
                ids.push(self.other_rooms[idx].id.clone());
            }
        }
        for &idx in &self.filtered_other_rooms {
            if !self.other_rooms[idx].suggested {
                ids.push(self.other_rooms[idx].id.clone());
            }
        }
        ids
    }

    /// The room id currently under keyboard selection, if any. The sidebar
    /// highlights it like a normal selection.
    pub(crate) fn keyboard_selected_room(&self) -> Option<&Arc<str>> {
        match &self.list_selection {
            Some(ListSelection::Rooms { ids, index }) => ids.get(*index),
            _ => None,
        }
    }
}
