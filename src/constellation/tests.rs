use super::subscriptions::get_room_data;
use super::*;
use crate::matrix;
use crate::settings;
use cosmic::Application;
use std::collections::HashMap;

fn create_test_app() -> Constellation {
    Constellation {
        core: cosmic::app::Core::default(),
        matrix: None,
        sync_status: matrix::SyncStatus::Disconnected,
        room_list: Vec::new(),
        room_index: std::collections::HashMap::new(),
        filtered_room_list: Vec::new(),
        other_rooms: Vec::new(),
        filtered_other_rooms: Vec::new(),
        selected_room: None,
        pending_link: None,
        pending_event_focus: None,
        active_event_focus: None,
        open_link_dialog: None,
        pending_alias_op: None,
        timeline_items: eyeball_im::Vector::new(),
        composer_content: cosmic::widget::text_editor::Content::new(),
        composer_preview_events: Vec::new(),
        composer_preview_links: Vec::new(),
        composer_is_preview: false,
        composer_attachments: Vec::new(),
        user_id: None,
        media_cache: std::collections::HashMap::new(),
        og_cache: std::collections::HashMap::new(),
        #[cfg(feature = "video-player")]
        video_cache: std::collections::HashMap::new(),
        #[cfg(feature = "video-player")]
        loading_videos: std::collections::HashSet::new(),
        creating_room: false,
        creating_space: false,
        new_room_name: String::new(),
        inviting_to_space: false,
        invite_to_space_id: String::new(),
        inviting_to_room: false,
        invite_to_room_id: String::new(),
        error: None,
        login_homeserver: String::new(),
        login_username: String::new(),
        login_password: String::new(),
        auth_flow: AuthFlow::Idle,
        qr_code_bytes: None,
        qr_check_code_sender: None,
        qr_user_code: None,
        qr_check_code_input: String::new(),
        is_registering_mode: false,
        is_registering: false,
        is_initializing: false,
        is_sync_indicator_active: false,
        search_query: String::new(),
        is_search_active: false,
        public_search_results: Vec::new(),
        is_searching_messages: false,
        search_has_more: false,
        is_searching_more_messages: false,
        message_search_results: Vec::new(),
        search_generation: 0,
        global_message_search_results: Vec::new(),
        is_searching_global_messages: false,
        global_search_scope: matrix::GlobalSearchScope::All,
        is_searching_public: false,
        new_room_is_video: false,
        active_reaction_picker: None,
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
        selected_space: None,
        space_nav_model: cosmic::widget::nav_bar::Model::default(),
        space_nav_fingerprint: None,
        current_settings_panel: None,
        user_settings: settings::user::State::default(),
        room_settings: settings::room::State::default(),
        space_settings: settings::space::State::default(),
        app_settings: settings::app::State::default(),
        active_thread_root: None,
        threaded_timeline_items: eyeball_im::Vector::new(),
        is_loading_more: false,
        replying_to: None,
        editing_item: None,
        call_participants: HashMap::new(),
        last_timeline_offset: Default::default(),
        last_threaded_timeline_offset: Default::default(),
        fullscreen_image: None,
        emoji_search_query: String::new(),
        selected_emoji_group: None,
        is_composer_emoji_picker_active: false,
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
        panes: create_main_panes(DEFAULT_SIDEBAR_RATIO),
        keybinds: crate::constellation::keybind::Bindings::defaults(),
        shortcuts: crate::settings::shortcuts::State::default(),
        list_selection: None,
        sidebar_ratio: DEFAULT_SIDEBAR_RATIO,
    }
}

#[test]
fn test_update_filtered_rooms_no_search_no_space() {
    let mut app = create_test_app();
    app.room_list = vec![
        matrix::RoomData {
            id: std::sync::Arc::from("!room1:matrix.org"),
            name: Some("Room 1".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            suggested: false,
        },
        matrix::RoomData {
            id: std::sync::Arc::from("!space1:matrix.org"),
            name: Some("Space 1".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: true,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            suggested: false,
        },
    ];

    app.update_filtered_rooms();

    assert_eq!(app.filtered_room_list.len(), 1);
    assert_eq!(
        app.room_list[app.filtered_room_list[0]].id.as_ref(),
        "!room1:matrix.org"
    );
}

#[test]
fn test_update_filtered_rooms_search_by_name() {
    let mut app = create_test_app();
    app.room_list = vec![
        matrix::RoomData {
            id: std::sync::Arc::from("!room1:matrix.org"),
            name: Some("Alpha Room".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            suggested: false,
        },
        matrix::RoomData {
            id: std::sync::Arc::from("!room2:matrix.org"),
            name: Some("Beta Room".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            suggested: false,
        },
    ];

    app.search_query = "alpha".to_string();
    app.update_filtered_rooms();

    assert_eq!(app.filtered_room_list.len(), 1);
    assert_eq!(
        app.room_list[app.filtered_room_list[0]].id.as_ref(),
        "!room1:matrix.org"
    );
}

#[test]
fn test_update_filtered_rooms_search_by_id() {
    let mut app = create_test_app();
    app.room_list = vec![
        matrix::RoomData {
            id: std::sync::Arc::from("!room1:matrix.org"),
            name: Some("Alpha Room".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            suggested: false,
        },
        matrix::RoomData {
            id: std::sync::Arc::from("!room2:matrix.org"),
            name: Some("Beta Room".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            suggested: false,
        },
    ];

    app.search_query = "!ROOM2".to_string();
    app.update_filtered_rooms();

    assert_eq!(app.filtered_room_list.len(), 1);
    assert_eq!(
        app.room_list[app.filtered_room_list[0]].id.as_ref(),
        "!room2:matrix.org"
    );
}

#[test]
fn test_update_filtered_rooms_search_no_match() {
    let mut app = create_test_app();
    app.room_list = vec![matrix::RoomData {
        id: std::sync::Arc::from("!room1:matrix.org"),
        name: Some("Alpha Room".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    }];

    app.search_query = "gamma".to_string();
    app.update_filtered_rooms();

    assert_eq!(app.filtered_room_list.len(), 0);
}

#[test]
fn test_update_filtered_rooms_with_selected_space_no_matrix() {
    let mut app = create_test_app();
    app.room_list = vec![matrix::RoomData {
        id: std::sync::Arc::from("!room1:matrix.org"),
        name: Some("Alpha Room".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    }];

    app.selected_space = Some(matrix_sdk::ruma::RoomId::parse("!space1:matrix.org").unwrap());

    app.update_filtered_rooms();

    assert_eq!(app.filtered_room_list.len(), 0);
}

#[tokio::test]
async fn test_update_filtered_rooms_with_selected_space_and_matrix() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let engine = match matrix::MatrixEngine::new(tmp_dir.path().to_path_buf()).await {
        Ok(e) => e,
        Err(_) => return, // Skip if engine fails to init (e.g. no dbus)
    };

    let mut app = create_test_app();
    app.matrix = Some(engine);

    app.room_list = vec![matrix::RoomData {
        id: std::sync::Arc::from("!room1:matrix.org"),
        name: Some("Alpha Room".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    }];

    app.other_rooms = vec![
        matrix::RoomData {
            id: std::sync::Arc::from("!room2:matrix.org"),
            name: Some("Beta Room".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            suggested: false,
        },
        matrix::RoomData {
            id: std::sync::Arc::from("!room3:matrix.org"),
            name: Some("Gamma Room".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            suggested: false,
        },
    ];

    // Mark !room2 as already joined
    app.joined_room_ids
        .insert(std::sync::Arc::from("!room2:matrix.org"));

    app.selected_space = Some(matrix_sdk::ruma::RoomId::parse("!space1:matrix.org").unwrap());

    // Test without search query first
    app.update_filtered_rooms();

    // other_rooms should be filtered to remove joined ones, leaving only Gamma Room (!room3)
    assert_eq!(app.other_rooms.len(), 1);
    assert_eq!(app.other_rooms[0].id.as_ref(), "!room3:matrix.org");

    // filtered_other_rooms should contain the index of Gamma Room
    assert_eq!(app.filtered_other_rooms.len(), 1);
    assert_eq!(app.filtered_other_rooms[0], 0); // index 0 in the newly retained `other_rooms` vector

    // Now test with search query
    app.search_query = "beta".to_string(); // Note: room2 was removed, so this should return empty
    app.update_filtered_rooms();
    assert_eq!(app.filtered_other_rooms.len(), 0);

    app.search_query = "gamma".to_string();
    app.update_filtered_rooms();
    assert_eq!(app.filtered_other_rooms.len(), 1);
}

#[tokio::test]
async fn test_get_room_data_not_found() {
    let tmp_dir = tempfile::tempdir().unwrap();
    let engine = match matrix::MatrixEngine::new(tmp_dir.path().to_path_buf()).await {
        Ok(e) => e,
        Err(e) => {
            tracing::info!(
                "Skipping test due to engine initialization failure (likely dbus/keyring): {}",
                e
            );
            return;
        }
    };

    let room_id = matrix_sdk::ruma::RoomId::parse("!nonexistent:example.com").unwrap();

    let result = get_room_data(&engine, &room_id).await;

    assert!(result.is_none());
}

#[test]
fn test_update_room_joined_error() {
    let mut app = create_test_app();
    let _ = app.update(Message::RoomJoined(
        Err("some connection error".to_string()),
    ));

    assert_eq!(
        app.error,
        Some(crate::fl!("error-failed-join-room", error = "some connection error").to_string())
    );
}

#[test]
fn test_room_name_cache() {
    let mut app = create_test_app();
    let room_id: std::sync::Arc<str> = std::sync::Arc::from("!room1:matrix.org");

    assert_eq!(app.get_room_name(&room_id), None);

    app.room_name_cache
        .insert(room_id.clone(), "Cached Room Name".to_string());
    assert_eq!(app.get_room_name(&room_id), Some("Cached Room Name"));

    app.room_list = vec![matrix::RoomData {
        id: room_id.clone(),
        name: Some("Active Room Name".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    }];
    app.rebuild_room_index();
    assert_eq!(app.get_room_name(&room_id), Some("Active Room Name"));
}

#[test]
fn test_search_query_changed_debounce() {
    let mut app = create_test_app();
    assert_eq!(app.search_generation, 0);

    // Typing a search query should increment the search generation (debounce tracking)
    let _ = app.update(Message::SearchQueryChanged("hello".to_string()));
    assert_eq!(app.search_query, "hello");
    assert_eq!(app.search_generation, 1);

    // Typing more should increment it further
    let _ = app.update(Message::SearchQueryChanged("hello world".to_string()));
    assert_eq!(app.search_query, "hello world");
    assert_eq!(app.search_generation, 2);

    // Clearing the search query should set flags to false and increment generation again
    app.is_searching_public = true;
    app.is_searching_messages = true;
    let _ = app.update(Message::SearchQueryChanged("".to_string()));
    assert_eq!(app.search_query, "");
    assert_eq!(app.search_generation, 3);
    assert!(!app.is_searching_public);
    assert!(!app.is_searching_messages);
}

#[test]
fn test_dnd_data_received_uri_list() {
    let mut app = create_test_app();
    let temp_dir = tempfile::tempdir().unwrap();
    let temp_file_path = temp_dir.path().join("dropped_file.txt");
    std::fs::write(&temp_file_path, "test").unwrap();

    let uri = format!("file://{}\r\n", temp_file_path.to_str().unwrap());
    let data = uri.into_bytes();

    let _ = app.update(Message::DndDataReceived("text/uri-list".to_string(), data));

    assert_eq!(app.composer_attachments.len(), 1);
    assert_eq!(app.composer_attachments[0], temp_file_path);
}

fn space_room(id: &str, name: Option<&str>) -> matrix::RoomData {
    matrix::RoomData {
        id: std::sync::Arc::from(id),
        name: name.map(str::to_string),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: true,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    }
}

fn nav_space_ids(app: &Constellation) -> Vec<Option<std::sync::Arc<str>>> {
    app.space_nav_model
        .iter()
        .map(|entity| {
            app.space_nav_model
                .data::<std::sync::Arc<str>>(entity)
                .cloned()
        })
        .collect()
}

#[test]
fn test_rebuild_space_nav_model_lists_all_rooms_and_spaces() {
    let mut app = create_test_app();
    app.room_list = vec![
        space_room("!space1:matrix.org", Some("Space One")),
        space_room("!space2:matrix.org", None),
    ];

    app.rebuild_space_nav_model();

    // Position 0 is the "All rooms" pseudo-entry (no room id data); the two
    // joined spaces follow with their ids attached.
    let ids = nav_space_ids(&app);
    assert_eq!(ids.len(), 3);
    assert_eq!(ids[0], None);
    assert_eq!(ids[1].as_deref(), Some("!space1:matrix.org"));
    assert_eq!(ids[2].as_deref(), Some("!space2:matrix.org"));

    let texts: Vec<String> = app
        .space_nav_model
        .iter()
        .map(|e| app.space_nav_model.text(e).unwrap_or("").to_string())
        .collect();
    assert_eq!(texts[1], "Space One");
    assert!(!texts[2].is_empty());
}

#[test]
fn test_rebuild_space_nav_model_skips_invalid_room_ids() {
    let mut app = create_test_app();
    app.room_list = vec![space_room("not-a-valid-room-id", Some("Bad"))];

    app.rebuild_space_nav_model();

    // Only the "All rooms" entry survives.
    assert_eq!(nav_space_ids(&app), vec![None]);
}

#[test]
fn test_rebuild_space_nav_model_ignores_plain_rooms() {
    let mut app = create_test_app();
    app.room_list = vec![matrix::RoomData {
        id: std::sync::Arc::from("!room:matrix.org"),
        name: Some("Room".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        order: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        suggested: false,
    }];

    app.rebuild_space_nav_model();

    assert_eq!(nav_space_ids(&app), vec![None]);
}

#[test]
fn test_select_space_activates_matching_nav_entry() {
    let mut app = create_test_app();
    app.room_list = vec![space_room("!space1:matrix.org", Some("Space One"))];
    app.rebuild_space_nav_model();

    // Select the space through the same message path the nav bar uses.
    let _ = app.update(Message::SelectSpace(Some(std::sync::Arc::from(
        "!space1:matrix.org",
    ))));

    let active = app.space_nav_model.active();
    let active_id = app
        .space_nav_model
        .data::<std::sync::Arc<str>>(active)
        .cloned();
    assert_eq!(active_id.as_deref(), Some("!space1:matrix.org"));
}

#[test]
fn test_select_all_rooms_activates_first_nav_entry() {
    let mut app = create_test_app();
    app.room_list = vec![space_room("!space1:matrix.org", Some("Space One"))];
    app.rebuild_space_nav_model();

    // Switch away then back to "All rooms".
    let _ = app.update(Message::SelectSpace(Some(std::sync::Arc::from(
        "!space1:matrix.org",
    ))));
    let _ = app.update(Message::SelectSpace(None));

    let active = app.space_nav_model.active();
    assert_eq!(app.space_nav_model.position(active), Some(0));
    assert_eq!(
        app.space_nav_model.data::<std::sync::Arc<str>>(active),
        None
    );
}

#[test]
fn test_rebuild_space_nav_model_stable_fingerprint() {
    let mut app = create_test_app();
    app.room_list = vec![space_room("!space1:matrix.org", Some("Space One"))];

    app.rebuild_space_nav_model();
    let first = app.space_nav_model.iter().collect::<Vec<_>>();

    // Rebuild with no change: entity ids stay identical (scroll/selection
    // state is not invalidated).
    app.rebuild_space_nav_model();
    let second = app.space_nav_model.iter().collect::<Vec<_>>();
    assert_eq!(first, second);

    // Adding a space changes the fingerprint and rebuilds.
    app.room_list
        .push(space_room("!space2:matrix.org", Some("Space Two")));
    app.rebuild_space_nav_model();
    assert_eq!(nav_space_ids(&app).len(), 3);
}

#[test]
fn test_create_main_panes_default_and_custom_ratio() {
    let panes = create_main_panes(0.30);
    assert_eq!(panes.len(), 2);
    match panes.layout() {
        cosmic::widget::pane_grid::Node::Split { ratio, .. } => {
            assert!((*ratio - 0.30).abs() < 1e-5);
        }
        _ => panic!("expected a split node"),
    }

    // Out of bounds and non-finite ratios fall back to default
    let fallback_nan = create_main_panes(f32::NAN);
    match fallback_nan.layout() {
        cosmic::widget::pane_grid::Node::Split { ratio, .. } => {
            assert!((*ratio - DEFAULT_SIDEBAR_RATIO).abs() < 1e-5);
        }
        _ => panic!("expected a split node"),
    }

    let fallback_neg = create_main_panes(-0.5);
    match fallback_neg.layout() {
        cosmic::widget::pane_grid::Node::Split { ratio, .. } => {
            assert!((*ratio - DEFAULT_SIDEBAR_RATIO).abs() < 1e-5);
        }
        _ => panic!("expected a split node"),
    }
}

#[test]
fn test_pane_resized_updates_state_and_config() {
    let mut app = create_test_app();
    assert!((app.sidebar_ratio - DEFAULT_SIDEBAR_RATIO).abs() < 1e-5);
    assert!((app.current_sidebar_ratio() - DEFAULT_SIDEBAR_RATIO).abs() < 1e-5);

    let split_id = match app.panes.layout() {
        cosmic::widget::pane_grid::Node::Split { id, .. } => *id,
        _ => panic!("expected split node in main panes"),
    };

    let _ = app.update(Message::PaneResized(
        cosmic::widget::pane_grid::ResizeEvent {
            split: split_id,
            ratio: 0.38,
        },
    ));

    assert!((app.sidebar_ratio - 0.38).abs() < 1e-5);
    assert!((app.current_sidebar_ratio() - 0.38).abs() < 1e-5);

    let config = app.build_config();
    assert!((config.sidebar_ratio - 0.38).abs() < 1e-5);
}
#[test]
fn test_room_settings_open_panel_routes_to_settings_panel() {
    let mut app = create_test_app();

    // #423: permissions moved out of Room settings into its own page.
    let _ = app.update(Message::RoomSettings(settings::room::Message::OpenPanel(
        SettingsPanel::Permissions,
    )));
    assert_eq!(app.current_settings_panel, Some(SettingsPanel::Permissions));

    // #424: Room settings button opens the manage members page.
    let _ = app.update(Message::RoomSettings(settings::room::Message::OpenPanel(
        SettingsPanel::ManageRoomMembers,
    )));
    assert_eq!(
        app.current_settings_panel,
        Some(SettingsPanel::ManageRoomMembers)
    );
}
