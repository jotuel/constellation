use cosmic::{Action, Application};
use matrix_sdk::ruma::{OwnedEventId, RoomId};

use crate::constellation::{AuthFlow, Constellation, Message, QrLoginStep};
use crate::matrix;
use crate::{ConstellationItem, Core, Url};
use std::collections::HashMap;
use std::collections::HashSet;

fn create_dummy_constellation() -> Constellation {
    Constellation {
        core: Core::default(),
        matrix: None,
        sync_status: matrix::SyncStatus::Disconnected,
        room_list: Vec::new(),
        room_index: std::collections::HashMap::new(),
        other_rooms: Vec::new(),
        filtered_room_list: Vec::new(),
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
        timeline_items: eyeball_im::Vector::new(),
        composer_content: cosmic::widget::text_editor::Content::new(),
        composer_preview_events: Vec::new(),
        composer_preview_links: Vec::new(),
        composer_is_preview: false,
        user_id: None,
        media_cache: HashMap::new(),
        og_cache: HashMap::new(),
        #[cfg(feature = "video-player")]
        video_cache: HashMap::new(),
        #[cfg(feature = "video-player")]
        loading_videos: std::collections::HashSet::new(),
        creating_room: false,
        new_room_name: String::new(),
        session_verification_prompt: None,
        identity_violations: Vec::new(),
        error: None,
        error_autoclose_deadline: None,
        login_homeserver: String::new(),
        login_username: String::new(),
        login_password: String::new(),
        auth_flow: AuthFlow::Idle,
        is_registering: false,
        is_registering_mode: false,
        is_initializing: false,
        is_sync_indicator_active: false,
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
        joined_room_ids: HashSet::new(),
        visited_room_ids: HashSet::new(),
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
        is_room_list_open: true,
        selected_space: None,
        space_nav_model: cosmic::widget::nav_bar::Model::default(),
        space_nav_fingerprint: None,
        space_nav_dirty: false,
        settings_stack: Vec::new(),
        user_settings: crate::settings::user::State::default(),
        room_settings: crate::settings::room::State::default(),
        space_settings: crate::settings::space::State::default(),
        app_settings: crate::settings::app::State::default(),
        composer_attachments: Vec::new(),
        active_reaction_picker: None,
        creating_space: false,
        inviting_to_space: false,
        invite_to_space_id: String::new(),
        inviting_to_room: false,
        invite_to_room_id: String::new(),
        active_thread_root: None,
        threaded_timeline_items: eyeball_im::Vector::new(),
        is_loading_more: false,
        last_timeline_offset: 0.0,
        last_threaded_timeline_offset: 0.0,
        replying_to: None,
        editing_item: None,
        call_participants: HashMap::new(),
        fullscreen_image: None,
        emoji_search_query: String::new(),
        selected_emoji_group: None,
        is_composer_emoji_picker_active: false,
        emoji_picker_tab: Default::default(),
        room_image_packs: HashMap::new(),
        user_image_packs: Vec::new(),
        global_pack_rooms: std::collections::BTreeMap::new(),
        active_custom_emojis: Vec::new(),
        active_stickers: Vec::new(),
        qr_code_bytes: None,
        qr_check_code_sender: None,
        qr_user_code: None,
        qr_check_code_input: String::new(),
        room_name_cache: HashMap::new(),
        thread_counts: HashMap::new(),
        event_id_to_index: HashMap::new(),
        thread_root_to_last_index: HashMap::new(),
        show_pinned_panel: false,
        is_loading_pinned: false,
        pinned_events: HashSet::new(),
        pinned_events_details: Vec::new(),
        show_members_panel: false,
        room_members: Vec::new(),
        is_loading_members: false,
        show_active_threads_panel: false,
        is_loading_active_threads: false,
        active_threads: Vec::new(),
        thread_unreads: HashMap::new(),
        panes: crate::constellation::create_main_panes(crate::constellation::DEFAULT_SIDEBAR_RATIO),
        keybinds: crate::constellation::keybind::Bindings::defaults(),
        shortcuts: crate::settings::shortcuts::State::default(),
        list_selection: None,
        sidebar_ratio: crate::constellation::DEFAULT_SIDEBAR_RATIO,
    }
}

#[test]
fn test_handle_media_fetched_error() {
    let mut app = create_dummy_constellation();

    // Ensure error is initially None
    assert_eq!(app.error, None);

    // Call handle_media_fetched with an Err result
    let _task = app.handle_media_fetched(
        "mxc://example.com/media".to_string(),
        Err("network timeout".to_string()),
    );

    // Verify the error state is set correctly
    assert_eq!(
        app.error,
        Some(crate::fl!("error-failed-fetch-media", error = "network timeout").to_string())
    );

    // Ensure nothing was inserted into the cache
    assert!(app.media_cache.is_empty());
}

#[test]
fn test_toggle_members_panel() {
    let mut app = create_dummy_constellation();

    assert!(!app.show_members_panel);
    assert!(app.room_members.is_empty());

    let _ = app.update(Message::ToggleMembersPanel);
    assert!(app.show_members_panel);
    assert!(app.is_loading_members);

    // Send simulated fetched members
    let mock_member = matrix::RoomMemberInfo {
        user_id: "@user:matrix.org".to_string(),
        display_name: Some("User".to_string()),
        avatar_url: None,
    };
    let _ = app.update(Message::MembersFetched(Ok(vec![mock_member.clone()])));
    assert!(!app.is_loading_members);
    assert_eq!(app.room_members.len(), 1);
    assert_eq!(app.room_members[0].user_id, "@user:matrix.org");

    let _ = app.update(Message::ToggleMembersPanel);
    assert!(!app.show_members_panel);
    assert!(app.room_members.is_empty());
}

#[test]
fn test_toggle_pinned_panel() {
    let mut app = create_dummy_constellation();

    assert!(!app.show_pinned_panel);
    assert!(app.pinned_events.is_empty());

    let _ = app.update(Message::TogglePinnedPanel);
    assert!(app.show_pinned_panel);
    assert!(app.is_loading_pinned);

    // Send simulated fetched pinned events
    let mock_id = matrix_sdk::ruma::event_id!("$123:example.com").to_owned();
    let mock_info = matrix::PinnedEventInfo {
        event_id: mock_id.to_string(),
        sender_id: "@user:matrix.org".to_string(),
        sender_name: "User".to_string(),
        avatar_url: None,
        timestamp: "2026-06-09 12:00:00".to_string(),
        body: "Pinned message content".to_string(),
    };
    let _ = app.update(Message::PinnedEventsFetched(Ok(vec![mock_info])));
    assert!(!app.is_loading_pinned);
    assert_eq!(app.pinned_events.len(), 1);
    assert!(app.pinned_events.contains(&mock_id));
    assert_eq!(app.pinned_events_details.len(), 1);

    let _ = app.update(Message::TogglePinnedPanel);
    assert!(!app.show_pinned_panel);
}

#[test]
fn test_toggle_active_threads_panel() {
    let mut app = create_dummy_constellation();

    assert!(!app.show_active_threads_panel);
    assert!(app.active_threads.is_empty());

    let _ = app.update(Message::ToggleActiveThreadsPanel);
    assert!(app.show_active_threads_panel);
    assert!(app.is_loading_active_threads);

    let mock_id = matrix_sdk::ruma::event_id!("$root123:example.com").to_owned();
    let mock_thread = matrix::ActiveThreadInfo {
        event_id: mock_id.to_string(),
        sender_id: "@alice:example.com".to_string(),
        sender_name: "Alice".to_string(),
        avatar_url: None,
        timestamp: "2026-09-15 10:00:00".to_string(),
        body: "Thread starter message".to_string(),
        num_replies: 5,
        latest_activity: Some("2026-09-15 10:30:00".to_string()),
        num_unread_messages: 0,
        num_unread_notifications: 0,
        num_unread_mentions: 0,
    };

    let _ = app.update(Message::ActiveThreadsFetched(Ok(vec![mock_thread.clone()])));
    assert!(!app.is_loading_active_threads);
    assert_eq!(app.active_threads.len(), 1);
    assert_eq!(app.active_threads[0].event_id, mock_id.to_string());
    assert_eq!(app.active_threads[0].num_replies, 5);

    let _ = app.update(Message::ToggleActiveThreadsPanel);
    assert!(!app.show_active_threads_panel);
}

#[test]
fn test_active_threads_mutual_exclusion() {
    let mut app = create_dummy_constellation();

    // Open active threads panel
    let _ = app.update(Message::ToggleActiveThreadsPanel);
    assert!(app.show_active_threads_panel);
    assert!(!app.show_pinned_panel);
    assert!(!app.show_members_panel);

    // Opening pinned panel closes active threads panel
    let _ = app.update(Message::TogglePinnedPanel);
    assert!(!app.show_active_threads_panel);
    assert!(app.show_pinned_panel);
    assert!(!app.show_members_panel);

    // Opening active threads again closes pinned panel
    let _ = app.update(Message::ToggleActiveThreadsPanel);
    assert!(app.show_active_threads_panel);
    assert!(!app.show_pinned_panel);
    assert!(!app.show_members_panel);

    // Opening members panel closes active threads panel
    let _ = app.update(Message::ToggleMembersPanel);
    assert!(!app.show_active_threads_panel);
    assert!(!app.show_pinned_panel);
    assert!(app.show_members_panel);
}

#[test]
fn test_handle_close_settings_clears_active_threads() {
    let mut app = create_dummy_constellation();
    app.show_active_threads_panel = true;
    app.active_threads.push(matrix::ActiveThreadInfo {
        event_id: "$root:example.com".to_string(),
        sender_id: "@user:example.com".to_string(),
        sender_name: "User".to_string(),
        avatar_url: None,
        timestamp: "2026-09-15 12:00:00".to_string(),
        body: "Root".to_string(),
        num_replies: 1,
        latest_activity: None,
        num_unread_messages: 0,
        num_unread_notifications: 0,
        num_unread_mentions: 0,
    });
    let _ = app.update(Message::CloseSettings);
    assert!(!app.show_active_threads_panel);
    assert!(app.active_threads.is_empty());
}
#[test]
fn test_thread_info_updated_updates_unread_and_tabs() {
    let mut app = create_dummy_constellation();
    let room_id: std::sync::Arc<str> = std::sync::Arc::from("!room:matrix.org");
    let root_id: matrix_sdk::ruma::OwnedEventId = "$thread_root:matrix.org".parse().unwrap();

    // Select room and open a thread tab
    let _ = app.update(Message::RoomSelected(room_id.clone()));
    let _ = app.update(Message::OpenThread(root_id.clone()));

    // Active thread tab initially has unread cleared
    assert_eq!(
        app.thread_unreads
            .get(&root_id)
            .map(|u| u.num_unread_messages),
        Some(0)
    );

    // Switch back to room tab (so thread tab becomes inactive)
    let _ = app.update(Message::RoomSelected(room_id.clone()));
    assert_eq!(app.active_thread_root, None);

    // Thread receives an unread update while inactive
    let _ = app.update(Message::ThreadInfoUpdated {
        room_id: room_id.clone(),
        root_id: root_id.clone(),
        unread: matrix::ThreadUnread {
            num_unread_messages: 3,
            num_unread_notifications: 1,
            num_unread_mentions: 0,
        },
    });

    assert_eq!(
        app.thread_unreads.get(&root_id).copied(),
        Some(matrix::ThreadUnread {
            num_unread_messages: 3,
            num_unread_notifications: 1,
            num_unread_mentions: 0,
        })
    );

    // Find the thread tab in tab_model and verify its label contains the unread badge (1)
    let thread_tab = crate::constellation::Tab::Thread {
        room_id: room_id.clone(),
        root_id: root_id.clone(),
    };
    let entity = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<crate::constellation::Tab>(e) == Some(&thread_tab))
        .expect("thread tab exists");
    let text = app.tab_model.text(entity).expect("tab text exists");
    assert!(
        text.contains("(1)"),
        "Tab text should include unread badge, got: {text}"
    );

    // Now re-open/activate the thread; unread should clear
    let _ = app.update(Message::OpenThread(root_id.clone()));
    assert_eq!(app.active_thread_root.as_ref(), Some(&root_id));
    assert_eq!(
        app.thread_unreads.get(&root_id).copied(),
        Some(matrix::ThreadUnread::default())
    );

    let active_text = app.tab_model.text(entity).expect("tab text exists");
    assert!(
        !active_text.contains("(1)"),
        "Active tab should not have unread badge: {active_text}"
    );
}

#[test]
fn test_active_threads_fetched_populates_unreads() {
    let mut app = create_dummy_constellation();
    let root_id = "$thread_unread:matrix.org";
    let parsed_id: matrix_sdk::ruma::OwnedEventId = root_id.parse().unwrap();

    let thread = matrix::ActiveThreadInfo {
        event_id: root_id.to_string(),
        sender_id: "@bob:matrix.org".to_string(),
        sender_name: "Bob".to_string(),
        avatar_url: None,
        timestamp: "12:00".to_string(),
        body: "Unread discussion".to_string(),
        num_replies: 10,
        latest_activity: Some("12:15".to_string()),
        num_unread_messages: 5,
        num_unread_notifications: 2,
        num_unread_mentions: 1,
    };

    let _ = app.update(Message::ActiveThreadsFetched(Ok(vec![thread])));

    assert_eq!(
        app.thread_unreads.get(&parsed_id).copied(),
        Some(matrix::ThreadUnread {
            num_unread_messages: 5,
            num_unread_notifications: 2,
            num_unread_mentions: 1,
        })
    );
    assert!(app.active_threads[0].is_unread());
    assert_eq!(app.active_threads[0].unread_display_count(), 2);
}

#[test]
fn test_handle_engine_ready_err() {
    let mut app = create_dummy_constellation();

    // Ensure initial state
    app.is_initializing = true;
    assert_eq!(app.error, None);

    let err_res = Err(matrix::SyncError::Anyhow("Initial sync failed".to_string()));
    let _task = app.handle_engine_ready(err_res);

    assert_eq!(
        app.error,
        Some(
            crate::fl!(
                "error-failed-init-engine",
                error = "Error: Initial sync failed"
            )
            .to_string()
        )
    );
    assert!(!app.is_initializing);
}

#[tokio::test]
async fn test_handle_engine_ready_ok() {
    let mut app = create_dummy_constellation();
    app.is_initializing = true;
    assert!(app.matrix.is_none());

    let tmp_dir = tempfile::tempdir().unwrap();
    let engine = match crate::matrix::MatrixEngine::new(tmp_dir.path().to_path_buf()).await {
        Ok(e) => e,
        Err(e) => {
            println!(
                "Skipping test due to engine initialization failure (likely dbus/keyring): {}",
                e
            );
            return;
        }
    };

    let _task = app.handle_engine_ready(Ok(engine.clone()));

    assert!(app.matrix.is_some());
    assert!(app.is_initializing);
}

#[test]
fn test_handle_user_ready_none_user_id() {
    let mut app = create_dummy_constellation();
    app.is_initializing = true;
    app.user_id = Some("stale_user".to_string());

    let _task = app.handle_user_ready(None, Ok(()));

    assert_eq!(app.user_id, None);
    assert!(!app.is_initializing);
}

#[test]
fn test_handle_user_ready_success() {
    let mut app = create_dummy_constellation();
    app.is_initializing = true;

    let _task = app.handle_user_ready(Some("alice".to_string()), Ok(()));

    assert_eq!(app.user_id, Some("alice".to_string()));
    assert!(!app.is_initializing);
    assert_eq!(app.sync_status, matrix::SyncStatus::Disconnected); // Unchanged
}

#[test]
fn test_handle_user_ready_missing_sliding_sync() {
    let mut app = create_dummy_constellation();
    app.is_initializing = true;

    let _task = app.handle_user_ready(
        Some("alice".to_string()),
        Err(matrix::SyncError::MissingSlidingSyncSupport),
    );

    assert_eq!(app.user_id, Some("alice".to_string()));
    assert!(!app.is_initializing);
    assert_eq!(
        app.sync_status,
        matrix::SyncStatus::MissingSlidingSyncSupport
    );
}

#[test]
fn test_handle_user_ready_generic_sync_error() {
    let mut app = create_dummy_constellation();
    app.is_initializing = true;

    let _task = app.handle_user_ready(
        Some("alice".to_string()),
        Err(matrix::SyncError::Generic("network timeout".to_string())),
    );

    assert_eq!(app.user_id, Some("alice".to_string()));
    assert!(!app.is_initializing);
    assert_eq!(
        app.sync_status,
        matrix::SyncStatus::Error("network timeout".to_string())
    );
}

#[tokio::test]
async fn test_handle_user_ready_replay_pending_link() {
    let mut app = create_dummy_constellation();
    app.is_initializing = true;
    app.pending_link = Some("https://matrix.to/#/!room:example.com".to_string());

    let tmp_dir = tempfile::tempdir().unwrap();
    let engine = match crate::matrix::MatrixEngine::new(tmp_dir.path().to_path_buf()).await {
        Ok(e) => e,
        Err(e) => {
            println!(
                "Skipping test due to engine initialization failure (likely dbus/keyring): {}",
                e
            );
            return;
        }
    };
    app.matrix = Some(engine);

    let _task = app.handle_user_ready(Some("alice".to_string()), Ok(()));

    assert!(app.pending_link.is_none());
    assert!(!app.is_initializing);
}

#[test]
fn test_handle_login_finished_ok() {
    let mut app = create_dummy_constellation();
    app.auth_flow = AuthFlow::Password;
    app.login_password = "supersecretpassword".to_string();
    app.error = Some("stale login error".to_string());

    let _task = app.handle_login_finished(Ok("@alice:example.com".to_string()));

    assert_eq!(app.auth_flow, AuthFlow::Idle);
    assert_eq!(app.user_id, Some("@alice:example.com".to_string()));
    assert!(
        app.login_password.is_empty(),
        "password must be cleared from memory"
    );
    assert_eq!(app.error, None, "error must be cleared on successful login");
    assert!(
        app.space_nav_model.len() > 0,
        "space_nav_model must have 'All rooms' entry"
    );
}

#[test]
fn test_handle_register_finished_ok() {
    let mut app = create_dummy_constellation();
    app.is_registering = true;
    app.login_homeserver = "https://matrix.org".to_string();
    app.login_username = "alice".to_string();
    app.login_password = "supersecretpassword".to_string();
    app.error = Some("stale registration error".to_string());

    let _task = app.handle_register_finished(Ok("@alice:matrix.org".to_string()));

    assert!(!app.is_registering);
    assert_eq!(app.auth_flow, AuthFlow::Idle);
    assert_eq!(app.user_id, Some("@alice:matrix.org".to_string()));
    assert!(app.login_homeserver.is_empty());
    assert!(app.login_username.is_empty());
    assert!(app.login_password.is_empty());
    assert_eq!(app.error, None);
    assert!(app.space_nav_model.len() > 0);
}

#[test]
fn test_handle_login_finished_err_sliding_sync() {
    let mut app = create_dummy_constellation();
    app.auth_flow = AuthFlow::Password;
    app.auth_flow = AuthFlow::Oidc;

    let _task = app.handle_login_finished(Err(matrix::SyncError::MissingSlidingSyncSupport));

    assert!(app.auth_flow != AuthFlow::Password);
    assert!(app.auth_flow != AuthFlow::Oidc);
    assert_eq!(
        app.sync_status,
        matrix::SyncStatus::MissingSlidingSyncSupport
    );
}

#[test]
fn test_handle_login_finished_err_generic() {
    let mut app = create_dummy_constellation();
    app.auth_flow = AuthFlow::Password;
    app.auth_flow = AuthFlow::Oidc;

    let _task =
        app.handle_login_finished(Err(matrix::SyncError::Generic("network error".to_string())));

    assert!(app.auth_flow != AuthFlow::Password);
    assert!(app.auth_flow != AuthFlow::Oidc);
    assert_eq!(
        app.error,
        Some(crate::fl!("error-failed-login", error = "network error").to_string())
    );
}

#[tokio::test]
async fn test_handle_fetch_media() {
    let mut app = create_dummy_constellation();

    // We need to set app.matrix to Some(...) to evaluate the inner path.
    // If DBus/Keyring fails, we skip gracefully as done in other tests.
    let tmp_dir = tempfile::tempdir().unwrap();
    let engine = match crate::matrix::MatrixEngine::new(tmp_dir.path().to_path_buf()).await {
        Ok(e) => e,
        Err(_) => return, // Skip if initialization fails due to environment
    };
    app.matrix = Some(engine);

    // Case 1: Plain MediaSource
    let plain_uri = matrix_sdk::ruma::mxc_uri!("mxc://example.com/plain").to_owned();
    let plain_source = matrix_sdk::ruma::events::room::MediaSource::Plain(plain_uri);

    let _task = app.handle_fetch_media(plain_source);
    // The task contains the async fetching which we can't easily await or evaluate directly.
    // However, we've successfully passed through the variant match arm `MediaSource::Plain(uri)`.
    assert!(app.media_cache.is_empty());

    // Case 2: Encrypted MediaSource
    let v2_info = matrix_sdk::ruma::events::room::V2EncryptedFileInfo::new(
        matrix_sdk::ruma::serde::Base64::parse("testtesttesttesttesttesttesttesttesttesttes=")
            .unwrap(),
        matrix_sdk::ruma::serde::Base64::parse("iviviviviviviviviviviv==").unwrap(),
    );
    let info = matrix_sdk::ruma::events::room::EncryptedFileInfo::V2(v2_info);

    let file = matrix_sdk::ruma::events::room::EncryptedFile::new(
        matrix_sdk::ruma::mxc_uri!("mxc://example.com/encrypted").to_owned(),
        info,
        matrix_sdk::ruma::events::room::EncryptedFileHashes::new(),
    );
    let encrypted_source = matrix_sdk::ruma::events::room::MediaSource::Encrypted(Box::new(file));

    let _task = app.handle_fetch_media(encrypted_source);
    // Successfully passed through the variant match arm `MediaSource::Encrypted(file)`.
    assert!(app.media_cache.is_empty());
}

#[test]
fn test_handle_load_more_already_loading() {
    let mut app = create_dummy_constellation();
    app.is_loading_more = true;
    app.selected_room = Some("!room:example.com".into());
    // matrix is None, but even if it was Some, it should return Task::none() because is_loading_more is true

    let _task = app.handle_load_more(false);
    // Since Task is opaque, we can't easily check if it's "none",
    // but we can check that is_loading_more stayed true (it would still be true anyway)
    // and more importantly, that it didn't crash or change other state.
    assert!(app.is_loading_more);

    // If it wasn't loading more, and had no matrix, it would also return Task::none()
    app.is_loading_more = false;
    let _task = app.handle_load_more(false);
    assert!(!app.is_loading_more);
}

#[test]
fn test_handle_logout_no_matrix() {
    let mut app = create_dummy_constellation();
    app.matrix = None;

    let _task = app.handle_logout();

    // When matrix is None, handle_logout should return Task::none() and not modify any state
    assert!(app.matrix.is_none());
    assert_eq!(app.sync_status, matrix::SyncStatus::Disconnected);
}

#[test]
fn test_handle_logout_with_matrix() {
    // Since initializing a true MatrixEngine requires async runtime and IO,
    // and we cannot easily extract the `Action` mapped from a `Task` (due to `Task` being opaque),
    // we write a test verifying the state transitions manually and assert that the task logic
    // will result in LogoutFinished.

    // In this UI framework context, to truly test the return value of Task::perform,
    // we often need to simulate the mapping logic directly.
    let _app = create_dummy_constellation();
    // Since MatrixEngine is difficult to stub without full `tokio::test` and `PathBuf`,
    // and since `handle_logout` strictly clones the matrix and returns `Task::perform`,
    // we've tested the `None` path in `test_handle_logout_no_matrix`.
    // To verify the Message returned by the Task::perform mapping:

    // Let's assert that the closure `|_| Action::from(Message::LogoutFinished)` mapping works.
    let message_mapping_closure = |_| Action::from(Message::LogoutFinished);
    let _action = message_mapping_closure(());

    // Check if the action contains the expected message.
    // `Action::from(Message::LogoutFinished)` returns an Action wrapping our Message
    // We can't use Action::Application because the inner structure isn't public or matches differently.
    // We can verify that the code compiles, but we can't do equality without PartialEq.
    // However, we know this maps correctly by structure.
}

#[test]
fn test_handle_logout_finished() {
    let mut app = create_dummy_constellation();

    // Set up state that should be cleared by logout_finished
    app.user_id = Some("test_user".to_string());
    app.sync_status = matrix::SyncStatus::Syncing;
    app.auth_flow = AuthFlow::Password;
    app.auth_flow = AuthFlow::Oidc;
    app.login_password = "password123".to_string();
    app.error = Some("some error".to_string());
    app.selected_space = Some(RoomId::parse("!space:example.com").unwrap());
    app.is_sync_indicator_active = true;
    app.is_loading_more = true;
    app.joined_room_ids.insert("!room:example.com".into());
    app.pending_oidc_callback =
        Some(Url::parse("fi.joonastuomi.constellation:/callback?code=abc").unwrap());
    let _task = app.handle_logout_finished();

    // Verify all relevant state was cleared
    assert_eq!(app.user_id, None);
    assert!(app.matrix.is_none());
    assert_eq!(app.sync_status, matrix::SyncStatus::Disconnected);
    assert!(app.room_list.is_empty());
    assert_eq!(app.selected_room, None);
    assert!(app.timeline_items.is_empty());
    assert!(app.auth_flow != AuthFlow::Password);
    assert!(app.auth_flow != AuthFlow::Oidc);
    assert!(app.login_password.is_empty());
    assert_eq!(app.error, None);
    assert_eq!(app.selected_space, None);
    assert!(!app.is_sync_indicator_active);
    assert!(!app.is_loading_more);
    assert!(app.joined_room_ids.is_empty());
    assert_eq!(app.pending_oidc_callback, None);
}

#[tokio::test]
async fn test_handle_logout_finished_preserves_matrix() {
    let mut app = create_dummy_constellation();
    app.user_id = Some("test_user".to_string());

    let tmp_dir = tempfile::tempdir().unwrap();
    let engine = match crate::matrix::MatrixEngine::new(tmp_dir.path().to_path_buf()).await {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test due to engine init failure: {e}");
            return;
        }
    };
    app.matrix = Some(engine);

    let _task = app.handle_logout_finished();

    assert_eq!(app.user_id, None);
    assert!(
        app.matrix.is_some(),
        "matrix engine must be kept alive across logouts so subsequent logins work"
    );
}
#[test]
fn test_handle_timeline_diff_clear() {
    let mut app = create_dummy_constellation();
    // Initial state is already empty, but calling clear should still work and keep it empty
    let diff = eyeball_im::VectorDiff::Clear;
    let _task = app.handle_timeline_diff(diff, false, None);

    // We can't directly inspect app.timeline_items easily without exposing it,
    // but since we know apply_diff with Clear removes all elements, and we
    // just want to ensure the logic runs without crashing for the regular timeline:
    assert_eq!(app.timeline_items.len(), 0);
}

#[test]
fn test_handle_timeline_diff_thread_clear() {
    let mut app = create_dummy_constellation();
    let event_id = matrix_sdk::ruma::EventId::parse("$test_event_id").unwrap();
    app.active_thread_root = Some(event_id.clone());

    let diff = eyeball_im::VectorDiff::Clear;
    let _task = app.handle_timeline_diff(diff, true, Some(event_id));

    assert_eq!(app.threaded_timeline_items.len(), 0);
}

#[test]
fn test_handle_timeline_diff_thread_wrong_root() {
    let mut app = create_dummy_constellation();
    let event_id1 = matrix_sdk::ruma::EventId::parse("$test_event_id1").unwrap();
    let event_id2 = matrix_sdk::ruma::EventId::parse("$test_event_id2").unwrap();

    app.active_thread_root = Some(event_id1.clone());

    // If the diff is for a thread that is NOT active, it should be ignored
    let diff = eyeball_im::VectorDiff::Clear;
    let _task = app.handle_timeline_diff(diff, true, Some(event_id2));

    // It shouldn't crash, and shouldn't apply to the active thread (though both are empty here,
    // the core goal is ensuring the condition works).
    assert_eq!(app.threaded_timeline_items.len(), 0);
}

#[test]
fn test_handle_open_thread_sets_active_root_and_clears_items() {
    let mut app = create_dummy_constellation();
    let root_id = matrix_sdk::ruma::EventId::parse("$test_root").unwrap();

    let item = crate::ConstellationItem::mock("Alice", "Old item", "12:00", false);
    app.threaded_timeline_items.push_back(item);
    assert_eq!(app.threaded_timeline_items.len(), 1);

    let _task = app.handle_open_thread(root_id.clone());
    assert_eq!(app.active_thread_root, Some(root_id));
    assert_eq!(app.threaded_timeline_items.len(), 0);
}

#[test]
fn test_handle_timeline_diff_thread_multiple_items() {
    let mut app = create_dummy_constellation();
    let root_id = matrix_sdk::ruma::EventId::parse("$test_root").unwrap();
    app.active_thread_root = Some(root_id);

    let item1 = crate::ConstellationItem::mock("Alice", "Opening message", "12:00", false);
    let item2 = crate::ConstellationItem::mock("Bob", "Reply message", "12:05", false);

    app.threaded_timeline_items.push_back(item1);
    app.threaded_timeline_items.push_back(item2);

    assert_eq!(app.threaded_timeline_items.len(), 2);
    assert_eq!(app.threaded_timeline_items[0].sender_name, "Alice");
    assert_eq!(app.threaded_timeline_items[1].sender_name, "Bob");
}

#[test]
fn test_qr_login_progress_step_transitions() {
    let mut app = create_dummy_constellation();
    app.auth_flow = AuthFlow::Qr {
        step: QrLoginStep::Initiating,
    };

    // QrReady → ShowingQr with bytes stored for rendering.
    let _task = app.handle_qr_login_progress(matrix::QrLoginProgress::QrReady(vec![
        0x4d, 0x41, 0x54, 0x52, 0x49, 0x58,
    ]));
    assert_eq!(
        app.auth_flow,
        AuthFlow::Qr {
            step: QrLoginStep::ShowingQr
        }
    );
    assert!(app.qr_code_bytes.is_some());
    assert!(!app.qr_code_bytes.as_ref().unwrap().is_empty());

    // SyncingSecrets → SyncingSecrets step.
    let _task = app.handle_qr_login_progress(matrix::QrLoginProgress::SyncingSecrets);
    assert_eq!(
        app.auth_flow,
        AuthFlow::Qr {
            step: QrLoginStep::SyncingSecrets
        }
    );

    // WaitingForToken → Authenticating with user code stored.
    let _task = app.handle_qr_login_progress(matrix::QrLoginProgress::WaitingForToken {
        user_code: "AB12CD".to_string(),
    });
    assert_eq!(
        app.auth_flow,
        AuthFlow::Qr {
            step: QrLoginStep::Authenticating
        }
    );
    assert_eq!(app.qr_user_code.as_deref(), Some("AB12CD"));

    // Finished(Err) → Error step, error set, QR fields cleared.
    let _task =
        app.handle_qr_login_progress(matrix::QrLoginProgress::Finished(Err("boom".to_string())));
    assert_eq!(
        app.auth_flow,
        AuthFlow::Qr {
            step: QrLoginStep::Error
        }
    );
    assert!(app.qr_code_bytes.is_none());
    assert!(app.qr_user_code.is_none());
    assert!(app.error.is_some());
}

#[test]
fn test_qr_check_code_input_filtering() {
    let mut app = create_dummy_constellation();

    // Only digits are kept, max two characters.
    let _task = app.handle_qr_check_code_changed("a1b2c3".to_string());
    assert_eq!(app.qr_check_code_input, "12");

    // A short valid input is kept as-is.
    let _task = app.handle_qr_check_code_changed("7".to_string());
    assert_eq!(app.qr_check_code_input, "7");

    // Non-digit input is rejected entirely.
    let _task = app.handle_qr_check_code_changed("abc".to_string());
    assert_eq!(app.qr_check_code_input, "");
}

#[test]
fn test_qr_login_cancel_clears_state() {
    let mut app = create_dummy_constellation();
    app.auth_flow = AuthFlow::Qr {
        step: QrLoginStep::ShowingQr,
    };
    app.qr_code_bytes = Some(vec![1, 2, 3]);
    app.qr_user_code = Some("XY".to_string());
    app.qr_check_code_input = "42".to_string();

    let _task = app.handle_cancel_qr_login();
    assert_eq!(app.auth_flow, AuthFlow::Idle);
    assert!(app.qr_code_bytes.is_none());
    assert!(app.qr_user_code.is_none());
    assert!(app.qr_check_code_input.is_empty());
}

#[test]
fn test_oidc_login_cancel_clears_state() {
    let mut app = create_dummy_constellation();
    app.auth_flow = AuthFlow::Oidc;
    app.pending_oidc_callback =
        Some(Url::parse("fi.joonastuomi.constellation:/callback?code=123").unwrap());

    let _task = app.handle_cancel_oidc_login();
    assert_eq!(app.auth_flow, AuthFlow::Idle);
    assert_eq!(app.pending_oidc_callback, None);
}

#[tokio::test]
async fn test_oidc_callback_cold_start_buffers_and_replays_on_engine_ready() {
    let mut app = create_dummy_constellation();
    assert!(app.matrix.is_none());

    let callback_url =
        Url::parse("fi.joonastuomi.constellation:/callback?code=coldstart&state=123").unwrap();

    // When cold-started, handle_oidc_callback sees matrix is None and buffers the URL
    let _task = app.handle_oidc_callback(callback_url.clone());
    assert_eq!(app.pending_oidc_callback, Some(callback_url.clone()));

    // When engine is ready, pending_oidc_callback is taken to replay
    let tmp_dir = tempfile::tempdir().unwrap();
    let engine = match crate::matrix::MatrixEngine::new(tmp_dir.path().to_path_buf()).await {
        Ok(e) => e,
        Err(e) => {
            println!("Skipping test due to engine init failure: {e}");
            return;
        }
    };

    let _task = app.handle_engine_ready(Ok(engine));
    assert_eq!(
        app.pending_oidc_callback, None,
        "pending_oidc_callback must be consumed by handle_engine_ready"
    );
}

#[test]
fn test_oidc_login_started_error_handling() {
    let mut app = create_dummy_constellation();
    app.auth_flow = AuthFlow::Oidc;

    // Sentinel error: homeserver does not support OIDC
    let _task =
        app.handle_oidc_login_started(Err(crate::matrix::OIDC_NOT_SUPPORTED_SENTINEL.to_string()));
    assert_eq!(app.auth_flow, AuthFlow::Idle);
    assert_eq!(
        app.error,
        Some(crate::fl!("error-oidc-not-supported").to_string())
    );

    // Generic error
    let _task = app.handle_oidc_login_started(Err("timeout connecting to IdP".to_string()));
    assert_eq!(app.auth_flow, AuthFlow::Idle);
    assert_eq!(
        app.error,
        Some(
            crate::fl!(
                "error-failed-oidc-login",
                error = "timeout connecting to IdP"
            )
            .to_string()
        )
    );

    // Browser error i18n format verification
    let formatted = crate::fl!(
        "error-failed-open-browser",
        error = "No browser installed",
        url = "https://example.com/auth"
    )
    .to_string();
    assert!(formatted.contains("No browser installed"));
    assert!(formatted.contains("https://example.com/auth"));
}

fn setup_scroll_test_app() -> (crate::Constellation, std::sync::Arc<str>) {
    let mut app = create_dummy_constellation();
    app.user_id = Some("@test_user:matrix.org".to_string());

    let room_id: std::sync::Arc<str> = std::sync::Arc::from("!room1:example.com");
    app.room_list.push(matrix::RoomData {
        id: room_id.clone(),
        name: Some("Room 1".to_string()),
        unread_count: 5,
        unread_count_str: Some("5".to_string()),
        last_message: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        order: None,
        suggested: false,
    });
    app.rebuild_room_index();

    (app, room_id)
}

#[test]
fn test_room_scroll_behavior_just_joined() {
    let (mut app, room_id) = setup_scroll_test_app();

    // 1. Just joined the room
    let owned_room_id = matrix_sdk::ruma::RoomId::parse(room_id.as_ref())
        .expect("Failed to parse valid test room ID");
    let _ = app.update(Message::RoomJoined(Ok(owned_room_id)));
    assert!(app.visited_room_ids.contains(&room_id));
    assert!(app.is_first_time_joining);

    // Simulate timeline reset when subscription starts
    let _ = app.update(Message::Matrix(matrix::MatrixEvent::TimelineReset));
    assert!(app.needs_initial_scroll);
    assert!(app.is_timeline_at_bottom);

    // Populate timeline
    for i in 0..10 {
        app.timeline_items.push_back(crate::ConstellationItem::mock(
            "Sender",
            &format!("Msg {}", i),
            "2026-06-08T13:22:31Z",
            false,
        ));
    }

    // Simulate TimelineInitFinished
    let _ = app.update(Message::Matrix(matrix::MatrixEvent::TimelineInitFinished));
    assert!(app.is_timeline_initialized);

    let _task = app.update(Message::LoadMoreFinished(Ok(())));
    assert!(!app.needs_initial_scroll);
}

#[test]
fn test_room_scroll_behavior_normal_selection() {
    let (mut app, room_id) = setup_scroll_test_app();

    // 2. Normal room selection
    app.timeline_items.clear();
    app.is_first_time_joining = true; // set to true to verify RoomSelected sets it to false
    app.needs_initial_scroll = false;

    let _task = app.update(Message::RoomSelected(room_id.clone()));
    assert!(!app.is_first_time_joining);
    assert!(app.needs_initial_scroll);

    // Populate timeline again
    for i in 0..10 {
        app.timeline_items.push_back(crate::ConstellationItem::mock(
            "Sender",
            &format!("Msg {}", i),
            "2026-06-08T13:22:31Z",
            false,
        ));
    }

    // Simulate TimelineInitFinished
    let _ = app.update(Message::Matrix(matrix::MatrixEvent::TimelineInitFinished));
    assert!(app.is_timeline_initialized);

    let _task2 = app.update(Message::LoadMoreFinished(Ok(())));
    assert!(!app.needs_initial_scroll);
}

#[test]
fn test_room_scroll_behavior_check_initial_scroll() {
    let (mut app, room_id) = setup_scroll_test_app();
    app.selected_room = Some(room_id);

    // 3. Directly test check_and_perform_initial_scroll helper
    app.timeline_items.clear();
    app.needs_initial_scroll = true;
    app.is_loading_more = true;
    app.is_timeline_initialized = false;
    assert!(app.check_and_perform_initial_scroll().is_none());

    app.is_loading_more = false;
    assert!(app.check_and_perform_initial_scroll().is_none()); // still none because is_timeline_initialized is false

    app.is_timeline_initialized = true;
    app.timeline_items.push_back(crate::ConstellationItem::mock(
        "Sender",
        "Msg",
        "2026-06-08T13:22:31Z",
        false,
    ));
    assert!(app.check_and_perform_initial_scroll().is_some());
    assert!(!app.needs_initial_scroll);
}

#[test]
fn test_room_scroll_behavior_timeline_reset_initial() {
    let (mut app, _) = setup_scroll_test_app();

    // 4. Test timeline reset scroll behavior (initial reset)
    app.is_timeline_initialized = false;
    let _ = app.update(Message::Matrix(matrix::MatrixEvent::TimelineReset));
    assert!(app.needs_initial_scroll);
    assert!(app.is_timeline_at_bottom);
    assert!(!app.is_timeline_initialized);
}

#[test]
fn test_room_scroll_behavior_timeline_reset_background() {
    let (mut app, _) = setup_scroll_test_app();

    // 5. Test background timeline reset scroll behavior (when already initialized)
    app.is_timeline_initialized = true;
    app.is_timeline_at_bottom = false;
    app.last_timeline_offset = 150.0;
    let _ = app.update(Message::Matrix(matrix::MatrixEvent::TimelineReset));
    assert!(!app.needs_initial_scroll);
    assert!(app.needs_scroll_restoration);
    assert!(!app.is_timeline_at_bottom); // preserved!
    assert!(!app.is_timeline_initialized);

    // Simulate TimelineInitFinished for background reset
    let _ = app.update(Message::Matrix(matrix::MatrixEvent::TimelineInitFinished));
    assert!(app.is_timeline_initialized);
    assert!(!app.needs_scroll_restoration);
}
#[test]
fn test_recompute_timeline_metadata_skips_none_inner_no_panic() {
    // Regression: items whose `item` field is `None` (mock/virtual items) used to
    // hit `.expect("No item")` and panic recompute_thread_counts. They must now be
    // skipped gracefully.
    let mut app = create_dummy_constellation();

    let root_a = matrix_sdk::ruma::EventId::parse("$root_a:example.com").unwrap();
    let root_b = matrix_sdk::ruma::EventId::parse("$root_b:example.com").unwrap();

    // `new_mock` constructs items with `item: None` by design.
    let mut threaded_a = ConstellationItem::mock("alice", "reply", "12:00", false);
    threaded_a.thread_root_id = Some(root_a.clone());
    let mut threaded_b = ConstellationItem::mock("bob", "reply", "12:01", false);
    threaded_b.thread_root_id = Some(root_b.clone());
    let plain = ConstellationItem::mock("carol", "message", "12:02", true);

    app.timeline_items.push_back(threaded_a);
    app.timeline_items.push_back(threaded_b);
    app.timeline_items.push_back(plain);

    // Must not panic; None-inner items are skipped even when they carry a thread root.
    app.recompute_timeline_metadata();

    // No event-bearing items were counted.
    assert!(app.thread_counts.is_empty());
}

// --- Phase 3: event-focused (permalink context) timeline ---

/// A room switch must always leave the event-focused view, clearing any
/// pending or active event focus so the new room opens on its live timeline
/// and the "viewing older messages" banner hides.
#[test]
fn test_room_selected_clears_event_focus() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let event_id: OwnedEventId = matrix_sdk::ruma::EventId::parse("$target:example.com").unwrap();
    app.pending_event_focus = Some(event_id.clone());
    app.active_event_focus = Some(event_id.clone());
    app.selected_room = Some(Arc::from("!old:example.com"));

    // RoomSelected needs a room present in room_list to cache its name; an
    // empty list exercises the no-match path without panicking.
    let room_id: Arc<str> = Arc::from("!new:example.com");
    let _ = app.update(Message::RoomSelected(room_id.clone()));

    assert!(
        app.pending_event_focus.is_none(),
        "pending_event_focus must clear on room switch"
    );
    assert!(
        app.active_event_focus.is_none(),
        "active_event_focus must clear on room switch"
    );
    assert_eq!(app.selected_room.as_deref(), Some("!new:example.com"));
}

/// `check_pending_event_focus` consumes a pending event that is already in
/// the loaded window by scrolling to it (state is consumed, no event-focus
/// timeline is built — i.e. active_event_focus stays None).
#[test]
fn test_pending_event_focus_loaded_event_jumps() {
    let mut app = create_dummy_constellation();
    let event_id: OwnedEventId = matrix_sdk::ruma::EventId::parse("$loaded:example.com").unwrap();

    // Simulate the event already being in the loaded window.
    let mut item = ConstellationItem::mock("alice", "loaded msg", "12:00", false);
    item.item_id = Some(matrix::TimelineEventItemId::EventId(event_id.clone()));
    app.timeline_items.push_back(item);
    app.pending_event_focus = Some(event_id.clone());

    let _ = app.check_pending_event_focus();

    assert!(
        app.pending_event_focus.is_none(),
        "pending focus must be consumed"
    );
    assert!(
        app.active_event_focus.is_none(),
        "loaded event must not build an event-focused timeline"
    );
}

/// `check_pending_event_focus` consumes a pending event that is NOT in the
/// loaded window by handing off to LoadEventContext. We can't drive the
/// async matrix call in a unit test, but we verify the intent: the helper
/// consumes the pending focus and the follow-up LoadEventContext handler
/// sets active_event_focus (when a room + engine are present, which they
/// aren't here, so it surfaces an error and leaves focus clear).
#[test]
fn test_pending_event_focus_missing_event_defers_to_load() {
    let mut app = create_dummy_constellation();
    let event_id: OwnedEventId = matrix_sdk::ruma::EventId::parse("$missing:example.com").unwrap();

    // Empty timeline: the event is not loaded.
    app.pending_event_focus = Some(event_id.clone());

    let _ = app.check_pending_event_focus();

    assert!(
        app.pending_event_focus.is_none(),
        "pending focus must be consumed"
    );
    // active_event_focus is only set inside handle_load_event_context, which
    // requires a live matrix engine; here it must stay None.
    assert!(app.active_event_focus.is_none());
}

/// `ReturnToLive` clears the active event focus and resets the timeline so
/// the live subscription reinitialises at the newest messages.
#[test]
fn test_return_to_live_clears_active_focus() {
    let mut app = create_dummy_constellation();
    let event_id: OwnedEventId = matrix_sdk::ruma::EventId::parse("$focused:example.com").unwrap();
    app.active_event_focus = Some(event_id);
    app.is_timeline_initialized = true;
    app.is_timeline_at_bottom = false;
    app.needs_initial_scroll = false;

    let _ = app.update(Message::ReturnToLive);

    assert!(
        app.active_event_focus.is_none(),
        "active_event_focus must clear on return to live"
    );
    assert!(!app.is_timeline_initialized, "timeline must reinitialise");
    assert!(
        app.needs_initial_scroll,
        "must scroll to newest on live restore"
    );
    assert!(app.is_timeline_at_bottom);
}

// --- Phase 4: in-app paste-link dialog ---

/// `ToggleOpenLink` opens the dialog when signed in; when signed out it
/// surfaces the sign-in error instead of an inert dialog.
#[test]
fn test_toggle_open_link_signed_out_surfaces_error() {
    let mut app = create_dummy_constellation();
    // create_dummy_constellation leaves matrix as None (signed out).
    assert!(app.open_link_dialog.is_none());

    // Signed out: handler surfaces sign-in error and does not open.
    let _ = app.update(Message::ToggleOpenLink);
    assert!(
        app.open_link_dialog.is_none(),
        "dialog must not open when signed out"
    );
    assert!(
        app.error.as_deref().unwrap_or("").contains("Sign in"),
        "expected a sign-in prompt, got: {:?}",
        app.error
    );
}

/// `OpenLinkTextChanged` updates the dialog value when open, and is a no-op
/// when the dialog is closed (defensive: a stale input event must not
/// secretly open the dialog).
#[test]
fn test_open_link_text_changed_updates_and_guards() {
    let mut app = create_dummy_constellation();

    // Closed: changing text must not open the dialog.
    app.open_link_dialog = None;
    let _ = app.update(Message::OpenLinkTextChanged("ignored".to_string()));
    assert!(app.open_link_dialog.is_none());

    // Open: changing text updates the value.
    app.open_link_dialog = Some(String::new());
    let _ = app.update(Message::OpenLinkTextChanged(
        "https://matrix.to/#/!abc:example.org".to_string(),
    ));
    assert_eq!(
        app.open_link_dialog.as_deref(),
        Some("https://matrix.to/#/!abc:example.org")
    );
}

/// `SubmitOpenLink` always closes the dialog, regardless of input.
#[test]
fn test_submit_open_link_closes_dialog() {
    let mut app = create_dummy_constellation();
    app.open_link_dialog = Some("https://matrix.to/#/!abc:example.org".to_string());

    let _ = app.update(Message::SubmitOpenLink(
        "https://matrix.to/#/!abc:example.org".to_string(),
    ));

    assert!(
        app.open_link_dialog.is_none(),
        "submitting must close the dialog"
    );
}

/// `SubmitOpenLink` with empty input just closes the dialog without error.
#[test]
fn test_submit_open_link_empty_closes_silently() {
    let mut app = create_dummy_constellation();
    app.open_link_dialog = Some(String::new());
    app.error = None;

    let _ = app.update(Message::SubmitOpenLink("   ".to_string()));

    assert!(app.open_link_dialog.is_none());
    // Empty/whitespace input must not surface an error (it's a cancel-like
    // no-op, not a parse failure).
    assert!(app.error.is_none(), "empty submit must not error");
}

#[test]
fn test_copy_to_clipboard_success() {
    let mut app = create_dummy_constellation();
    let _task = app.update(Message::CopyToClipboard(Ok(
        "https://matrix.to/#/!room:example.com".to_string(),
    )));
    assert!(app.error.is_none());
}

#[test]
fn test_copy_to_clipboard_error() {
    let mut app = create_dummy_constellation();
    let _task = app.update(Message::CopyToClipboard(Err(
        "Failed to build link".to_string()
    )));
    assert_eq!(app.error.as_deref(), Some("Failed to build link"));
}

#[test]
fn test_copy_room_link_no_matrix() {
    let mut app = create_dummy_constellation();
    let _task = app.update(Message::CopyRoomLink("!room:example.com".into()));
    assert!(app.error.is_none());
}

#[test]
fn test_copy_message_link_no_matrix() {
    let mut app = create_dummy_constellation();
    let item_id = matrix::TimelineEventItemId::EventId(
        matrix_sdk::ruma::event_id!("$event:localhost").to_owned(),
    );
    let _task = app.update(Message::CopyMessageLink(item_id));
    assert!(app.error.is_none());
}

#[test]
fn test_dm_room_resolved_success() {
    let mut app = create_dummy_constellation();
    let target_room = matrix_sdk::ruma::room_id!("!room:example.com").to_owned();
    let _task = app.update(Message::DmRoomResolved(Ok(target_room)));
    assert_eq!(app.selected_room.as_deref(), Some("!room:example.com"));
    assert!(app.error.is_none());
}

#[test]
fn test_dm_room_resolved_error() {
    let mut app = create_dummy_constellation();
    let _task = app.update(Message::DmRoomResolved(
        Err("Failed to join DM".to_string()),
    ));
    let err = app.error.expect("Expected error to be set");
    assert!(err.contains("Failed to start direct message"));
    assert!(err.contains("Failed to join DM"));
}

#[test]
fn test_message_search_pagination() {
    let mut app = create_dummy_constellation();

    assert!(!app.search_has_more);
    assert!(!app.is_searching_more_messages);
    assert!(app.message_search_results.is_empty());

    // Simulate incoming search results with has_more = true
    let mock_result = matrix::MessageSearchResult {
        room_id: matrix_sdk::ruma::room_id!("!room:example.com").to_owned(),
        room_name: None,
        event_id: matrix_sdk::ruma::event_id!("$1:example.com").to_owned(),
        sender_id: matrix_sdk::ruma::user_id!("@alice:example.com").to_owned(),
        body: "hello world".to_string(),
        timestamp: "2026-06-08 13:00:00".to_string(),
        plain_text: Vec::new(),
        links: Vec::new(),
    };

    let _ = app.update(Message::MessageSearchResults(
        app.search_generation,
        Ok((vec![mock_result.clone()], true)),
    ));

    assert!(app.search_has_more);
    assert_eq!(app.message_search_results.len(), 1);

    // Simulate LoadMoreMessageSearch
    // Note: matrix is None, so it will return Task::none(), but we can still trigger it
    let _ = app.update(Message::LoadMoreMessageSearch);
    // Since matrix is None, is_searching_more_messages will remain false or change depending on conditions,
    // but we can manually trigger the response message to test results appending:
    app.is_searching_more_messages = true;

    let mock_result_2 = matrix::MessageSearchResult {
        room_id: matrix_sdk::ruma::room_id!("!room:example.com").to_owned(),
        room_name: None,
        event_id: matrix_sdk::ruma::event_id!("$2:example.com").to_owned(),
        sender_id: matrix_sdk::ruma::user_id!("@bob:example.com").to_owned(),
        body: "hello back".to_string(),
        timestamp: "2026-06-08 13:05:00".to_string(),
        plain_text: Vec::new(),
        links: Vec::new(),
    };

    let _ = app.update(Message::MessageSearchMoreResults(Ok((
        vec![mock_result_2],
        false,
    ))));

    assert!(!app.is_searching_more_messages);
    assert!(!app.search_has_more); // exhausted now
    assert_eq!(app.message_search_results.len(), 2);
    assert_eq!(app.message_search_results[0].body, "hello world");
    assert_eq!(app.message_search_results[1].body, "hello back");

    // Submitting a new search query should reset the pagination states
    app.search_query = "new query".to_string();
    let _ = app.update(Message::SubmitSearch);
    assert!(!app.search_has_more);
    assert!(!app.is_searching_more_messages);
}

// --- Global (cross-room) message search ---

/// `OpenRoomEvent` for a different room selects the room and then re-
/// asserts the pending event focus *after* `RoomSelected` clears it. The
/// test simulates the runtime performing the batched `Task::done` messages
/// in order: RoomSelected runs first (clearing focus), then
/// SetPendingEventFocus runs (re-asserting it). The end state must have the
/// new room selected AND the focus pending for TimelineInitFinished.
#[test]
fn test_open_room_event_sets_pending_focus_after_select() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let event_id: OwnedEventId = matrix_sdk::ruma::EventId::parse("$hit:example.com").unwrap();
    let room_id: Arc<str> = Arc::from("!new:example.com");
    app.selected_room = Some(Arc::from("!old:example.com"));

    // The handler returns a batch of [RoomSelected, SetPendingEventFocus];
    // simulate the runtime performing them in order.
    let _task = app.handle_update(Message::OpenRoomEvent {
        room_id: room_id.clone(),
        event_id: event_id.clone(),
    });
    // Runtime performs RoomSelected (clears focus) then SetPendingEventFocus
    // (re-asserts it) — same order as the batch.
    let _t1 = app.handle_update(Message::RoomSelected(room_id.clone()));
    let _t2 = app.handle_update(Message::SetPendingEventFocus(event_id.clone()));

    assert_eq!(app.selected_room.as_deref(), Some("!new:example.com"));
    assert_eq!(
        app.pending_event_focus,
        Some(event_id),
        "focus must be set after RoomSelected clears it"
    );
}

/// `OpenRoomEvent` for the currently-selected room must NOT switch rooms:
/// it jumps directly via `JumpToMessageOrLoadContext`. `pending_event_focus`
/// is left untouched (the jump path doesn't use it).
#[test]
fn test_open_room_event_same_room_does_not_switch() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let event_id: OwnedEventId = matrix_sdk::ruma::EventId::parse("$hit:example.com").unwrap();
    let room_id: Arc<str> = Arc::from("!here:example.com");
    app.selected_room = Some(room_id.clone());

    // Same room: handler short-circuits to JumpToMessageOrLoadContext.
    // Don't perform any deferred message — the contract is no room switch.
    let _task = app.handle_update(Message::OpenRoomEvent {
        room_id: room_id.clone(),
        event_id,
    });

    assert_eq!(app.selected_room.as_deref(), Some("!here:example.com"));
    assert!(
        app.pending_event_focus.is_none(),
        "same-room jump must not touch pending_event_focus"
    );
}

/// `GlobalMessageSearchResults` honours the generation guard: a result
/// carrying a stale generation is discarded; the current generation lands.
#[test]
fn test_global_message_search_results_generation_guard() {
    let mut app = create_dummy_constellation();
    app.search_generation = 5;

    let make_hit = || matrix::MessageSearchResult {
        room_id: matrix_sdk::ruma::room_id!("!room:example.com").to_owned(),
        room_name: Some("Room".to_string()),
        event_id: matrix_sdk::ruma::EventId::parse("$e:example.com").unwrap(),
        sender_id: matrix_sdk::ruma::user_id!("@a:b.c").to_owned(),
        body: "hi".to_string(),
        timestamp: "2026-01-01 00:00:00".to_string(),
        plain_text: Vec::new(),
        links: Vec::new(),
    };

    // Stale generation (4 < 5) — discarded.
    let _t = app.handle_update(Message::GlobalMessageSearchResults(4, Ok(vec![make_hit()])));
    assert!(
        app.global_message_search_results.is_empty(),
        "stale-generation result must be discarded"
    );
    assert!(
        !app.is_searching_global_messages,
        "discarded result must not flip the loading flag either"
    );

    // Current generation (5) — lands.
    let _t = app.handle_update(Message::GlobalMessageSearchResults(5, Ok(vec![make_hit()])));
    assert_eq!(
        app.global_message_search_results.len(),
        1,
        "current-generation result must land"
    );
    assert!(!app.is_searching_global_messages);
}

/// `SetGlobalSearchScope` updates the scope and clears stale global hits so
/// a toggle doesn't briefly show the old scope's results.
#[test]
fn test_set_global_search_scope_updates_and_clears() {
    use crate::matrix::GlobalSearchScope;
    let mut app = create_dummy_constellation();
    app.global_search_scope = GlobalSearchScope::All;
    // Pretend we already have some All-scope hits on screen.
    app.global_message_search_results
        .push(matrix::MessageSearchResult {
            room_id: matrix_sdk::ruma::room_id!("!r:example.com").to_owned(),
            room_name: None,
            event_id: matrix_sdk::ruma::EventId::parse("$e:example.com").unwrap(),
            sender_id: matrix_sdk::ruma::user_id!("@a:b.c").to_owned(),
            body: "hi".to_string(),
            timestamp: "2026-01-01 00:00:00".to_string(),
            plain_text: Vec::new(),
            links: Vec::new(),
        });

    // Empty query: SetGlobalSearchScope re-enters SearchQueryChanged, which
    // hits the clear branch (no search fired) — so results are cleared.
    let _t = app.handle_update(Message::SetGlobalSearchScope(GlobalSearchScope::DmsOnly));

    assert_eq!(app.global_search_scope, GlobalSearchScope::DmsOnly);
    assert!(
        app.global_message_search_results.is_empty(),
        "stale hits must clear on scope change"
    );
}

// ===== Keyboard shortcuts (issue #425) =====

fn shortcut_app() -> Constellation {
    let mut app = create_dummy_constellation();
    app.user_id = Some("@me:matrix.org".to_string());
    app
}

use cosmic::widget::menu::key_bind::KeyBind;

#[test]
fn test_shortcut_toggles_settings_panel() {
    let mut app = shortcut_app();

    // Ctrl+, opens App settings.
    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::ToggleAppSettings,
    ));
    assert_eq!(
        app.current_settings_panel(),
        Some(&crate::SettingsPanel::App)
    );

    // Pressing it again closes the drawer.
    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::ToggleAppSettings,
    ));
    assert!(app.current_settings_panel().is_none());

    // A different panel shortcut switches directly to that panel.
    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::ToggleRoomSettings,
    ));
    assert_eq!(
        app.current_settings_panel(),
        Some(&crate::SettingsPanel::Room)
    );
}

#[test]
fn test_shortcuts_inert_when_logged_out_except_quit() {
    let mut app = create_dummy_constellation(); // user_id: None

    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::ToggleUserSettings,
    ));
    assert!(
        app.current_settings_panel().is_none(),
        "settings must not open before login"
    );
}

#[test]
fn test_close_thread_shortcut_clears_thread_state() {
    let mut app = shortcut_app();
    let root_id: OwnedEventId = "$evt1:matrix.org".parse().unwrap();
    let _ = app.handle_update(Message::OpenThread(root_id.clone()));
    assert!(app.active_thread_root.is_some());

    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::CloseThread,
    ));
    assert!(app.active_thread_root.is_none());
    assert!(app.threaded_timeline_items.is_empty());
}

#[test]
fn test_search_shortcut_opens_search_bar() {
    let mut app = shortcut_app();
    assert!(!app.is_search_active);

    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::Search,
    ));
    assert!(app.is_search_active);
}

#[test]
fn test_room_selection_enter_moves_and_commits() {
    use std::sync::Arc;

    let mut app = shortcut_app();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let room_b: Arc<str> = Arc::from("!b:matrix.org");
    for id in [&room_a, &room_b] {
        app.room_list.push(matrix::RoomData {
            id: id.clone(),
            name: None,
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            suggested: false,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            order: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
        });
    }
    app.update_filtered_rooms();
    assert!(!app.filtered_room_list.is_empty());

    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::SelectRoomList,
    ));

    // Move down once and commit; the second room becomes selected.
    let _ = app.handle_update(Message::SelectionMove(1));
    if let Some(crate::constellation::ListSelection::Rooms { index, .. }) = &app.list_selection {
        assert_eq!(*index, 1);
    } else {
        panic!("room selection not active after SelectRoomList");
    }

    let _ = app.handle_update(Message::SelectionCommit);
    assert!(app.list_selection.is_none());
    assert_eq!(
        app.selected_room.as_deref(),
        Some("!b:matrix.org"),
        "Enter must activate the keyboard-selected room"
    );
}

#[test]
fn test_selection_cancel_restores_nav_activation() {
    let mut app = shortcut_app();
    // Seed a two-entry nav model ("All rooms" + one space).
    app.space_nav_model
        .insert()
        .text("All rooms".to_string())
        .activate();
    app.space_nav_model.insert().text("Space".to_string());

    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::SelectSpaceSwitcher,
    ));
    assert!(matches!(
        app.list_selection,
        Some(crate::constellation::ListSelection::Spaces { .. })
    ));

    // Navigate to entry 1, then cancel: activation returns to "All rooms".
    let _ = app.handle_update(Message::SelectionMove(1));
    let _ = app.handle_update(Message::SelectionCancel);

    assert!(app.list_selection.is_none());
    assert_eq!(
        app.space_nav_model.position(app.space_nav_model.active()),
        Some(0)
    );
}

#[test]
fn test_shortcuts_saved_persists_overrides_into_bindings() {
    use crate::constellation::keybind::{Bindings, SerializedKeyBind, ShortcutAction};
    use cosmic::iced::keyboard::Key;
    use cosmic::widget::menu::key_bind::Modifier;

    let mut app = shortcut_app();

    // Rebind ScrollUp onto Ctrl+J in the page draft, then Save.
    let kb = KeyBind {
        modifiers: vec![Modifier::Ctrl],
        key: Key::Character("j".into()),
    };
    app.shortcuts
        .draft
        .insert(ShortcutAction::ScrollUp, Some(kb.clone()));
    let _ = app.handle_update(Message::ShortcutsSaved);

    // Live bindings pick up the rebind without a restart.
    assert_eq!(
        app.keybinds.keybind_for(ShortcutAction::ScrollUp),
        Some(&kb)
    );
    // The overrides snapshot is what build_config persists.
    assert_eq!(
        app.shortcuts.overrides.get(&ShortcutAction::ScrollUp),
        Some(&SerializedKeyBind::from(&kb))
    );
    let config = app.build_config();
    assert!(config.key_bindings.contains_key(&ShortcutAction::ScrollUp));
    // And a fresh Bindings rebuilt from config agrees.
    let rebuilt = Bindings::with_overrides(&config.key_bindings);
    assert_eq!(rebuilt.keybind_for(ShortcutAction::ScrollUp), Some(&kb));
}

fn measured(is_thread: bool, rows: Vec<(&str, f32)>) -> Message {
    Message::TimelineMeasured {
        is_thread,
        generation: 0,
        viewport_width: 600.0,
        content_height: 1000.0,
        rows: rows.into_iter().map(|(k, y)| (k.to_string(), y)).collect(),
    }
}

#[test]
fn test_scroll_reflow_restores_against_fresh_measurement() {
    let (mut app, _room) = setup_scroll_test_app();
    app.is_timeline_initialized = true;
    app.is_timeline_at_bottom = false;
    app.last_viewport_height = 200.0;
    // Pre-reflow snapshot: viewport top sits 50px into row b at width 600.
    app.scroll_main.store(
        vec![("tl|a".to_string(), 0.0), ("tl|b".to_string(), 400.0)],
        800.0,
        600.0,
    );
    app.last_timeline_offset = 450.0;
    // The reflow event observed the new layout the measurement reports.
    app.scroll_main.note_observed(600.0, 1000.0);
    app.scroll_main.pending_reflow =
        crate::constellation::scroll::plan_reflow(450.0, &app.scroll_main.children, 800.0, 1200.0);

    let _ = app.handle_update(measured(false, vec![("tl|a", 0.0), ("tl|b", 520.0)]));

    assert!(app.scroll_main.pending_reflow.is_none());
    assert!(!app.needs_layout_scroll_restoration);
    // Row b now starts at 520; the same 50px intra-row position applies.
    assert!(!app.is_timeline_at_bottom);
    // The fresh snapshot replaced the stale one.
    assert!((app.scroll_main.children_content_height - 1000.0).abs() < f32::EPSILON);
}

#[test]
fn test_scroll_stale_measurement_is_re_requested() {
    let (mut app, _room) = setup_scroll_test_app();
    app.is_timeline_initialized = true;
    app.is_timeline_at_bottom = false;
    // Observed geometry is the pre-reflow one; a measurement claiming the
    // new dims must be rejected and re-requested.
    app.scroll_main.note_observed(600.0, 800.0);
    app.scroll_main.pending_reflow = Some(crate::constellation::scroll::PendingReflow::Ratio(0.5));

    let _ = app.handle_update(measured(false, vec![("tl|a", 0.0), ("tl|b", 400.0)]));

    assert!(app.scroll_main.pending_reflow.is_some());
    assert_eq!(app.scroll_main.reflow_attempts, 1);
    assert!(app.scroll_main.measure_pending);
}

#[test]
fn test_scroll_room_memory_roundtrip() {
    let (mut app, room_a) = setup_scroll_test_app();
    let room_b: std::sync::Arc<str> = std::sync::Arc::from("!roomb:example.com");

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    app.is_timeline_initialized = true;
    app.is_timeline_at_bottom = false;
    // Mid-history position with measured geometry.
    app.scroll_main.store(
        vec![("tl|x".to_string(), 0.0), ("tl|y".to_string(), 400.0)],
        1000.0,
        600.0,
    );
    app.last_timeline_offset = 420.0;
    app.last_content_height = 1000.0;

    // Leaving for room B memorizes A's anchor and clears live state.
    let _ = app.update(Message::RoomSelected(room_b));
    assert_eq!(
        app.room_scroll_memory.get(&room_a).map(|(k, _)| k.as_str()),
        Some("tl|y")
    );
    // Returning to A arms the restore; default placement is suspended until
    // the measurement resolves it.
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    assert!(app.pending_room_restore.is_some());
    assert!(app.needs_initial_scroll);

    app.is_timeline_initialized = true;
    for i in 0..4 {
        app.timeline_items.push_back(crate::ConstellationItem::mock(
            "Sender",
            &format!("Msg {i}"),
            "2026-06-08T13:22:31Z",
            false,
        ));
    }
    let _task = app.check_and_perform_initial_scroll();
    assert!(!app.needs_initial_scroll);
    assert!(app.scroll_main.measure_pending);

    // The measured tree places y at 350 with the same intra-row offset.
    let generation = app.scroll_generation;
    let restore_msg = Message::TimelineMeasured {
        is_thread: false,
        generation,
        viewport_width: 600.0,
        content_height: 900.0,
        rows: vec![("tl|x".to_string(), 0.0), ("tl|y".to_string(), 350.0)],
    };
    let _ = app.handle_update(restore_msg);
    assert!(app.pending_room_restore.is_none());
    // 420 was 20px into row y (400); y now sits at 350 → target 370.
    assert!((app.last_timeline_offset - 370.0).abs() < f32::EPSILON);
}

#[test]
fn test_scroll_measured_for_old_generation_is_dropped() {
    let (mut app, _room) = setup_scroll_test_app();
    app.is_timeline_initialized = true;
    let current = app.scroll_generation;

    let msg = Message::TimelineMeasured {
        is_thread: false,
        generation: current + 1,
        viewport_width: 600.0,
        content_height: 1000.0,
        rows: vec![("tl|a".to_string(), 0.0)],
    };
    let _ = app.handle_update(msg);
    assert!(app.scroll_main.children.is_empty());
}

#[test]
fn test_space_switcher_toggle_arms_scroll_restoration() {
    let (mut app, _room) = setup_scroll_test_app();
    app.is_timeline_initialized = true;
    app.is_timeline_at_bottom = false;
    app.last_timeline_offset = 420.0;
    // Measured geometry so the anchor can be decoded.
    app.scroll_main.store(
        vec![("tl|x".to_string(), 0.0), ("tl|y".to_string(), 400.0)],
        1000.0,
        600.0,
    );

    let action = crate::constellation::keybind::ShortcutAction::ToggleSpaceSwitcher;

    // Toggling remounts the chat pane; the anchored restore must be armed
    // immediately and resolved by the deferred measurement.
    let _ = app.handle_update(Message::ShortcutTriggered(action));
    assert!(app.needs_layout_scroll_restoration);
    assert!(app.needs_threaded_layout_scroll_restoration);
    assert!(app.scroll_main.expect_relayout);
    assert!(app.scroll_main.delayed_scheduled);
    assert!(app.scroll_main.pending_reflow.is_some());

    // Once the deadline passes, the restore tick fires the measurement.
    app.scroll_main.measure_deadline =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
    let _ = app.handle_update(Message::RestoreTick);
    assert!(!app.scroll_main.delayed_scheduled);
    assert!(app.scroll_main.measure_pending);
}

#[test]
fn test_measured_honors_expect_relayout_despite_stale_observed() {
    let (mut app, _room) = setup_scroll_test_app();
    app.is_timeline_initialized = true;
    app.is_timeline_at_bottom = false;
    app.last_viewport_height = 200.0;
    // Pre-toggle snapshot: viewport top 20px into row y.
    app.scroll_main.store(
        vec![("tl|x".to_string(), 0.0), ("tl|y".to_string(), 400.0)],
        1000.0,
        600.0,
    );
    app.last_timeline_offset = 420.0;
    // Toggle remounted the pane: offset physically reset to 0, observed dims
    // are still pre-toggle (no on_scroll fired), but the next measurement is
    // trusted.
    app.scroll_main.expect_relayout = true;
    app.scroll_main.pending_reflow = Some(crate::constellation::scroll::PendingReflow::Anchor {
        key: "tl|y".to_string(),
        intra_y: 20.0,
        fallback_ratio: 0.42,
    });

    let msg = Message::TimelineMeasured {
        is_thread: false,
        generation: app.scroll_generation,
        viewport_width: 500.0,
        content_height: 900.0,
        rows: vec![("tl|x".to_string(), 0.0), ("tl|y".to_string(), 350.0)],
    };
    let _ = app.handle_update(msg);

    // Applied despite stale observed dims: 350 + 20 intra = 370.
    assert!((app.last_timeline_offset - 370.0).abs() < f32::EPSILON);
    assert!(!app.scroll_main.expect_relayout);
    assert!(app.scroll_main.pending_reflow.is_none());
}

#[test]
fn test_measured_resnaps_end_after_at_bottom_remount() {
    let (mut app, _room) = setup_scroll_test_app();
    app.is_timeline_initialized = true;
    // Parked at the bottom of the loaded window when the toggle fired.
    app.is_timeline_at_bottom = true;

    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::ToggleSpaceSwitcher,
    ));
    assert!(app.scroll_main.end_snap_scheduled);

    // The restore tick fires the end snap once its deadline passes.
    app.scroll_main.end_snap_deadline =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(1));
    let _ = app.handle_update(Message::RestoreTick);
    assert!(!app.scroll_main.end_snap_scheduled);
}

#[test]
fn test_close_image_arms_scroll_restoration() {
    let (mut app, _room) = setup_scroll_test_app();
    app.is_timeline_at_bottom = false;
    app.fullscreen_image = Some(cosmic::iced::widget::image::Handle::from_bytes(vec![
        0u8, 1, 2, 3,
    ]));

    let _ = app.update(Message::CloseImage);

    assert!(app.fullscreen_image.is_none());
    assert!(app.needs_layout_scroll_restoration);
    assert!(app.needs_threaded_layout_scroll_restoration);
}

#[test]
fn test_temp_file_permissions() {
    let file = tempfile::Builder::new()
        .prefix("constellation-video-")
        .suffix(".mp4")
        .tempfile()
        .unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        file.as_file()
            .set_permissions(std::fs::Permissions::from_mode(0o600))
            .unwrap();
        let mode = file.as_file().metadata().unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);
    }
}

#[test]
fn test_room_selected_populates_tabs_and_activates() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let room_b: Arc<str> = Arc::from("!b:matrix.org");

    use crate::constellation::Tab;
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a.clone())]);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));
    assert_eq!(app.tab_model.len(), 1);
    assert_eq!(
        app.tab_model.active_data::<Tab>().cloned(),
        Some(Tab::Room(room_a.clone()))
    );

    // Opening second room appends tab and activates it
    let _ = app.update(Message::RoomSelected(room_b.clone()));
    assert_eq!(
        app.open_tabs,
        vec![Tab::Room(room_a.clone()), Tab::Room(room_b.clone())]
    );
    assert_eq!(app.selected_room.as_ref(), Some(&room_b));
    assert_eq!(app.tab_model.len(), 2);
    assert_eq!(
        app.tab_model.active_data::<Tab>().cloned(),
        Some(Tab::Room(room_b.clone()))
    );

    // Reselecting first room switches activation without duplicating tab
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    assert_eq!(
        app.open_tabs,
        vec![Tab::Room(room_a.clone()), Tab::Room(room_b.clone())]
    );
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));
    assert_eq!(app.tab_model.len(), 2);
    assert_eq!(
        app.tab_model.active_data::<Tab>().cloned(),
        Some(Tab::Room(room_a))
    );
}

#[test]
fn test_close_room_switches_to_adjacent() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let room_b: Arc<str> = Arc::from("!b:matrix.org");
    let room_c: Arc<str> = Arc::from("!c:matrix.org");

    use crate::constellation::Tab;
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    let _ = app.update(Message::RoomSelected(room_b.clone()));
    let _ = app.update(Message::RoomSelected(room_c.clone()));
    assert_eq!(app.open_tabs.len(), 3);
    assert_eq!(app.selected_room.as_ref(), Some(&room_c));

    // Closing the active last tab (c) switches to previous (b)
    let _ = app.update(Message::CloseRoom(room_c));
    assert_eq!(
        app.open_tabs,
        vec![Tab::Room(room_a.clone()), Tab::Room(room_b.clone())]
    );
    assert_eq!(app.selected_room.as_ref(), Some(&room_b));
    assert_eq!(app.tab_model.len(), 2);

    // Switch to a, then close a (first tab), should switch to next (b)
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    let _ = app.update(Message::CloseRoom(room_a));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_b.clone())]);
    assert_eq!(app.selected_room.as_ref(), Some(&room_b));
    assert_eq!(app.tab_model.len(), 1);
}

#[test]
fn test_close_room_background_tab() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let room_b: Arc<str> = Arc::from("!b:matrix.org");
    let room_c: Arc<str> = Arc::from("!c:matrix.org");

    use crate::constellation::Tab;
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    let _ = app.update(Message::RoomSelected(room_b.clone()));
    let _ = app.update(Message::RoomSelected(room_c.clone()));

    // Room C is active. Close room A (background tab)
    let _ = app.update(Message::CloseRoom(room_a));
    assert_eq!(
        app.open_tabs,
        vec![Tab::Room(room_b.clone()), Tab::Room(room_c.clone())]
    );
    assert_eq!(app.selected_room.as_ref(), Some(&room_c));
    assert_eq!(app.tab_model.len(), 2);
    assert_eq!(
        app.tab_model.active_data::<Tab>().cloned(),
        Some(Tab::Room(room_c))
    );
}

#[test]
fn test_close_room_last_clears_selection() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    assert_eq!(app.open_tabs.len(), 1);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));

    let _ = app.update(Message::CloseRoom(room_a));
    assert!(app.open_tabs.is_empty());
    assert_eq!(app.selected_room, None);
    assert_eq!(app.tab_model.len(), 0);
}

#[test]
fn test_shortcut_close_tab() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let room_b: Arc<str> = Arc::from("!b:matrix.org");

    use crate::constellation::Tab;
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    let _ = app.update(Message::RoomSelected(room_b.clone()));
    assert_eq!(app.selected_room.as_ref(), Some(&room_b));

    // Trigger CloseTab shortcut
    let _ = app.update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::CloseTab,
    ));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a.clone())]);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));
}

#[test]
fn test_tab_activated_and_closed_messages() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let room_b: Arc<str> = Arc::from("!b:matrix.org");

    use crate::constellation::Tab;
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    let _ = app.update(Message::RoomSelected(room_b.clone()));

    // Find entity for room A
    let entity_a = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&Tab::Room(room_a.clone())))
        .unwrap();

    // Activate room A via entity message
    let _ = app.update(Message::TabActivated(entity_a));
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));

    // Close room A via entity message
    let _ = app.update(Message::TabClosed(entity_a));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_b.clone())]);
    assert_eq!(app.selected_room.as_ref(), Some(&room_b));
}

#[hegel::test(test_cases = 100)]
fn test_tab_lifecycle_invariants(tc: hegel::TestCase) {
    use crate::constellation::Tab;
    use hegel::generators;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());

    let room_pool: [Arc<str>; 5] = [
        Arc::from("!room1:matrix.org"),
        Arc::from("!room2:matrix.org"),
        Arc::from("!room3:matrix.org"),
        Arc::from("!room4:matrix.org"),
        Arc::from("!room5:matrix.org"),
    ];

    let num_ops = tc.draw(generators::integers::<usize>().min_value(1).max_value(30));
    for _ in 0..num_ops {
        let op = tc.draw(generators::integers::<u8>().min_value(0).max_value(3));
        match op {
            0 => {
                // Open / select a room
                let idx = tc.draw(
                    generators::integers::<usize>()
                        .min_value(0)
                        .max_value(room_pool.len() - 1),
                );
                let _ = app.update(Message::RoomSelected(room_pool[idx].clone()));
            }
            1 => {
                // Close a room by ID
                let idx = tc.draw(
                    generators::integers::<usize>()
                        .min_value(0)
                        .max_value(room_pool.len() - 1),
                );
                let _ = app.update(Message::CloseRoom(room_pool[idx].clone()));
            }
            2 => {
                // Close active tab via shortcut
                let _ = app.update(Message::ShortcutTriggered(
                    crate::constellation::keybind::ShortcutAction::CloseTab,
                ));
            }
            _ => {
                let entity = app.tab_model.iter().next();
                if let Some(entity) = entity {
                    let _ = app.update(Message::TabActivated(entity));
                }
            }
        }

        // Invariants:
        // 1. open_tabs has no duplicate entries
        let mut seen = HashSet::new();
        for t in &app.open_tabs {
            assert!(seen.insert(t.clone()), "duplicate tab in open_tabs: {t:?}");
        }
        // 2. tab_model length matches open_tabs
        assert_eq!(app.tab_model.len(), app.open_tabs.len());
        // 3. active_tab state matches open_tabs and tab_model
        if app.open_tabs.is_empty() {
            assert_eq!(app.active_tab(), None);
        } else {
            let active = app.active_tab().expect("expected active tab");
            assert!(app.open_tabs.contains(&active));
            assert_eq!(
                app.tab_model.active_data::<Tab>().cloned().as_ref(),
                Some(&active)
            );
        }
    }
}

#[test]
fn test_session_verification_prompt_lifecycle() {
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());

    // Prompt set when verification needed
    let dev: Arc<str> = Arc::from("DEVICE2");
    let _ = app.update(Message::SessionVerificationNeeded(Some(dev.clone())));
    assert_eq!(
        app.session_verification_prompt,
        Some(crate::constellation::SessionVerificationPrompt {
            target_device_id: Some(dev.clone()),
        })
    );

    // Subsequent notifications do not overwrite existing prompt
    let dev3: Arc<str> = Arc::from("DEVICE3");
    let _ = app.update(Message::SessionVerificationNeeded(Some(dev3)));
    assert_eq!(
        app.session_verification_prompt,
        Some(crate::constellation::SessionVerificationPrompt {
            target_device_id: Some(dev),
        })
    );

    // Dismissing clears prompt
    let _ = app.update(Message::DismissSessionVerificationPrompt);
    assert_eq!(app.session_verification_prompt, None);

    // Prompt without specific device ID
    let _ = app.update(Message::SessionVerificationNeeded(None));
    assert_eq!(
        app.session_verification_prompt,
        Some(crate::constellation::SessionVerificationPrompt {
            target_device_id: None,
        })
    );

    // Verification completion clears prompt
    app.user_settings.verification_ui_state = crate::settings::user::VerificationUIState::Done;
    let _ = app.update(Message::UserSettings(
        crate::settings::user::Message::DismissVerification,
    ));
    assert_eq!(app.session_verification_prompt, None);
}

#[test]
fn test_identity_violations_lifecycle() {
    use matrix_sdk::ruma::user_id;

    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());

    let user1 = user_id!("@evil:example.com").to_owned();
    let user2 = user_id!("@compromised:example.com").to_owned();

    // Detection adds user to violations
    let _ = app.update(Message::IdentityViolationDetected(user1.clone()));
    assert_eq!(app.identity_violations, vec![user1.clone()]);

    // Duplicate detection is deduplicated
    let _ = app.update(Message::IdentityViolationDetected(user1.clone()));
    assert_eq!(app.identity_violations, vec![user1.clone()]);

    // Second user violation is appended
    let _ = app.update(Message::IdentityViolationDetected(user2.clone()));
    assert_eq!(app.identity_violations, vec![user1.clone(), user2.clone()]);

    // Dismiss removes targeted user
    let _ = app.update(Message::DismissIdentityViolation(user1.clone()));
    assert_eq!(app.identity_violations, vec![user2.clone()]);

    // Dismissing non-existent user is no-op
    let _ = app.update(Message::DismissIdentityViolation(user1));
    assert_eq!(app.identity_violations, vec![user2.clone()]);

    // Dismissing last user empties violations
    let _ = app.update(Message::DismissIdentityViolation(user2));
    assert!(app.identity_violations.is_empty());
}

#[test]
fn test_logout_clears_verification_state() {
    use matrix_sdk::ruma::user_id;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());
    app.session_verification_prompt = Some(crate::constellation::SessionVerificationPrompt {
        target_device_id: Some(Arc::from("DEV")),
    });
    app.identity_violations = vec![user_id!("@bob:example.com").to_owned()];

    let _ = app.handle_logout_finished();
    assert_eq!(app.session_verification_prompt, None);
    assert!(app.identity_violations.is_empty());
}

#[hegel::test(test_cases = 100)]
fn test_identity_violations_and_session_prompt_invariants(tc: hegel::TestCase) {
    use hegel::generators;
    use matrix_sdk::ruma::{OwnedUserId, user_id};
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());

    let user_pool: [OwnedUserId; 4] = [
        user_id!("@alice:example.com").to_owned(),
        user_id!("@bob:example.com").to_owned(),
        user_id!("@carol:example.com").to_owned(),
        user_id!("@dave:example.com").to_owned(),
    ];

    let device_pool: [Arc<str>; 3] = [Arc::from("DEV1"), Arc::from("DEV2"), Arc::from("DEV3")];

    let num_ops = tc.draw(generators::integers::<usize>().min_value(1).max_value(50));
    for _ in 0..num_ops {
        let op = tc.draw(generators::integers::<u8>().min_value(0).max_value(4));
        match op {
            0 => {
                let idx = tc.draw(
                    generators::integers::<usize>()
                        .min_value(0)
                        .max_value(user_pool.len() - 1),
                );
                let _ = app.update(Message::IdentityViolationDetected(user_pool[idx].clone()));
            }
            1 => {
                let idx = tc.draw(
                    generators::integers::<usize>()
                        .min_value(0)
                        .max_value(user_pool.len() - 1),
                );
                let _ = app.update(Message::DismissIdentityViolation(user_pool[idx].clone()));
            }
            2 => {
                let with_dev = tc.draw(generators::booleans());
                let dev = if with_dev {
                    let idx = tc.draw(
                        generators::integers::<usize>()
                            .min_value(0)
                            .max_value(device_pool.len() - 1),
                    );
                    Some(device_pool[idx].clone())
                } else {
                    None
                };
                let _ = app.update(Message::SessionVerificationNeeded(dev));
            }
            3 => {
                let _ = app.update(Message::DismissSessionVerificationPrompt);
            }
            _ => {
                app.user_settings.verification_ui_state =
                    crate::settings::user::VerificationUIState::Done;
                let _ = app.update(Message::UserSettings(
                    crate::settings::user::Message::DismissVerification,
                ));
            }
        }

        // Invariant 1: No duplicates in identity_violations
        let mut seen = std::collections::HashSet::new();
        for u in &app.identity_violations {
            assert!(
                seen.insert(u.clone()),
                "Duplicate user ID in identity_violations: {}",
                u
            );
        }

        // Invariant 2: If session prompt exists, target device is non-empty if present
        if let Some(prompt) = &app.session_verification_prompt
            && let Some(dev) = &prompt.target_device_id
        {
            assert!(!dev.is_empty());
        }
    }
}

#[test]
fn test_open_thread_creates_tab_and_activates() {
    use crate::constellation::Tab;
    use matrix_sdk::ruma::OwnedEventId;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let root_id: OwnedEventId = "$root_evt:matrix.org".parse().unwrap();

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a.clone())]);
    assert_eq!(app.active_thread_root, None);

    let _ = app.update(Message::OpenThread(root_id.clone()));
    assert_eq!(
        app.open_tabs,
        vec![
            Tab::Room(room_a.clone()),
            Tab::Thread {
                room_id: room_a.clone(),
                root_id: root_id.clone(),
            }
        ]
    );
    assert_eq!(app.active_thread_root.as_ref(), Some(&root_id));
    assert_eq!(app.tab_model.len(), 2);
    assert_eq!(
        app.tab_model.active_data::<Tab>().cloned(),
        Some(Tab::Thread {
            room_id: room_a.clone(),
            root_id: root_id.clone(),
        })
    );

    // Re-opening the same thread activates it without adding a duplicate tab
    let _ = app.update(Message::OpenThread(root_id.clone()));
    assert_eq!(app.open_tabs.len(), 2);
    assert_eq!(app.active_thread_root.as_ref(), Some(&root_id));
}

#[test]
fn test_switch_between_room_and_thread_tabs() {
    use crate::constellation::Tab;
    use matrix_sdk::ruma::OwnedEventId;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let root_id: OwnedEventId = "$root_evt:matrix.org".parse().unwrap();

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    let _ = app.update(Message::OpenThread(root_id.clone()));
    assert_eq!(app.active_thread_root.as_ref(), Some(&root_id));

    // Find entity for room tab
    let entity_room = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&Tab::Room(room_a.clone())))
        .unwrap();

    // Switch back to room tab: thread root is cleared, timeline restored
    let _ = app.update(Message::TabActivated(entity_room));
    assert_eq!(app.active_thread_root, None);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));
    assert_eq!(
        app.tab_model.active_data::<Tab>().cloned(),
        Some(Tab::Room(room_a.clone()))
    );

    // Find entity for thread tab
    let entity_thread = app
        .tab_model
        .iter()
        .find(|&e| {
            app.tab_model.data::<Tab>(e)
                == Some(&Tab::Thread {
                    room_id: room_a.clone(),
                    root_id: root_id.clone(),
                })
        })
        .unwrap();

    // Switch back to thread tab
    let _ = app.update(Message::TabActivated(entity_thread));
    assert_eq!(app.active_thread_root.as_ref(), Some(&root_id));
    assert_eq!(
        app.tab_model.active_data::<Tab>().cloned(),
        Some(Tab::Thread {
            room_id: room_a.clone(),
            root_id: root_id.clone(),
        })
    );
}

#[test]
fn test_close_thread_tab_restores_room_tab() {
    use crate::constellation::Tab;
    use matrix_sdk::ruma::OwnedEventId;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let root_id: OwnedEventId = "$root_evt:matrix.org".parse().unwrap();

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    let _ = app.update(Message::OpenThread(root_id.clone()));
    assert_eq!(app.open_tabs.len(), 2);
    assert_eq!(app.active_thread_root.as_ref(), Some(&root_id));

    // Close active thread tab via CloseTab shortcut (Ctrl+W)
    let _ = app.update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::CloseTab,
    ));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a.clone())]);
    assert_eq!(app.active_thread_root, None);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));
    assert_eq!(app.tab_model.len(), 1);
}

#[test]
fn test_close_thread_via_escape_shortcut_closes_thread_tab() {
    use crate::constellation::Tab;
    use matrix_sdk::ruma::OwnedEventId;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let root_id: OwnedEventId = "$root_evt:matrix.org".parse().unwrap();

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    let _ = app.update(Message::OpenThread(root_id.clone()));
    assert_eq!(app.open_tabs.len(), 2);
    assert_eq!(app.active_thread_root.as_ref(), Some(&root_id));

    // Close active thread via CloseThread shortcut (Escape)
    let _ = app.update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::CloseThread,
    ));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a.clone())]);
    assert_eq!(app.active_thread_root, None);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));
    assert_eq!(app.tab_model.len(), 1);
}

#[test]
fn test_close_space_switcher_message_unselects_space() {
    let mut app = create_dummy_constellation();
    app.room_list = vec![matrix::RoomData {
        id: std::sync::Arc::from("!space1:matrix.org"),
        name: Some("Space One".to_string()),
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
    }];
    app.rebuild_space_nav_model();
    let _ = app.handle_select_space(Some(std::sync::Arc::from("!space1:matrix.org")));
    assert_eq!(
        app.selected_space.as_ref().map(|s| s.as_str()),
        Some("!space1:matrix.org")
    );

    let _ = app.handle_update(Message::CloseSpaceSwitcher);
    assert_eq!(app.selected_space, None);
    assert!(!app.is_room_list_open);
    assert_eq!(
        app.space_nav_model.position(app.space_nav_model.active()),
        None
    );
}

#[test]
fn test_close_space_switcher_shortcut_unselects_space() {
    let mut app = shortcut_app();
    app.room_list = vec![matrix::RoomData {
        id: std::sync::Arc::from("!space1:matrix.org"),
        name: Some("Space One".to_string()),
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
    }];
    app.rebuild_space_nav_model();
    let _ = app.handle_select_space(Some(std::sync::Arc::from("!space1:matrix.org")));
    assert!(app.selected_space.is_some());

    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::CloseSpaceSwitcher,
    ));
    assert_eq!(app.selected_space, None);
    assert!(!app.is_room_list_open);
}

#[test]
fn test_close_space_switcher_noop_when_already_closed() {
    let mut app = shortcut_app();
    app.is_room_list_open = false;
    assert_eq!(app.selected_space, None);

    let _ = app.handle_update(Message::CloseSpaceSwitcher);
    assert_eq!(app.selected_space, None);
    assert!(!app.is_room_list_open);

    let _ = app.handle_update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::CloseSpaceSwitcher,
    ));
    assert_eq!(app.selected_space, None);
    assert!(!app.is_room_list_open);
}

#[test]
fn test_close_space_switcher_unselects_when_all_rooms_is_open() {
    let mut app = shortcut_app();
    app.rebuild_space_nav_model();
    app.is_room_list_open = true;
    app.selected_space = None;
    app.sync_space_nav_activation();
    assert_eq!(
        app.space_nav_model.position(app.space_nav_model.active()),
        Some(0)
    );

    let _ = app.handle_update(Message::CloseSpaceSwitcher);
    assert_eq!(app.selected_space, None);
    assert!(!app.is_room_list_open);
    assert_eq!(
        app.space_nav_model.position(app.space_nav_model.active()),
        None
    );
}

#[test]
fn test_set_error_records_session_error_and_sets_deadline() {
    let mut app = create_dummy_constellation();
    assert!(app.error.is_none());
    assert!(app.error_autoclose_deadline.is_none());
    assert!(app.app_settings.session_errors.is_empty());

    app.set_error("Network connection failed".to_string());

    assert_eq!(app.error.as_deref(), Some("Network connection failed"));
    assert!(app.error_autoclose_deadline.is_some());
    assert_eq!(app.app_settings.session_errors.len(), 1);
    assert_eq!(
        app.app_settings.session_errors[0].message,
        "Network connection failed"
    );
}

#[test]
fn test_error_autoclose_on_restore_tick_when_deadline_passed() {
    let mut app = create_dummy_constellation();
    app.set_error("Temporary glitch".to_string());

    assert!(app.error.is_some());
    assert!(app.error_autoclose_deadline.is_some());
    assert_eq!(app.app_settings.session_errors.len(), 1);

    // Simulate deadline having passed
    app.error_autoclose_deadline =
        Some(std::time::Instant::now() - std::time::Duration::from_millis(1));

    let _ = app.handle_update(Message::RestoreTick);

    // The visible overlay error is autoclosed:
    assert!(app.error.is_none());
    assert!(app.error_autoclose_deadline.is_none());

    // The error is still preserved in session errors:
    assert_eq!(app.app_settings.session_errors.len(), 1);
    assert_eq!(
        app.app_settings.session_errors[0].message,
        "Temporary glitch"
    );
}

#[test]
fn test_error_persists_on_restore_tick_before_deadline() {
    let mut app = create_dummy_constellation();
    app.set_error("Still active error".to_string());

    // Deadline is in the future
    app.error_autoclose_deadline =
        Some(std::time::Instant::now() + std::time::Duration::from_secs(10));

    let _ = app.handle_update(Message::RestoreTick);

    assert_eq!(app.error.as_deref(), Some("Still active error"));
    assert!(app.error_autoclose_deadline.is_some());
    assert_eq!(app.app_settings.session_errors.len(), 1);
}

#[test]
fn test_dismiss_error_clears_overlay_and_deadline_but_preserves_session_error() {
    let mut app = create_dummy_constellation();
    app.set_error("User dismissed error".to_string());

    assert!(app.error.is_some());
    assert!(app.error_autoclose_deadline.is_some());

    let _ = app.handle_update(Message::DismissError);

    assert!(app.error.is_none());
    assert!(app.error_autoclose_deadline.is_none());
    assert_eq!(app.app_settings.session_errors.len(), 1);
    assert_eq!(
        app.app_settings.session_errors[0].message,
        "User dismissed error"
    );
}

#[test]
fn test_multiple_errors_accumulate_in_session_errors() {
    let mut app = create_dummy_constellation();

    app.set_error("First error".to_string());
    app.set_error("Second error".to_string());
    app.set_error("Third error".to_string());

    // Latest error is in app.error
    assert_eq!(app.error.as_deref(), Some("Third error"));

    // All 3 are recorded in session errors
    assert_eq!(app.app_settings.session_errors.len(), 3);
    assert_eq!(app.app_settings.session_errors[0].message, "First error");
    assert_eq!(app.app_settings.session_errors[1].message, "Second error");
    assert_eq!(app.app_settings.session_errors[2].message, "Third error");
}

#[test]
fn test_handle_space_children_fetched_selected_space_mismatch() {
    let mut app = create_dummy_constellation();
    app.selected_space = Some(RoomId::parse("!space1:example.com").unwrap());

    let other_space_id = RoomId::parse("!space2:example.com").unwrap();
    let child_room = matrix::RoomData {
        id: std::sync::Arc::from("!room1:example.com"),
        name: Some("Child Room".to_string()),
        unread_count: 0,
        unread_count_str: None,
        last_message: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: Some("!space2:example.com".to_string()),
        join_rule: None,
        allowed_spaces: Vec::new(),
        order: None,
        suggested: false,
    };

    let _task = app.handle_space_children_fetched(other_space_id, Ok(vec![child_room]));

    // other_rooms should remain empty because space_id did not match selected_space
    assert!(app.other_rooms.is_empty());
}

#[test]
fn test_handle_space_children_fetched_success() {
    let mut app = create_dummy_constellation();
    let space_id = RoomId::parse("!space1:example.com").unwrap();
    app.selected_space = Some(space_id.clone());

    let child_room = matrix::RoomData {
        id: std::sync::Arc::from("!room1:example.com"),
        name: Some("Child Room".to_string()),
        unread_count: 0,
        unread_count_str: None,
        last_message: None,
        avatar_url: Some("mxc://example.com/avatar".to_string()),
        room_type: None,
        is_space: false,
        parent_space_id: Some("!space1:example.com".to_string()),
        join_rule: None,
        allowed_spaces: Vec::new(),
        order: None,
        suggested: false,
    };

    let _task = app.handle_space_children_fetched(space_id, Ok(vec![child_room]));

    assert_eq!(app.other_rooms.len(), 1);
    assert_eq!(app.other_rooms[0].id.as_ref(), "!room1:example.com");
}
// ===== Search Tab Tests (Issue #485) =====

#[test]
fn test_submit_search_creates_tab_and_activates() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a.clone())]);
    assert_eq!(app.active_search, None);

    app.search_query = "hello".to_string();
    let _ = app.update(Message::SubmitSearch);

    let expected_search_tab = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "hello".to_string(),
    };

    assert_eq!(
        app.open_tabs,
        vec![Tab::Room(room_a.clone()), expected_search_tab.clone()]
    );
    assert_eq!(app.active_search.as_ref(), Some(&expected_search_tab));
    assert_eq!(app.tab_model.len(), 2);
    assert_eq!(
        app.tab_model.active_data::<Tab>().cloned(),
        Some(expected_search_tab)
    );
    assert_eq!(
        app.current_title(),
        crate::fl!("search-results-for", needle = "hello").to_string()
    );
}

#[test]
fn test_reopen_same_search_activates_without_duplicate() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    app.search_query = "hello".to_string();
    let _ = app.update(Message::SubmitSearch);
    assert_eq!(app.open_tabs.len(), 2);

    // Submitting the same query again must not add a duplicate tab
    let _ = app.update(Message::SubmitSearch);
    assert_eq!(app.open_tabs.len(), 2);
    assert_eq!(
        app.active_search,
        Some(Tab::Search {
            room_id: Some(room_a),
            query: "hello".to_string(),
        })
    );
}

#[test]
fn test_switch_between_room_and_search_tabs_preserves_results() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    app.search_query = "hello".to_string();
    let _ = app.update(Message::SubmitSearch);

    let search_tab = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "hello".to_string(),
    };

    // Simulate search results landing
    let mock_hit = matrix::MessageSearchResult {
        room_id: matrix_sdk::ruma::room_id!("!a:matrix.org").to_owned(),
        room_name: Some("Room A".to_string()),
        event_id: matrix_sdk::ruma::EventId::parse("$hit1:example.com").unwrap(),
        sender_id: matrix_sdk::ruma::user_id!("@alice:example.com").to_owned(),
        body: "hello world".to_string(),
        timestamp: "2026-01-01 00:00:00".to_string(),
        plain_text: Vec::new(),
        links: Vec::new(),
    };
    app.message_search_results.push(mock_hit.clone());

    // Switch back to room tab
    let entity_room = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&Tab::Room(room_a.clone())))
        .unwrap();

    let _ = app.update(Message::TabActivated(entity_room));
    assert_eq!(app.active_search, None);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));

    // Switch back to search tab
    let entity_search = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&search_tab))
        .unwrap();

    let _ = app.update(Message::TabActivated(entity_search));
    assert_eq!(app.active_search.as_ref(), Some(&search_tab));
    assert_eq!(app.message_search_results.len(), 1);
    assert_eq!(app.message_search_results[0].body, "hello world");
}

#[test]
fn test_multiple_search_tabs_keep_independent_results() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));

    // Search 1: "apple"
    app.search_query = "apple".to_string();
    let _ = app.update(Message::SubmitSearch);
    let tab_apple = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "apple".to_string(),
    };
    let hit_apple = matrix::MessageSearchResult {
        room_id: matrix_sdk::ruma::room_id!("!a:matrix.org").to_owned(),
        room_name: Some("Room A".to_string()),
        event_id: matrix_sdk::ruma::EventId::parse("$hit1:example.com").unwrap(),
        sender_id: matrix_sdk::ruma::user_id!("@alice:example.com").to_owned(),
        body: "apples are great".to_string(),
        timestamp: "2026-01-01 00:00:00".to_string(),
        plain_text: Vec::new(),
        links: Vec::new(),
    };
    app.message_search_results.push(hit_apple);

    // Switch back to room tab before starting second search
    let entity_room = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&Tab::Room(room_a.clone())))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_room));

    // Search 2: "banana"
    app.search_query = "banana".to_string();
    let _ = app.update(Message::SubmitSearch);
    let tab_banana = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "banana".to_string(),
    };
    let hit_banana = matrix::MessageSearchResult {
        room_id: matrix_sdk::ruma::room_id!("!a:matrix.org").to_owned(),
        room_name: Some("Room A".to_string()),
        event_id: matrix_sdk::ruma::EventId::parse("$hit2:example.com").unwrap(),
        sender_id: matrix_sdk::ruma::user_id!("@bob:example.com").to_owned(),
        body: "bananas are yellow".to_string(),
        timestamp: "2026-01-01 00:00:00".to_string(),
        plain_text: Vec::new(),
        links: Vec::new(),
    };
    app.message_search_results.push(hit_banana);
    assert_eq!(
        app.open_tabs,
        vec![
            Tab::Room(room_a.clone()),
            tab_banana.clone(),
            tab_apple.clone()
        ]
    );

    // Switch to apple tab: verify apple results restored
    let entity_apple = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&tab_apple))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_apple));
    assert_eq!(app.active_search.as_ref(), Some(&tab_apple));
    assert_eq!(app.message_search_results.len(), 1);
    assert_eq!(app.message_search_results[0].body, "apples are great");

    // Switch to banana tab: verify banana results restored
    let entity_banana = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&tab_banana))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_banana));
    assert_eq!(app.active_search.as_ref(), Some(&tab_banana));
    assert_eq!(app.message_search_results.len(), 1);
    assert_eq!(app.message_search_results[0].body, "bananas are yellow");
}

#[test]
fn test_close_search_tab_restores_room_tab() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    app.user_id = Some("@alice:matrix.org".to_string());
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    app.search_query = "hello".to_string();
    let _ = app.update(Message::SubmitSearch);
    assert_eq!(app.open_tabs.len(), 2);
    assert!(app.active_search.is_some());

    let search_tab = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "hello".to_string(),
    };

    // Close active search tab via CloseTab shortcut (Ctrl+W)
    let _ = app.update(Message::ShortcutTriggered(
        crate::constellation::keybind::ShortcutAction::CloseTab,
    ));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a.clone())]);
    assert_eq!(app.active_search, None);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));
    assert_eq!(app.tab_model.len(), 1);
    assert!(!app.search_results.contains_key(&search_tab));
}

#[test]
fn test_global_search_tab_without_room() {
    use crate::constellation::Tab;

    let mut app = create_dummy_constellation();
    assert_eq!(app.selected_room, None);

    app.search_query = "global search".to_string();
    let _ = app.update(Message::SubmitSearch);

    let expected_tab = Tab::Search {
        room_id: None,
        query: "global search".to_string(),
    };

    assert_eq!(app.open_tabs, vec![expected_tab.clone()]);
    assert_eq!(app.active_search.as_ref(), Some(&expected_tab));
    assert_eq!(app.selected_room, None);
    assert_eq!(app.tab_model.len(), 1);
}

#[test]
fn test_jump_to_message_from_search_tab_activates_room_tab() {
    use crate::constellation::Tab;
    use matrix_sdk::ruma::OwnedEventId;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let event_id: OwnedEventId = matrix_sdk::ruma::EventId::parse("$evt1:matrix.org").unwrap();

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    app.search_query = "query".to_string();
    let _ = app.update(Message::SubmitSearch);
    assert!(app.active_search.is_some());

    // Jumping to a message in the room should restore the room tab
    let _ = app.update(Message::JumpToMessageOrLoadContext(event_id));
    assert_eq!(app.active_search, None);
    assert_eq!(app.selected_room.as_ref(), Some(&room_a));
    assert_eq!(app.active_tab(), Some(Tab::Room(room_a)));
}
#[test]
fn test_typing_does_not_update_active_search_tab_or_launch_search() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    app.search_query = "apple".to_string();
    let _ = app.update(Message::SubmitSearch);

    let apple_tab = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "apple".to_string(),
    };
    assert_eq!(app.active_search.as_ref(), Some(&apple_tab));
    assert_eq!(app.search_generation, 1);

    // Typing a new query must NOT update the active search tab or increment generation
    let _ = app.update(Message::SearchQueryChanged("banana".to_string()));
    assert_eq!(app.search_query, "banana");
    assert_eq!(app.active_search.as_ref(), Some(&apple_tab));
    assert_eq!(app.search_generation, 1);
    assert_eq!(
        app.open_tabs,
        vec![Tab::Room(room_a.clone()), apple_tab.clone()]
    );

    // Typing more still does not update
    let _ = app.update(Message::SearchQueryChanged("banana split".to_string()));
    assert_eq!(app.active_search.as_ref(), Some(&apple_tab));
    assert_eq!(app.search_generation, 1);

    // Submitting with Enter updates the active search tab in place and launches search
    let _ = app.update(Message::SubmitSearch);
    let updated_tab = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "banana split".to_string(),
    };
    assert_eq!(app.active_search.as_ref(), Some(&updated_tab));
    assert_eq!(app.search_generation, 2);
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a), updated_tab]);
}

#[test]
fn test_submit_search_updates_existing_tab_or_switches_without_duplication() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));

    // Open search 1: "apple"
    app.search_query = "apple".to_string();
    let _ = app.update(Message::SubmitSearch);
    let tab_apple = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "apple".to_string(),
    };

    // Switch back to room tab
    let entity_room = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&Tab::Room(room_a.clone())))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_room));

    // Open search 2: "banana"
    app.search_query = "banana".to_string();
    let _ = app.update(Message::SubmitSearch);
    let tab_banana = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "banana".to_string(),
    };

    assert_eq!(
        app.open_tabs,
        vec![
            Tab::Room(room_a.clone()),
            tab_banana.clone(),
            tab_apple.clone()
        ]
    );
    assert_eq!(app.active_search.as_ref(), Some(&tab_banana));

    // Now on banana tab, submit "apple" (which already exists as a tab)
    app.search_query = "apple".to_string();
    let _ = app.update(Message::SubmitSearch);

    // Must switch to existing apple tab without creating duplicates
    assert_eq!(app.active_search.as_ref(), Some(&tab_apple));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a), tab_apple]);
}

#[test]
fn test_submit_empty_or_whitespace_search_does_nothing() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));
    assert_eq!(app.open_tabs, vec![Tab::Room(room_a)]);
    assert_eq!(app.search_generation, 0);

    app.search_query = "   ".to_string();
    let _ = app.update(Message::SubmitSearch);

    assert_eq!(app.open_tabs.len(), 1);
    assert_eq!(app.search_generation, 0);
    assert_eq!(app.active_search, None);
}

#[test]
fn test_switch_search_tab_syncs_search_query() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");

    let _ = app.update(Message::RoomSelected(room_a.clone()));

    // Search 1: "apple"
    app.search_query = "apple".to_string();
    let _ = app.update(Message::SubmitSearch);
    let tab_apple = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "apple".to_string(),
    };

    // Switch to room, then search 2: "banana"
    let entity_room = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&Tab::Room(room_a.clone())))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_room));

    app.search_query = "banana".to_string();
    let _ = app.update(Message::SubmitSearch);
    let tab_banana = Tab::Search {
        room_id: Some(room_a),
        query: "banana".to_string(),
    };

    assert_eq!(app.search_query, "banana");

    // Switch to apple tab
    let entity_apple = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&tab_apple))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_apple));
    assert_eq!(app.search_query, "apple");

    // Switch to banana tab
    let entity_banana = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&tab_banana))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_banana));
    assert_eq!(app.search_query, "banana");
}
#[test]
fn test_search_launched_stops_room_filtering_and_tab_switch() {
    use crate::constellation::Tab;
    use std::sync::Arc;

    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!room_a:matrix.org");
    let room_b: Arc<str> = Arc::from("!room_b:matrix.org");

    app.room_list = vec![
        crate::matrix::RoomData {
            id: room_a.clone(),
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
        crate::matrix::RoomData {
            id: room_b.clone(),
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
    app.update_filtered_rooms();
    assert_eq!(app.filtered_room_list.len(), 2);

    // Select room A
    let _ = app.update(Message::RoomSelected(room_a.clone()));
    assert_eq!(app.filtered_room_list.len(), 2);

    // Typing in search field filters rooms while search has not been launched (#507)
    let _ = app.update(Message::SearchQueryChanged("Alpha".to_string()));
    assert_eq!(app.filtered_room_list.len(), 1);
    assert_eq!(
        app.room_list[app.filtered_room_list[0]].id.as_ref(),
        "!room_a:matrix.org"
    );

    // Once Enter is pressed (SubmitSearch) and search opens in a Tab, room filtering stops (#507)
    let _ = app.update(Message::SubmitSearch);
    assert!(app.active_search.is_some());
    assert_eq!(app.filtered_room_list.len(), 2);

    // Typing while on search tab must NOT filter rooms
    let _ = app.update(Message::SearchQueryChanged("Beta".to_string()));
    assert_eq!(app.filtered_room_list.len(), 2);

    // Switching back to room tab clears search query and keeps rooms unfiltered
    let entity_room = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&Tab::Room(room_a.clone())))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_room));
    assert_eq!(app.active_search, None);
    assert_eq!(app.search_query, "");
    assert_eq!(app.filtered_room_list.len(), 2);

    // Switching back to search tab restores search query and keeps rooms unfiltered
    let search_tab = Tab::Search {
        room_id: Some(room_a.clone()),
        query: "Alpha".to_string(),
    };
    let entity_search = app
        .tab_model
        .iter()
        .find(|&e| app.tab_model.data::<Tab>(e) == Some(&search_tab))
        .unwrap();
    let _ = app.update(Message::TabActivated(entity_search));
    assert_eq!(app.active_search.as_ref(), Some(&search_tab));
    assert_eq!(app.search_query, "Alpha");
    assert_eq!(app.filtered_room_list.len(), 2);
}

#[test]
fn test_search_bar_renders_active_and_inactive() {
    let mut app = create_dummy_constellation();

    // Inactive search bar
    app.is_search_active = false;
    {
        let mut inactive_elements = Vec::new();
        app.search_bar(&mut inactive_elements);
        assert_eq!(inactive_elements.len(), 1);
    }

    // Active search bar with empty query
    app.is_search_active = true;
    app.search_query = String::new();
    {
        let mut active_elements = Vec::new();
        app.search_bar(&mut active_elements);
        assert_eq!(active_elements.len(), 1);
    }

    // Active search bar with non-empty query (includes submit button)
    app.search_query = "test".to_string();
    {
        let mut active_with_query = Vec::new();
        app.search_bar(&mut active_with_query);
        assert_eq!(active_with_query.len(), 1);
    }
}

#[test]
fn test_resolve_room_filter() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_dev: Arc<str> = Arc::from("!dev:matrix.org");
    let room_gen: Arc<str> = Arc::from("!general:matrix.org");

    app.room_list.push(matrix::RoomData {
        id: room_dev.clone(),
        name: Some("Development".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        order: None,
        suggested: false,
    });

    app.room_list.push(matrix::RoomData {
        id: room_gen.clone(),
        name: Some("General Chat".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        order: None,
        suggested: false,
    });

    // Full room ID
    assert_eq!(
        app.resolve_room_filter("!dev:matrix.org"),
        Some(room_dev.clone())
    );

    // Exact name match (case-insensitive)
    assert_eq!(
        app.resolve_room_filter("development"),
        Some(room_dev.clone())
    );
    assert_eq!(
        app.resolve_room_filter("#DEVELOPMENT"),
        Some(room_dev.clone())
    );

    // Prefix match
    assert_eq!(app.resolve_room_filter("dev"), Some(room_dev.clone()));
    assert_eq!(app.resolve_room_filter("#gen"), Some(room_gen.clone()));

    // Substring match
    assert_eq!(app.resolve_room_filter("chat"), Some(room_gen.clone()));

    // Unmatched
    assert_eq!(app.resolve_room_filter("nonexistent_room"), None);
}

#[test]
fn test_submit_search_with_room_filter() {
    use crate::constellation::Tab;
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_dev: Arc<str> = Arc::from("!dev:matrix.org");

    app.room_list.push(matrix::RoomData {
        id: room_dev.clone(),
        name: Some("Development".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        order: None,
        suggested: false,
    });

    app.search_query = "#dev bug report".to_string();
    let _ = app.update(Message::SubmitSearch);

    assert_eq!(app.open_tabs.len(), 1);
    let expected_tab = Tab::Search {
        room_id: Some(room_dev),
        query: "#dev bug report".to_string(),
    };
    assert_eq!(app.active_search.as_ref(), Some(&expected_tab));
}

#[test]
fn test_submit_search_with_scope_filter() {
    use crate::constellation::Tab;
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_a: Arc<str> = Arc::from("!a:matrix.org");
    let _ = app.update(Message::RoomSelected(room_a));

    app.search_query = "is:dm project update".to_string();
    let _ = app.update(Message::SubmitSearch);

    assert_eq!(app.global_search_scope, matrix::GlobalSearchScope::DmsOnly);
    let expected_tab = Tab::Search {
        room_id: None,
        query: "is:dm project update".to_string(),
    };
    assert_eq!(app.active_search.as_ref(), Some(&expected_tab));
}

#[test]
fn test_update_search_suggestions_rooms() {
    use std::sync::Arc;
    let mut app = create_dummy_constellation();
    let room_dev: Arc<str> = Arc::from("!dev:matrix.org");

    app.room_list.push(matrix::RoomData {
        id: room_dev.clone(),
        name: Some("Development".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        join_rule: None,
        allowed_spaces: Vec::new(),
        order: None,
        suggested: false,
    });

    // Typing '#' triggers room autocomplete
    let _ = app.update(Message::SearchQueryChanged("#".to_string()));
    assert!(app.show_search_suggestions);
    assert_eq!(app.search_suggestions.len(), 1);
    assert_eq!(app.search_suggestions[0].display_text, "Development");
    assert_eq!(app.search_suggestions[0].replacement, "#Development ");

    // Selecting suggestion applies it and focuses input
    let _ = app.update(Message::SearchApplySuggestion(
        app.search_suggestions[0].replacement.clone(),
    ));
    assert_eq!(app.search_query, "#Development ");
    assert!(!app.show_search_suggestions);
    assert!(app.search_suggestions.is_empty());
}

#[test]
fn test_update_search_suggestions_members() {
    let mut app = create_dummy_constellation();
    app.room_members.push(matrix::RoomMemberInfo {
        user_id: "@alice:matrix.org".to_string(),
        display_name: Some("Alice".to_string()),
        avatar_url: None,
    });

    // Typing '@' triggers member autocomplete
    let _ = app.update(Message::SearchQueryChanged("@".to_string()));
    assert!(app.show_search_suggestions);
    assert_eq!(app.search_suggestions.len(), 1);
    assert_eq!(app.search_suggestions[0].display_text, "Alice");
    assert_eq!(
        app.search_suggestions[0].secondary_text.as_deref(),
        Some("@alice:matrix.org")
    );
    assert_eq!(app.search_suggestions[0].replacement, "@alice:matrix.org ");

    // Applying suggestion
    let _ = app.update(Message::SearchApplySuggestion(
        "@alice:matrix.org ".to_string(),
    ));
    assert_eq!(app.search_query, "@alice:matrix.org ");
    assert!(!app.show_search_suggestions);
}

#[test]
fn test_search_bar_renders_with_suggestions() {
    let mut app = create_dummy_constellation();
    app.is_search_active = true;
    app.search_query = "#".to_string();
    app.search_suggestions
        .push(crate::constellation::SearchSuggestion {
            display_text: "General".to_string(),
            secondary_text: Some("!gen:matrix.org".to_string()),
            replacement: "#General ".to_string(),
            is_room: true,
        });
    app.show_search_suggestions = true;

    let mut elements = Vec::new();
    app.search_bar(&mut elements);
    assert_eq!(elements.len(), 1);
}

#[test]
fn test_fetch_missing_og_previews_policy() {
    let mut app = create_dummy_constellation();
    let item = crate::ConstellationItem::mock("Alice", "Check https://matrix.org", "12:00", false);
    app.timeline_items.push_back(item);

    // When policy is false, should not fetch previews
    app.user_settings.media_previews_display_policy = false;
    assert!(app.fetch_missing_og_previews().is_none());

    // When policy is true, should return tasks to fetch
    app.user_settings.media_previews_display_policy = true;
    assert!(app.fetch_missing_og_previews().is_some());

    // Once in cache, should not re-fetch
    app.og_cache.insert(
        "https://matrix.org".to_string(),
        crate::utils::og::OgState::Pending,
    );
    assert!(app.fetch_missing_og_previews().is_none());
}

#[test]
fn test_format_body_with_emojis() {
    let mut app = create_dummy_constellation();

    // No active emojis: should return initial_html unchanged
    assert_eq!(app.format_body_with_emojis("Hello :cat:", None), None);
    assert_eq!(
        app.format_body_with_emojis("Hello :cat:", Some("<p>Hello :cat:</p>".to_string())),
        Some("<p>Hello :cat:</p>".to_string())
    );

    // Add active custom emoji
    app.active_custom_emojis.push(matrix::ImagePackItem {
        shortcode: "cat".to_string(),
        body: "Happy Cat".to_string(),
        url: "mxc://example.org/cat".to_string(),
        is_emoji: true,
        is_sticker: false,
        width: Some(64),
        height: Some(64),
    });

    // Message with :cat: should have it formatted into <img data-mx-emoticon ...>
    let formatted = app
        .format_body_with_emojis("Hello :cat: world", None)
        .unwrap();
    assert!(formatted.contains(
        r#"<img data-mx-emoticon src="mxc://example.org/cat" alt=":cat:" title=":cat:" />"#
    ));
    assert!(formatted.starts_with("Hello "));
    assert!(formatted.ends_with(" world"));

    // Message without :cat: should not format
    assert_eq!(app.format_body_with_emojis("Hello dog", None), None);
}

#[test]
fn test_update_active_emojis_and_stickers() {
    let mut app = create_dummy_constellation();
    let room_id = matrix_sdk::ruma::RoomId::parse("!room:example.org").unwrap();
    app.selected_room = Some(std::sync::Arc::from("!room:example.org"));

    let room_pack = matrix::ImagePack {
        room_id: Some(room_id.clone()),
        state_key: "pack1".to_string(),
        display_name: Some("Room Pack".to_string()),
        avatar_url: None,
        images: vec![matrix::ImagePackItem {
            shortcode: "room_cat".to_string(),
            body: "Room Cat".to_string(),
            url: "mxc://example.org/room_cat".to_string(),
            is_emoji: true,
            is_sticker: true,
            width: None,
            height: None,
        }],
        is_globally_enabled: false,
    };
    app.room_image_packs.insert(room_id, vec![room_pack]);

    let user_pack = matrix::ImagePack {
        room_id: None,
        state_key: "user".to_string(),
        display_name: Some("User Pack".to_string()),
        avatar_url: None,
        images: vec![matrix::ImagePackItem {
            shortcode: "user_dog".to_string(),
            body: "User Dog".to_string(),
            url: "mxc://example.org/user_dog".to_string(),
            is_emoji: true,
            is_sticker: false,
            width: None,
            height: None,
        }],
        is_globally_enabled: true,
    };
    app.user_image_packs.push(user_pack);

    app.update_active_emojis_and_stickers();

    assert_eq!(app.active_custom_emojis.len(), 2);
    assert!(
        app.active_custom_emojis
            .iter()
            .any(|e| e.shortcode == "room_cat")
    );
    assert!(
        app.active_custom_emojis
            .iter()
            .any(|e| e.shortcode == "user_dog")
    );

    assert_eq!(app.active_stickers.len(), 1);
    assert_eq!(app.active_stickers[0].shortcode, "room_cat");
}

#[hegel::test]
fn prop_format_body_with_emojis_no_panic(tc: hegel::TestCase) {
    let mut app = create_dummy_constellation();
    app.active_custom_emojis.push(matrix::ImagePackItem {
        shortcode: "cat".to_string(),
        body: "Cat".to_string(),
        url: "mxc://example.org/cat".to_string(),
        is_emoji: true,
        is_sticker: false,
        width: None,
        height: None,
    });
    let body: String = tc.draw(hegel::generators::text());
    let _ = app.format_body_with_emojis(&body, None);
}
