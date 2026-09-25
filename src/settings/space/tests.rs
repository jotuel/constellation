use super::*;

#[test]
fn test_name_changed() {
    let mut state = State::default();
    let _ = state.update(Message::NameChanged("New Space Name".to_string()), &None);
    assert_eq!(state.name, "New Space Name");
}

#[test]
fn test_topic_changed() {
    let mut state = State::default();
    let _ = state.update(Message::TopicChanged("New Topic".to_string()), &None);
    assert_eq!(state.topic, "New Topic");
}

#[test]
fn test_canonical_alias_changed() {
    let mut state = State::default();
    let _ = state.update(
        Message::CanonicalAliasChanged("#new_alias:example.com".to_string()),
        &None,
    );
    assert_eq!(state.canonical_alias, "#new_alias:example.com");
}

#[test]
fn test_dismiss_error() {
    let mut state = State {
        error: Some("An error occurred".to_string()),
        ..Default::default()
    };
    let _ = state.update(Message::DismissError, &None);
    assert_eq!(state.error, None);
}

#[test]
fn test_child_filter_changed() {
    let mut state = State::default();
    let _ = state.update(Message::ChildFilterChanged("test".to_string()), &None);
    assert_eq!(state.child_filter, "test");
}

#[test]
fn test_open_panel_messages() {
    let panels = [
        crate::SettingsPanel::SpaceProfile,
        crate::SettingsPanel::SpaceAccess,
        crate::SettingsPanel::ManageSpaceRooms,
    ];

    for panel in panels {
        let msg = Message::OpenPanel(panel.clone());
        match msg {
            Message::OpenPanel(p) => assert_eq!(p, panel),
            _ => panic!("Expected OpenPanel"),
        }
        let mut state = State::default();
        let _ = state.update(Message::OpenPanel(panel), &None);
    }
}

#[test]
fn test_space_settings_subpages_view_smoke() {
    let state = State {
        name: "Smoke Test Space".to_string(),
        canonical_alias: "#smoke_space:example.com".to_string(),
        topic: "Space Topic".to_string(),
        is_public: true,
        is_invite_only: false,
        error: Some("Test Error".to_string()),
        ..Default::default()
    };

    // Exercise all view methods to ensure no panics
    let _ = state.view_overview();
    let _ = state.view();
    let _ = state.view_profile_page();
    let _ = state.view_access_page();
    let _ = state.view_manage();
}

#[test]
fn test_space_settings_loading_view() {
    let state = State {
        is_loading: true,
        ..Default::default()
    };

    let _ = state.view_overview();
    let _ = state.view();
}

#[test]
fn test_space_settings_with_children() {
    let state = State {
        name: "Space with children".to_string(),
        children: vec![crate::matrix::RoomData {
            id: std::sync::Arc::from("!room1:example.com"),
            name: Some("Child Room 1".to_string()),
            last_message: None,
            unread_count: 0,
            unread_count_str: None,
            avatar_url: None,
            room_type: None,
            is_space: false,
            parent_space_id: None,
            join_rule: None,
            allowed_spaces: Vec::new(),
            order: Some("0".to_string()),
            suggested: true,
        }],
        ..Default::default()
    };

    let _ = state.view_overview();
    let _ = state.view_manage();
}
