use super::{AuthFlow, Constellation, ListSelection, MenuAct, Message, SettingsPanel};
use crate::constellation::keybind;
use crate::matrix;
use crate::settings;
use crate::utils::widget::tooltip_button_at;

use cosmic::iced::{Alignment, Subscription};
use cosmic::widget::icon::Named;
use cosmic::widget::tooltip::Position;
use cosmic::widget::{
    Column, RcElementWrapper, Row, button, container, icon, menu, nav_bar, text, text_input,
};
use cosmic::{Action, Application, Core, Element, Task};
use eyeball_im::Vector;
use std::collections::HashMap;

impl Application for Constellation {
    type Executor = cosmic::executor::Default;
    type Message = Message;
    type Flags = Option<String>;
    const APP_ID: &'static str = "fi.joonastuomi.Constellation";

    fn core(&self) -> &Core {
        &self.core
    }

    fn core_mut(&mut self) -> &mut Core {
        &mut self.core
    }

    fn header_start(&self) -> Vec<Element<'_, Self::Message>> {
        let mut start = Vec::new();

        if self.user_id.is_none() {
            return start;
        }
        self.search_bar(&mut start);
        start.push(crate::view::switcher::view_menu_create().into());

        start
    }

    fn header_end(&self) -> Vec<Element<'_, Self::Message>> {
        let mut end = Vec::new();

        if self.user_id.is_some() {
            let user_btn = button::icon(Named::new("user-available-symbolic"));
            let user_tooltip =
                tooltip_button_at(user_btn, crate::fl!("user-menu"), Position::Bottom);
            let key_binds = std::collections::HashMap::new();

            let menu_tree = menu::Tree::with_children(
                RcElementWrapper::new(user_tooltip),
                menu::items(
                    &key_binds,
                    vec![
                        menu::Item::Button(
                            crate::fl!("open-link"),
                            Some(cosmic::widget::icon::Handle::from(
                                cosmic::widget::icon::Named::new("web-browser-symbolic"),
                            )),
                            MenuAct::OpenLink,
                        ),
                        menu::Item::Button(
                            crate::fl!("app-settings"),
                            Some(cosmic::widget::icon::Handle::from(
                                cosmic::widget::icon::Named::new("applications-system"),
                            )),
                            MenuAct::AppSettings,
                        ),
                        menu::Item::Button(
                            crate::fl!("user-settings"),
                            Some(cosmic::widget::icon::Handle::from(
                                cosmic::widget::icon::Named::new("preferences-system-and-accounts"),
                            )),
                            MenuAct::UserSettings,
                        ),
                        menu::Item::Button(
                            crate::fl!("logout"),
                            Some(cosmic::widget::icon::Handle::from(
                                cosmic::widget::icon::Named::new("system-log-out"),
                            )),
                            MenuAct::Logout,
                        ),
                    ],
                ),
            );

            let user_menu = menu::bar(vec![menu_tree])
                .item_height(menu::ItemHeight::Dynamic(40))
                .item_width(menu::ItemWidth::Uniform(160))
                .spacing(4.0);

            end.push(user_menu.into());
        }

        end
    }

    fn nav_model(&self) -> Option<&nav_bar::Model> {
        self.user_id.as_ref()?;
        Some(&self.space_nav_model)
    }

    fn on_nav_select(&mut self, id: nav_bar::Id) -> Task<Action<Self::Message>> {
        self.space_nav_model.activate(id);
        let space_id = self
            .space_nav_model
            .data::<std::sync::Arc<str>>(id)
            .cloned();
        self.handle_select_space(space_id)
    }

    fn init(core: Core, flags: Self::Flags) -> (Self, Task<Action<Self::Message>>) {
        let data_dir = dirs::data_dir().map(|d| d.join("fi.joonastuomi.Constellation"));

        let mut tasks = Vec::new();
        tasks.push(task_create_matrix_engine(data_dir));

        if let Some(uri) = flags {
            // Classify the launch URI the same way the IPC subscription does,
            // so an OIDC callback completes login while a Matrix permalink
            // opens the right room/event.
            tasks.push(Task::done(Action::from(
                crate::constellation::subscriptions::classify_ipc_uri(&uri),
            )));
        }

        let config = settings::config::Config::load();

        let mut app = app(core, config);

        let title_task = app.update_title();
        tasks.push(title_task);

        (app, Task::batch(tasks))
    }

    fn context_drawer(
        &self,
    ) -> Option<cosmic::app::context_drawer::ContextDrawer<'_, Self::Message>> {
        if let Some(panel) = &self.current_settings_panel {
            let title = match panel {
                SettingsPanel::App => crate::fl!("app-settings"),
                SettingsPanel::User => crate::fl!("user-settings"),
                SettingsPanel::Room => crate::fl!("room-settings"),
                SettingsPanel::Permissions => crate::fl!("permissions"),
                SettingsPanel::Space => crate::fl!("space-settings"),
                SettingsPanel::Members => crate::fl!("room-members"),
                SettingsPanel::Pinned => crate::fl!("pinned-messages"),
                SettingsPanel::ActiveThreads => crate::fl!("active-threads"),
                SettingsPanel::ManageRoomMembers => crate::fl!("manage-members"),
                SettingsPanel::ManageSpaceRooms => crate::fl!("manage-spaces-users"),
                SettingsPanel::Shortcuts => crate::fl!("shortcuts-title"),
            };

            let panel_content = match panel {
                SettingsPanel::User => self.user_settings.view().map(Message::UserSettings),
                SettingsPanel::Room => self.room_settings.view().map(Message::RoomSettings),
                SettingsPanel::Permissions => self
                    .room_settings
                    .view_permissions_page()
                    .map(Message::RoomSettings),
                SettingsPanel::Space => self.space_settings.view().map(Message::SpaceSettings),
                SettingsPanel::App => self.app_settings.view().map(Message::AppSettings),
                SettingsPanel::Shortcuts => self.shortcuts.view().map(Message::Shortcuts),
                SettingsPanel::Members => self.view_members_panel(),
                SettingsPanel::Pinned => self.view_pinned_panel(),
                SettingsPanel::ActiveThreads => self.view_active_threads_panel(),
                SettingsPanel::ManageRoomMembers => {
                    self.room_settings.view_manage().map(Message::RoomSettings)
                }
                SettingsPanel::ManageSpaceRooms => self
                    .space_settings
                    .view_manage()
                    .map(Message::SpaceSettings),
            };

            Some(
                cosmic::app::context_drawer::context_drawer(panel_content, Message::CloseSettings)
                    .title(title.to_string()),
            )
        } else if self.creating_room || self.creating_space {
            let title = if self.creating_room {
                crate::fl!("create-room")
            } else {
                crate::fl!("create-space")
            };
            let close_msg = if self.creating_room {
                Message::ToggleCreateRoom
            } else {
                Message::ToggleCreateSpace
            };
            Some(
                cosmic::app::context_drawer::context_drawer(self.view_create_form(), close_msg)
                    .title(title.to_string()),
            )
        } else if self.open_link_dialog.is_some() {
            Some(
                cosmic::app::context_drawer::context_drawer(
                    self.view_open_link_form(),
                    Message::ToggleOpenLink,
                )
                .title(crate::fl!("open-link-dialog-title").to_string()),
            )
        } else {
            None
        }
    }

    fn update(&mut self, message: Message) -> Task<Action<Self::Message>> {
        self.handle_update(message)
    }

    fn view(&self) -> Element<'_, Message> {
        self.view_app()
    }

    fn subscription(&self) -> Subscription<Self::Message> {
        let ipc_sub = self.ipc_subscription();

        let mut subs = vec![
            ipc_sub,
            // Drives deferred scroll restores; fires without user input.
            cosmic::iced::time::every(std::time::Duration::from_millis(50))
                .map(|_| crate::Message::RestoreTick),
        ];

        // While a keybind recording dialog is open, run only the capture
        // subscription so the recorded keys don't also fire live shortcuts.
        if self.shortcuts.recording.is_some() {
            subs.push(keybind::capture_subscription());
        } else if self.user_id.is_some() {
            let mode = match &self.list_selection {
                Some(ListSelection::Rooms { .. }) => keybind::SelectionMode::Rooms,
                Some(ListSelection::Spaces { .. }) => keybind::SelectionMode::Spaces,
                None => keybind::SelectionMode::None,
            };
            subs.push(keybind::subscription(&self.keybinds, mode));
        }

        let matrix = match &self.matrix {
            Some(m) => m,
            None => return Subscription::batch(subs),
        };

        let sync_sub = self.sync_subscription(matrix);

        subs.push(sync_sub);

        if let Some(room_id) = self.selected_room.clone() {
            if let Some(event_id) = self.active_event_focus.clone() {
                // Viewing a permalink context: feed the timeline from the
                // event-focused subscription instead of the live one.
                subs.push(self.event_timeline_subscription(matrix, room_id, event_id));
            } else {
                subs.push(self.timeline_subscription(matrix, room_id));
            }
        }

        if let (Some(room_id), Some(root_id)) =
            (self.selected_room.clone(), self.active_thread_root.clone())
        {
            subs.push(self.threaded_timeline_subscription(matrix, room_id, root_id));
        }
        // Subscribe to thread info updates for open thread tabs.
        for tab in &self.open_tabs {
            if let crate::constellation::Tab::Thread { room_id, root_id } = tab {
                subs.push(self.thread_info_subscription(matrix, room_id.clone(), root_id.clone()));
            }
        }

        // Also subscribe for threads shown in the active threads panel.
        if self.show_active_threads_panel
            && let Some(room_id) = &self.selected_room
        {
            for item in &self.active_threads {
                if let Ok(root_id) = matrix_sdk::ruma::EventId::parse(&item.event_id) {
                    subs.push(self.thread_info_subscription(matrix, room_id.clone(), root_id));
                }
            }
        }

        Subscription::batch(subs)
    }
}

impl Constellation {
    pub fn matrix(&self) -> Option<&matrix::MatrixEngine> {
        self.matrix.as_ref()
    }

    pub fn selected_room(&self) -> Option<&std::sync::Arc<str>> {
        self.selected_room.as_ref()
    }

    pub fn open_tabs(&self) -> &[super::Tab] {
        &self.open_tabs
    }

    pub fn user_id(&self) -> Option<&str> {
        self.user_id.as_deref()
    }

    pub fn search_query(&self) -> &str {
        &self.search_query
    }

    pub fn room_list(&self) -> &[matrix::RoomData] {
        &self.room_list
    }

    pub fn set_rooms_for_test(&mut self, rooms: Vec<matrix::RoomData>) {
        self.room_index.clear();
        for (i, room) in rooms.iter().enumerate() {
            self.room_index.insert(room.id.clone(), i);
        }
        self.room_list = rooms;
    }

    pub fn set_user_id_for_test(&mut self, user_id: Option<String>) {
        self.user_id = user_id;
    }

    pub fn set_selected_room_for_test(&mut self, room_id: Option<std::sync::Arc<str>>) {
        if let Some(id) = &room_id
            && !self.open_tabs.iter().any(|t| t.room_id() == Some(id))
        {
            self.open_tabs.push(super::Tab::Room(id.clone()));
        }
        self.selected_room = room_id;
    }

    pub(crate) fn search_bar<'header>(&'header self, start: &mut Vec<Element<'header, Message>>) {
        if self.is_search_active {
            let search_btn =
                button::icon(Named::new("window-close-symbolic")).on_press(Message::ToggleSearch);
            let search_tooltip =
                tooltip_button_at(search_btn, crate::fl!("close-search"), Position::Bottom);
            let input = text_input(crate::fl!("search-placeholder"), &self.search_query)
                .id(crate::SEARCH_INPUT_ID.clone())
                .on_input(Message::SearchQueryChanged)
                .on_submit(|_| Message::SubmitSearch)
                .width(200.0);

            let input_elem: Element<'header, Message> =
                if self.show_search_suggestions && !self.search_suggestions.is_empty() {
                    let mut col = Column::new()
                        .spacing(2)
                        .padding(4)
                        .width(cosmic::iced::Length::Fixed(240.0));

                    for item in &self.search_suggestions {
                        let icon_name = if item.is_room {
                            "chat-symbolic"
                        } else {
                            "avatar-default-symbolic"
                        };
                        let icon_elem = icon::icon(Named::new(icon_name).into()).size(16);

                        let mut text_col = Column::new().spacing(1);
                        text_col = text_col.push(text::body(&item.display_text).size(13));
                        if let Some(sec) = &item.secondary_text {
                            text_col = text_col.push(text::caption(sec));
                        }

                        let row_elem = Row::new()
                            .spacing(8)
                            .align_y(Alignment::Center)
                            .push(icon_elem)
                            .push(text_col);

                        let btn = button::custom(row_elem)
                            .class(cosmic::theme::Button::Text)
                            .on_press(Message::SearchApplySuggestion(item.replacement.clone()))
                            .width(cosmic::iced::Length::Fill);

                        col = col.push(btn);
                    }

                    let popup_container = container(col).class(cosmic::theme::Container::Dropdown);

                    cosmic::widget::popover(input)
                        .popup(popup_container)
                        .position(cosmic::widget::popover::Position::Bottom)
                        .on_close(Message::SearchDismissSuggestions)
                        .into()
                } else {
                    input.into()
                };

            let mut row = Row::new()
                .align_y(Alignment::Center)
                .push(search_tooltip)
                .push(input_elem);
            if !self.search_query.trim().is_empty() {
                let submit_btn =
                    button::icon(Named::new("edit-find-symbolic")).on_press(Message::SubmitSearch);
                let submit_tooltip =
                    tooltip_button_at(submit_btn, crate::fl!("search"), Position::Bottom);
                row = row.push(submit_tooltip);
            }
            start.push(row.into());
        } else {
            let search_btn =
                button::icon(Named::new("edit-find-symbolic")).on_press(Message::ToggleSearch);
            let search_tooltip =
                tooltip_button_at(search_btn, crate::fl!("search"), Position::Bottom);
            start.push(search_tooltip);
        }
    }
}

pub fn app(core: Core, config: settings::config::Config) -> Constellation {
    let keybinds = keybind::Bindings::with_overrides(&config.key_bindings);
    let mut shortcuts = settings::shortcuts::State::from_bindings(&keybinds);
    shortcuts.overrides = config.key_bindings.clone();

    Constellation {
        core: core.clone(),
        matrix: None,
        sync_status: matrix::SyncStatus::Disconnected,
        room_list: Vec::new(),
        room_index: std::collections::HashMap::new(),
        filtered_room_list: Vec::new(),
        other_rooms: Vec::new(),
        filtered_other_rooms: Vec::new(),
        selected_room: None,
        open_tabs: Vec::new(),
        active_search: None,
        search_results: HashMap::new(),
        tab_model: cosmic::widget::segmented_button::SingleSelectModel::default(),
        pending_link: None,
        pending_oidc_callback: None,
        pending_event_focus: None,
        active_event_focus: None,
        open_link_dialog: None,
        pending_alias_op: None,
        timeline_items: Vector::new(),
        composer_content: cosmic::widget::text_editor::Content::new(),
        composer_preview_events: Vec::new(),
        composer_preview_links: Vec::new(),
        composer_is_preview: false,
        composer_attachments: Vec::new(),
        user_id: None,
        media_cache: HashMap::new(),
        og_cache: HashMap::new(),
        #[cfg(feature = "video-player")]
        video_cache: HashMap::new(),
        #[cfg(feature = "video-player")]
        loading_videos: std::collections::HashSet::new(),
        creating_room: false,
        creating_space: false,
        new_room_name: String::new(),
        inviting_to_space: false,
        invite_to_space_id: String::new(),
        inviting_to_room: false,
        invite_to_room_id: String::new(),
        session_verification_prompt: None,
        identity_violations: Vec::new(),
        error: None,
        error_autoclose_deadline: None,
        login_homeserver: "https://matrix.org".to_string(),
        login_username: String::new(),
        login_password: String::new(),
        auth_flow: AuthFlow::Idle,
        qr_code_bytes: None,
        qr_check_code_sender: None,
        qr_user_code: None,
        qr_check_code_input: String::new(),
        is_registering_mode: false,
        is_registering: false,
        is_initializing: true,
        is_sync_indicator_active: false,
        is_loading_more: false,
        last_timeline_offset: 0.0,
        last_threaded_timeline_offset: 0.0,
        search_query: String::new(),
        is_search_active: false,
        search_suggestions: Vec::new(),
        show_search_suggestions: false,
        public_search_results: Vec::new(),
        is_searching_public: false,
        message_search_results: Vec::new(),
        is_searching_messages: false,
        search_has_more: false,
        is_searching_more_messages: false,
        search_generation: 0,
        global_message_search_results: Vec::new(),
        is_searching_global_messages: false,
        global_search_scope: matrix::GlobalSearchScope::All,
        new_room_is_video: false,
        active_reaction_picker: None,
        active_thread_root: None,
        threaded_timeline_items: Vector::new(),
        joined_room_ids: std::collections::HashSet::new(),
        visited_room_ids: std::collections::HashSet::new(),
        is_first_time_joining: false,
        needs_initial_scroll: false,
        needs_scroll_restoration: false,
        needs_threaded_scroll_restoration: false,
        is_timeline_at_bottom: true,
        is_threaded_timeline_at_bottom: true,
        is_timeline_initialized: false,
        is_threaded_timeline_initialized: false,
        last_content_height: 0.0,
        last_threaded_content_height: 0.0,
        last_viewport_width: 0.0,
        last_viewport_height: 0.0,
        last_threaded_viewport_width: 0.0,
        last_threaded_viewport_height: 0.0,
        needs_layout_scroll_restoration: false,
        needs_threaded_layout_scroll_restoration: false,
        needs_scroll_adjustment: false,
        needs_threaded_scroll_adjustment: false,
        scroll_main: Default::default(),
        scroll_thread: Default::default(),
        room_scroll_memory: HashMap::new(),
        pending_room_restore: None,
        scroll_generation: 0,
        replying_to: None,
        editing_item: None,
        is_room_list_open: true,
        selected_space: None,
        space_nav_model: cosmic::widget::nav_bar::Model::default(),
        space_nav_fingerprint: None,
        space_nav_dirty: false,
        current_settings_panel: None,
        user_settings: settings::user::State::from_config(&config),
        room_settings: Default::default(),
        space_settings: Default::default(),
        app_settings: settings::app::State::from_config(&config),
        call_participants: HashMap::new(),
        fullscreen_image: None,
        emoji_search_query: String::new(),
        selected_emoji_group: None,
        is_composer_emoji_picker_active: false,
        emoji_picker_tab: Default::default(),
        room_image_packs: std::collections::HashMap::new(),
        user_image_packs: Vec::new(),
        global_pack_rooms: std::collections::BTreeMap::new(),
        active_custom_emojis: Vec::new(),
        active_stickers: Vec::new(),
        room_name_cache: std::collections::HashMap::new(),
        thread_counts: std::collections::HashMap::new(),
        event_id_to_index: std::collections::HashMap::new(),
        thread_root_to_last_index: std::collections::HashMap::new(),
        show_pinned_panel: false,
        is_loading_pinned: false,
        pinned_events: std::collections::HashSet::new(),
        pinned_events_details: Vec::new(),
        show_members_panel: false,
        room_members: Vec::new(),
        is_loading_members: false,
        show_active_threads_panel: false,
        is_loading_active_threads: false,
        active_threads: Vec::new(),
        thread_unreads: std::collections::HashMap::new(),
        panes: crate::constellation::create_main_panes(config.sidebar_ratio),
        sidebar_ratio: if config.sidebar_ratio.is_finite()
            && (0.10..=0.85).contains(&config.sidebar_ratio)
        {
            config.sidebar_ratio
        } else {
            crate::constellation::DEFAULT_SIDEBAR_RATIO
        },
        keybinds,
        shortcuts,
        list_selection: None,
    }
}

fn task_create_matrix_engine(data_dir: Option<std::path::PathBuf>) -> Task<Action<Message>> {
    Task::perform(
        async move {
            let dir = data_dir.ok_or_else(|| {
                matrix::SyncError::from(anyhow::anyhow!("No standard data directory found"))
            })?;
            matrix::MatrixEngine::new(dir)
                .await
                .map_err(matrix::SyncError::from)
        },
        |res| Action::from(Message::EngineReady(res)),
    )
}
