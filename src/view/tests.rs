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
fn test_view_main_content_hides_header_while_filtering() {
    let mut constellation = Constellation::mock();
    // An active query hides the room's action-icon header row (#427); the
    // query lives in the window title instead of the content area.
    constellation.selected_room = Some(std::sync::Arc::from("!room:matrix.org"));
    constellation.is_search_active = true;
    constellation.search_query = "needle".to_string();
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

    // An active query owns the title (#427).
    constellation.is_search_active = true;
    constellation.search_query = "the".to_string();
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
fn test_view_main_content_renders_unread_room_cards() {
    let mut constellation = Constellation::mock();
    // #432: with no room selected, joined rooms carrying unread messages
    // render as clickable cards on the empty state.
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
