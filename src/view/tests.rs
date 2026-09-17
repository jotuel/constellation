#[cfg(test)]
use crate::constellation::Constellation;

#[test]
fn test_view_timeline_renders_without_panicking() {
    let constellation = Constellation::mock();
    let _element = constellation.view_timeline();
}

#[test]
fn test_view_threaded_timeline_renders_without_panicking() {
    let constellation = Constellation::mock();
    let _element = constellation.view_threaded_timeline();
}

#[test]
fn test_view_threaded_timeline_renders_multiple_messages() {
    let mut constellation = Constellation::mock();
    let root_id = matrix_sdk::ruma::EventId::parse("$root_event").unwrap();
    constellation.active_thread_root = Some(root_id.clone());

    let mut item1 = crate::ConstellationItem::mock("Alice", "Opening message", "12:00", false);
    item1.item_id = Some(crate::matrix::TimelineEventItemId::EventId(root_id.clone()));

    let reply_id = matrix_sdk::ruma::EventId::parse("$reply_event").unwrap();
    let mut item2 =
        crate::ConstellationItem::mock("Bob", "Reply message in thread", "12:05", false);
    item2.item_id = Some(crate::matrix::TimelineEventItemId::EventId(reply_id));
    item2.thread_root_id = Some(root_id);

    constellation.threaded_timeline_items.push_back(item1);
    constellation.threaded_timeline_items.push_back(item2);

    assert_eq!(constellation.threaded_timeline_items.len(), 2);
    let _element = constellation.view_threaded_timeline();
}

#[test]
fn test_view_timeline_with_url_previews() {
    let mut constellation = Constellation::mock();
    let url = "https://matrix.org";
    let item = crate::ConstellationItem::mock("Alice", "Check https://matrix.org", "12:00", false);
    constellation.timeline_items.push_back(item);

    constellation.og_cache.insert(
        url.to_string(),
        crate::utils::og::OgState::Loaded(std::sync::Arc::new(crate::utils::og::OgPreview {
            url: url.to_string(),
            title: Some("Matrix.org".to_string()),
            description: Some("Open network for secure communication".to_string()),
            site_name: Some("Matrix".to_string()),
            domain: "matrix.org".to_string(),
            image_url: None,
            image: None,
        })),
    );

    // When media_previews_display_policy is true
    constellation.user_settings.media_previews_display_policy = true;
    {
        let _element_enabled = constellation.view_timeline();
    }

    // When media_previews_display_policy is false
    constellation.user_settings.media_previews_display_policy = false;
    {
        let _element_disabled = constellation.view_timeline();
    }
}

#[test]
fn test_view_main_content_renders_without_panicking() {
    let constellation = Constellation::mock();
    let _element = constellation.view_main_content();
}

#[test]
fn test_view_composer_renders_without_panicking() {
    let constellation = Constellation::mock();
    let _element = constellation.view_composer();
}

#[test]
fn test_view_search_results_renders_without_panicking() {
    let constellation = Constellation::mock();
    let _element = constellation.view_search_results();
}

#[test]
fn test_view_main_content_renders_search_tab() {
    let mut constellation = Constellation::mock();
    constellation.selected_room = Some(std::sync::Arc::from("!room:matrix.org"));
    constellation.active_search = Some(crate::constellation::Tab::Search {
        room_id: Some(std::sync::Arc::from("!room:matrix.org")),
        query: "needle".to_string(),
    });
    let _element = constellation.view_main_content();
}

#[test]
fn test_current_title_follows_search_state() {
    let mut constellation = Constellation::mock();
    // No room open: app subtitle.
    assert_eq!(constellation.current_title(), crate::fl!("app-subtitle"));

    // Open room wins over the subtitle (name served from the cache).
    constellation.selected_room = Some(std::sync::Arc::from("!room:matrix.org"));
    constellation
        .room_name_cache
        .insert(std::sync::Arc::from("!room:matrix.org"), "Epaz".to_string());
    assert_eq!(constellation.current_title(), "Epaz");

    // An active search tab owns the title (#427, #485).
    constellation.active_search = Some(crate::constellation::Tab::Search {
        room_id: None,
        query: "the".to_string(),
    });
    assert_eq!(
        constellation.current_title(),
        crate::fl!("search-results-for", needle = "the").to_string()
    );
}

#[test]
fn test_view_members_panel_renders_without_panicking() {
    let constellation = Constellation::mock();
    let _element = constellation.view_members_panel();
}

#[test]
fn test_view_pinned_panel_renders_without_panicking() {
    let constellation = Constellation::mock();
    let _element = constellation.view_pinned_panel();
}

#[test]
fn test_view_active_threads_panel_renders_without_panicking() {
    let constellation = Constellation::mock();
    let _element = constellation.view_active_threads_panel();
}

#[test]
fn test_view_active_threads_panel_renders_loading() {
    let mut constellation = Constellation::mock();
    constellation.is_loading_active_threads = true;
    let _element = constellation.view_active_threads_panel();
}

#[test]
fn test_view_active_threads_panel_renders_with_items() {
    let mut constellation = Constellation::mock();
    constellation
        .active_threads
        .push(crate::matrix::ActiveThreadInfo {
            event_id: "$root1:example.com".to_string(),
            sender_id: "@alice:example.com".to_string(),
            sender_name: "Alice".to_string(),
            avatar_url: None,
            timestamp: "2026-09-15 10:00:00".to_string(),
            body: "First thread starter".to_string(),
            num_replies: 1,
            latest_activity: Some("2026-09-15 10:05:00".to_string()),
            num_unread_messages: 0,
            num_unread_notifications: 0,
            num_unread_mentions: 0,
        });
    constellation
        .active_threads
        .push(crate::matrix::ActiveThreadInfo {
            event_id: "$root2:example.com".to_string(),
            sender_id: "@bob:example.com".to_string(),
            sender_name: "Bob".to_string(),
            avatar_url: None,
            timestamp: "2026-09-15 11:00:00".to_string(),
            body: "Second thread starter".to_string(),
            num_replies: 12,
            latest_activity: None,
            num_unread_messages: 3,
            num_unread_notifications: 1,
            num_unread_mentions: 1,
        });

    let _element = constellation.view_active_threads_panel();
}
#[test]
fn test_view_active_threads_panel_renders_with_unread_badge() {
    let mut constellation = Constellation::mock();
    constellation
        .active_threads
        .push(crate::matrix::ActiveThreadInfo {
            event_id: "$root_unread:example.com".to_string(),
            sender_id: "@charlie:example.com".to_string(),
            sender_name: "Charlie".to_string(),
            avatar_url: None,
            timestamp: "14:00".to_string(),
            body: "Unread thread root".to_string(),
            num_replies: 4,
            latest_activity: Some("14:10".to_string()),
            num_unread_messages: 7,
            num_unread_notifications: 3,
            num_unread_mentions: 1,
        });

    let _element = constellation.view_active_threads_panel();
}

#[cfg(test)]
use crate::view::error::view_error;

#[test]
fn test_view_error_renders_without_panicking_with_str() {
    // Smoke test for static str
    let _element = view_error("Test Error");
}

#[test]
fn test_view_error_renders_without_panicking_with_string() {
    // Smoke test for owned String
    let _element = view_error(String::from("Another Test Error"));
}

#[test]
fn test_view_error_renders_without_panicking_with_empty_string() {
    // Smoke test for empty string
    let _element = view_error("");
}

#[test]
fn test_view_error_renders_without_panicking_with_long_string() {
    // Smoke test for long string
    let long_string = "a".repeat(1000);
    let _element = view_error(long_string);
}

#[test]
fn test_view_verification_card_none() {
    use crate::settings::user::VerificationUIState;
    use crate::view::verification::view_verification_card;
    let _element = view_verification_card(&VerificationUIState::None);
}

#[test]
fn test_view_verification_card_request_received() {
    use crate::settings::user::VerificationUIState;
    use crate::view::verification::view_verification_card;
    let _element = view_verification_card(&VerificationUIState::RequestReceived {
        sender: matrix_sdk::ruma::user_id!("@alice:example.com").to_owned(),
        device_id: Some(matrix_sdk::ruma::device_id!("DEVICE1").to_owned()),
    });
    let _element_no_dev = view_verification_card(&VerificationUIState::RequestReceived {
        sender: matrix_sdk::ruma::user_id!("@alice:example.com").to_owned(),
        device_id: None,
    });
}

#[test]
fn test_view_verification_card_waiting() {
    use crate::settings::user::VerificationUIState;
    use crate::view::verification::view_verification_card;
    let _element = view_verification_card(&VerificationUIState::WaitingForOtherDevice);
}

#[test]
fn test_view_verification_card_showing_emojis() {
    use crate::settings::user::VerificationUIState;
    use crate::view::verification::view_verification_card;
    let emojis = vec![
        ("🚀".to_string(), "Rocket".to_string()),
        ("🐶".to_string(), "Dog".to_string()),
    ];
    let _element = view_verification_card(&VerificationUIState::ShowingEmojis(emojis));
}

#[test]
fn test_view_verification_card_done() {
    use crate::settings::user::VerificationUIState;
    use crate::view::verification::view_verification_card;
    let _element = view_verification_card(&VerificationUIState::Done);
}

#[test]
fn test_view_verification_card_cancelled() {
    use crate::settings::user::VerificationUIState;
    use crate::view::verification::view_verification_card;
    let _element = view_verification_card(&VerificationUIState::Cancelled);
}
#[test]
fn test_view_app_renders_without_panicking() {
    let mut constellation = Constellation::mock();
    // Default mock is initializing = true
    {
        let _element = constellation.view_app();
    }

    // Logged out
    constellation.is_initializing = false;
    constellation.user_id = None;
    {
        let _element = constellation.view_app();
    }

    // Logged in (renders PaneGrid)
    constellation.user_id = Some("@user:matrix.org".to_string());
    {
        let _element = constellation.view_app();
    }
}

#[test]
fn test_view_app_with_session_verification_banner() {
    let mut constellation = Constellation::mock();
    constellation.is_initializing = false;
    constellation.user_id = Some("@user:matrix.org".to_string());

    // Prompt with candidate device
    constellation.session_verification_prompt =
        Some(crate::constellation::SessionVerificationPrompt {
            target_device_id: Some("DEVICE2".into()),
        });
    {
        let _element = constellation.view_app();
    }

    // Prompt without candidate device
    constellation.session_verification_prompt =
        Some(crate::constellation::SessionVerificationPrompt {
            target_device_id: None,
        });
    {
        let _element = constellation.view_app();
    }

    // When verification is active in settings, banner is suppressed
    constellation.user_settings.verification_ui_state =
        crate::settings::user::VerificationUIState::WaitingForOtherDevice;
    {
        let _element = constellation.view_app();
    }
}

#[test]
fn test_view_app_with_identity_violation_banner() {
    use matrix_sdk::ruma::user_id;

    let mut constellation = Constellation::mock();
    constellation.is_initializing = false;
    constellation.user_id = Some("@user:matrix.org".to_string());

    constellation
        .identity_violations
        .push(user_id!("@evil:example.com").to_owned());
    {
        let _element = constellation.view_app();
    }

    // Multiple violations
    constellation
        .identity_violations
        .push(user_id!("@bob:example.com").to_owned());
    {
        let _element = constellation.view_app();
    }

    // Both identity violation and session prompt set: identity violation takes precedence
    constellation.session_verification_prompt =
        Some(crate::constellation::SessionVerificationPrompt {
            target_device_id: Some("DEVICE1".into()),
        });
    {
        let _element = constellation.view_app();
    }
}

#[test]
fn test_get_room_name() {
    let mut constellation = Constellation::mock();
    let id1: std::sync::Arc<str> = std::sync::Arc::from("!room1:matrix.org");
    let id2: std::sync::Arc<str> = std::sync::Arc::from("!room2:matrix.org");
    let id3: std::sync::Arc<str> = std::sync::Arc::from("!room3:matrix.org");
    let id4: std::sync::Arc<str> = std::sync::Arc::from("!room4:matrix.org");

    // Case 1: Room exists in list and has a name
    constellation.room_list.push(crate::matrix::RoomData {
        id: id1.clone(),
        name: Some("Room One".to_string()),
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        join_rule: None,
        allowed_spaces: vec![],
        order: None,
        suggested: false,
    });
    constellation.room_index.insert(id1.clone(), 0);

    // Case 2: Room exists in list but has no name, should fall back to cache
    constellation.room_list.push(crate::matrix::RoomData {
        id: id2.clone(),
        name: None,
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        join_rule: None,
        allowed_spaces: vec![],
        order: None,
        suggested: false,
    });
    constellation.room_index.insert(id2.clone(), 1);
    constellation
        .room_name_cache
        .insert(id2.clone(), "Room Two Cache".to_string());

    // Case 3: Room exists in list, no name, and not in cache
    constellation.room_list.push(crate::matrix::RoomData {
        id: id3.clone(),
        name: None,
        last_message: None,
        unread_count: 0,
        unread_count_str: None,
        avatar_url: None,
        room_type: None,
        is_space: false,
        parent_space_id: None,
        join_rule: None,
        allowed_spaces: vec![],
        order: None,
        suggested: false,
    });
    constellation.room_index.insert(id3.clone(), 2);

    // Case 4: Room not in list but in cache
    constellation
        .room_name_cache
        .insert(id4.clone(), "Room Four Cache".to_string());

    assert_eq!(constellation.get_room_name(&id1), Some("Room One"));
    assert_eq!(constellation.get_room_name(&id2), Some("Room Two Cache"));
    assert_eq!(constellation.get_room_name(&id3), None);
    assert_eq!(constellation.get_room_name(&id4), Some("Room Four Cache"));
    assert_eq!(constellation.get_room_name("!nonexistent:matrix.org"), None);
}

#[cfg(test)]
use cosmic::Application;

#[test]
fn test_nav_model_hidden_when_logged_out() {
    let constellation = Constellation::mock();
    assert!(constellation.nav_model().is_none());
}

#[test]
fn test_nav_model_lists_spaces_and_selects() {
    let mut constellation = Constellation::mock();
    constellation.user_id = Some("@user:matrix.org".to_string());
    constellation.room_list = vec![crate::matrix::RoomData {
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
    constellation.rebuild_space_nav_model();

    let model = constellation
        .nav_model()
        .expect("nav model present when logged in");
    // "All rooms" + one space.
    assert_eq!(model.len(), 2);

    // Selecting the space entry through the nav bar path updates state.
    let entities: Vec<_> = model.iter().collect();
    let _ = constellation.on_nav_select(entities[1]);
    assert_eq!(
        constellation.selected_space.as_ref().map(|r| r.as_str()),
        Some("!space1:matrix.org")
    );
    // The logged-in main view renders without the old switcher column.
    let _element = constellation.view_app();
}

#[test]
fn test_view_main_content_renders_with_selected_room_header() {
    let mut constellation = Constellation::mock();
    // Exercises view_room_header, including the action buttons that replaced
    // the room name dropdown menu (#422).
    constellation.selected_room = Some(std::sync::Arc::from("!room:matrix.org"));
    let _element = constellation.view_main_content();
}

#[test]
fn test_view_tabbed_header_renders_multiple_tabs() {
    let mut constellation = Constellation::mock();
    let room_a: std::sync::Arc<str> = std::sync::Arc::from("!a:matrix.org");
    let room_b: std::sync::Arc<str> = std::sync::Arc::from("!b:matrix.org");
    let _ = constellation.update(crate::Message::RoomSelected(room_a.clone()));
    let _ = constellation.update(crate::Message::RoomSelected(room_b.clone()));

    assert_eq!(constellation.open_tabs.len(), 2);
    assert_eq!(
        constellation.selected_room.as_deref(),
        Some("!b:matrix.org")
    );
    let _element = constellation.view_main_content();
    let _header = constellation.view_tabbed_header(&room_b);
}

#[test]
fn test_view_tabbed_header_renders_thread_tab() {
    let mut constellation = Constellation::mock();
    let room_a: std::sync::Arc<str> = std::sync::Arc::from("!a:matrix.org");
    let root_id = matrix_sdk::ruma::EventId::parse("$root_event").unwrap();
    let _ = constellation.update(crate::Message::RoomSelected(room_a.clone()));
    let _ = constellation.update(crate::Message::OpenThread(root_id));

    assert_eq!(constellation.open_tabs.len(), 2);
    assert!(constellation.active_thread_root.is_some());
    let _element = constellation.view_main_content();
    let _header = constellation.view_tabbed_header(&room_a);
}

#[test]
fn test_view_main_content_renders_empty_state_when_no_room_selected() {
    let mut constellation = Constellation::mock();
    // With no room selected, renders standard empty state regardless of unread counts (#482).
    constellation.room_list = vec![
        crate::matrix::RoomData {
            id: std::sync::Arc::from("!unread:matrix.org"),
            name: Some("Busy Room".to_string()),
            last_message: None,
            unread_count: 3,
            unread_count_str: Some("(3)".to_string()),
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            order: None,
            suggested: false,
        },
        crate::matrix::RoomData {
            id: std::sync::Arc::from("!quiet:matrix.org"),
            name: Some("Quiet Room".to_string()),
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
        },
    ];
    let _element = constellation.view_main_content();
}

#[test]
fn test_view_sidebar_all_rooms_renders_rooms_with_activity_section() {
    let mut constellation = Constellation::mock();
    constellation.room_list = vec![
        crate::matrix::RoomData {
            id: std::sync::Arc::from("!unread:matrix.org"),
            name: Some("Busy Room".to_string()),
            last_message: None,
            unread_count: 3,
            unread_count_str: Some("(3)".to_string()),
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            order: None,
            suggested: false,
        },
        crate::matrix::RoomData {
            id: std::sync::Arc::from("!quiet:matrix.org"),
            name: Some("Quiet Room".to_string()),
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
        },
    ];
    constellation.update_filtered_rooms();
    assert_eq!(constellation.selected_space, None);

    let _sidebar = constellation.view_sidebar();

    let selectable = constellation.selectable_sidebar_rooms();
    assert_eq!(selectable.len(), 2);
    // Busy Room (unread_count > 0) comes first, then Quiet Room (#482)
    assert_eq!(selectable[0].as_ref(), "!unread:matrix.org");
    assert_eq!(selectable[1].as_ref(), "!quiet:matrix.org");
}

#[test]
fn test_view_sidebar_all_rooms_renders_without_activity_section_when_all_read() {
    let mut constellation = Constellation::mock();
    constellation.room_list = vec![
        crate::matrix::RoomData {
            id: std::sync::Arc::from("!quiet1:matrix.org"),
            name: Some("Quiet Room 1".to_string()),
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
        },
        crate::matrix::RoomData {
            id: std::sync::Arc::from("!quiet2:matrix.org"),
            name: Some("Quiet Room 2".to_string()),
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
        },
    ];
    constellation.update_filtered_rooms();
    assert_eq!(constellation.selected_space, None);

    let _sidebar = constellation.view_sidebar();

    let selectable = constellation.selectable_sidebar_rooms();
    assert_eq!(selectable.len(), 2);
}

#[test]
fn test_view_main_content_renders_no_room_selected_when_no_unread() {
    let mut constellation = Constellation::mock();
    // With no room selected and no unread messages, renders standard 'No Room Selected' view.
    constellation.room_list = vec![crate::matrix::RoomData {
        id: std::sync::Arc::from("!quiet:matrix.org"),
        name: Some("Quiet Room".to_string()),
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
    }];
    let _element = constellation.view_main_content();
}

#[test]
fn test_view_sidebar_with_selected_space_and_close() {
    let mut constellation = Constellation::mock();
    constellation.user_id = Some("@user:matrix.org".to_string());
    constellation.room_list = vec![
        crate::matrix::RoomData {
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
        },
        crate::matrix::RoomData {
            id: std::sync::Arc::from("!room1:matrix.org"),
            name: Some("Room One".to_string()),
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
    constellation.rebuild_space_nav_model();

    // Select space
    constellation.selected_space =
        Some(matrix_sdk::ruma::RoomId::parse("!space1:matrix.org").unwrap());
    constellation.sync_space_nav_activation();

    // Renders sidebar with space header, and view_app renders PaneGrid with sidebar
    {
        let _sidebar = constellation.view_sidebar();
        let _app = constellation.view_app();
    }
    // Close space switcher (unselect space and hide room list)
    let _ = constellation.update(crate::Message::CloseSpaceSwitcher);
    assert_eq!(constellation.selected_space, None);
    assert!(!constellation.is_room_list_open);

    // Nav bar is deactivated when room list is closed
    assert_eq!(
        constellation
            .space_nav_model
            .position(constellation.space_nav_model.active()),
        None
    );

    // view_app renders only main content (hiding rooms list)
    {
        let _app = constellation.view_app();
    }

    // Now select "All rooms" entry from the nav bar (entry 0)
    let entities: Vec<_> = constellation.space_nav_model.iter().collect();
    let _ = constellation.on_nav_select(entities[0]);
    assert!(constellation.is_room_list_open);
    assert_eq!(constellation.selected_space, None);

    // Active nav bar item is back to position 0 ("All rooms")
    let active = constellation.space_nav_model.active();
    assert_eq!(constellation.space_nav_model.position(active), Some(0));

    // Renders sidebar with close switcher icon, and view_app renders PaneGrid with sidebar
    {
        let _sidebar = constellation.view_sidebar();
        let _app = constellation.view_app();
    }
}
