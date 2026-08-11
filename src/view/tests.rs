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
    let constellation = Constellation::mock();
    let _element = constellation.view_app();
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
    constellation.room_name_cache.insert(id2.clone(), "Room Two Cache".to_string());

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
    constellation.room_name_cache.insert(id4.clone(), "Room Four Cache".to_string());

    assert_eq!(constellation.get_room_name(&id1), Some("Room One"));
    assert_eq!(constellation.get_room_name(&id2), Some("Room Two Cache"));
    assert_eq!(constellation.get_room_name(&id3), None);
    assert_eq!(constellation.get_room_name(&id4), Some("Room Four Cache"));
    assert_eq!(constellation.get_room_name("!nonexistent:matrix.org"), None);
}
