use super::*;

use std::sync::Arc;

#[test]
fn test_name_changed() {
    let mut state = State::default();
    let _ = state.update(Message::NameChanged("New Room Name".to_string()), &None);
    assert_eq!(state.name, "New Room Name");
}

#[test]
fn test_topic_changed() {
    let mut state = State::default();
    let _ = state.update(Message::TopicChanged("New Topic".to_string()), &None);
    assert_eq!(state.topic, "New Topic");
}

#[test]
fn test_load_room_no_matrix() {
    let mut state = State::default();
    let room_id: Arc<str> = Arc::from("!some_room:example.com");
    let _ = state.update(Message::LoadRoom(room_id.clone()), &None);

    // Without matrix engine, it shouldn't try to load
    assert!(!state.is_loading);
    assert_eq!(state.room_id, None);
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
fn test_invite_user_id_changed() {
    let mut state = State::default();
    let _ = state.update(
        Message::InviteUserIdChanged("@user:example.com".to_string()),
        &None,
    );
    assert_eq!(state.invite_user_id, "@user:example.com");
}

#[test]
fn test_action_reason_changed() {
    let mut state = State::default();
    let _ = state.update(Message::ActionReasonChanged("Spam".to_string()), &None);
    assert_eq!(state.action_reason, "Spam");
}

#[test]
fn test_member_filter_changed() {
    let mut state = State::default();
    let _ = state.update(Message::MemberFilterChanged("John".to_string()), &None);
    assert_eq!(state.member_filter, "John");
}

#[test]
fn test_join_rule_changed() {
    use matrix_sdk::ruma::events::room::join_rules::JoinRule;
    let mut state = State {
        room_id: Some(Arc::from("!room:example.com")),
        ..Default::default()
    };
    // This won't actually call the engine since we pass None, but we can check if it returns a Task
    let _task = state.update(Message::JoinRuleChanged(JoinRule::Public), &None);
    // The task should be none since matrix engine is None
    // We can't easily inspect Task, but we can verify it compiles and runs.
}

#[test]
fn test_history_visibility_changed() {
    use matrix_sdk::ruma::events::room::history_visibility::HistoryVisibility;
    let mut state = State {
        room_id: Some(Arc::from("!room:example.com")),
        ..Default::default()
    };
    // This won't actually call the engine since we pass None
    let _task = state.update(
        Message::HistoryVisibilityChanged(HistoryVisibility::Shared),
        &None,
    );
}

#[test]
fn test_restricted_space_id_changed() {
    let mut state = State::default();
    let _ = state.update(
        Message::RestrictedSpaceIdChanged("!space:example.com".to_string()),
        &None,
    );
    assert_eq!(state.restricted_space_id, "!space:example.com");
}

#[test]
fn test_join_rule_changed_knock() {
    use matrix_sdk::ruma::events::room::join_rules::JoinRule;
    let mut state = State {
        room_id: Some(Arc::from("!room:example.com")),
        ..Default::default()
    };
    let _ = state.update(Message::JoinRuleChanged(JoinRule::Knock), &None);
    assert_eq!(state.join_rule, Some(JoinRule::Knock));
}

#[test]
fn test_aliases_changed() {
    let mut state = State::default();

    // Test canonical alias change
    let _ = state.update(
        Message::CanonicalAliasChanged("#new:example.com".to_string()),
        &None,
    );
    assert_eq!(state.canonical_alias, "#new:example.com");

    // Test alt alias input
    let _ = state.update(
        Message::NewAltAliasInputChanged("#alt1:example.com".to_string()),
        &None,
    );
    assert_eq!(state.new_alt_alias_input, "#alt1:example.com");

    // Test alt alias addition
    let _ = state.update(Message::AltAliasAdded, &None);
    assert_eq!(state.alt_aliases, vec!["#alt1:example.com".to_string()]);
    assert_eq!(state.new_alt_alias_input, "");

    // Test alt alias removal
    let _ = state.update(
        Message::AltAliasRemoved("#alt1:example.com".to_string()),
        &None,
    );
    assert!(state.alt_aliases.is_empty());
}

#[test]
fn test_pending_power_level() {
    let mut state = State::default();
    let _ = state.update(
        Message::PendingPowerLevel("@user:example.com".to_string(), 75),
        &None,
    );
    assert_eq!(
        state.pending_power_level,
        Some(("@user:example.com".to_string(), 75))
    );
}

#[test]
fn test_commit_power_level_no_matrix() {
    let mut state = State {
        pending_power_level: Some(("@user:example.com".to_string(), 50)),
        ..Default::default()
    };
    let _ = state.update(
        Message::CommitPowerLevel("@user:example.com".to_string()),
        &None,
    );
    // Without a matrix engine the commit is a no-op, but the draft is consumed.
    assert!(state.pending_power_level.is_none());
    assert!(state.updating_power_level_for.is_none());
}

#[test]
fn test_commit_power_level_guarded_while_updating() {
    let mut state = State {
        pending_power_level: Some(("@user:example.com".to_string(), 50)),
        updating_power_level_for: Some("@user:example.com".to_string()),
        ..Default::default()
    };
    let _ = state.update(
        Message::CommitPowerLevel("@user:example.com".to_string()),
        &None,
    );
    // Already updating this user: the commit is ignored and the draft is preserved.
    assert_eq!(
        state.pending_power_level,
        Some(("@user:example.com".to_string(), 50))
    );
}

#[test]
fn test_notification_mode_changed_syncs_segmented_model() {
    use matrix_sdk::notification_settings::RoomNotificationMode;

    let mut state = State::default();
    assert_eq!(state.notification_mode, None);

    // Change to MentionsAndKeywordsOnly
    let _ = state.update(
        Message::NotificationModeChanged(RoomNotificationMode::MentionsAndKeywordsOnly),
        &None,
    );
    assert_eq!(
        state.notification_mode,
        Some(RoomNotificationMode::MentionsAndKeywordsOnly)
    );
    assert_eq!(
        state.notification_selector.model.active(),
        state.notification_selector.entities[1]
    );

    // Change to Mute
    let _ = state.update(
        Message::NotificationModeChanged(RoomNotificationMode::Mute),
        &None,
    );
    assert_eq!(state.notification_mode, Some(RoomNotificationMode::Mute));
    assert_eq!(
        state.notification_selector.model.active(),
        state.notification_selector.entities[2]
    );

    // Change to AllMessages
    let _ = state.update(
        Message::NotificationModeChanged(RoomNotificationMode::AllMessages),
        &None,
    );
    assert_eq!(
        state.notification_mode,
        Some(RoomNotificationMode::AllMessages)
    );
    assert_eq!(
        state.notification_selector.model.active(),
        state.notification_selector.entities[0]
    );
}

#[test]
fn test_room_loaded_syncs_notification_model() {
    use matrix_sdk::notification_settings::RoomNotificationMode;

    let mut state = State::default();
    let info = RoomInfo {
        name: "Test Room".to_string(),
        topic: "Topic".to_string(),
        avatar_url: None,
        membership: matrix_sdk::RoomState::Joined,
        ban_level: 50,
        invite_level: 50,
        kick_level: 50,
        redact_level: 50,
        events_default_level: 0,
        room_name_level: 50,
        room_topic_level: 50,
        room_avatar_level: 50,
        current_user_id: Some("@alice:example.com".to_string()),
        notification_mode: Some(RoomNotificationMode::Mute),
        join_rule: None,
        history_visibility: None,
        ignored_users: Vec::new(),
        is_encrypted: true,
        canonical_alias: Some("#room:example.com".to_string()),
        alt_aliases: vec!["#alias2:example.com".to_string()],
    };

    let _ = state.update(Message::RoomLoaded(Box::new(Ok(info))), &None);
    assert_eq!(state.name, "Test Room");
    assert_eq!(state.notification_mode, Some(RoomNotificationMode::Mute));
    assert_eq!(
        state.notification_selector.model.active(),
        state.notification_selector.entities[2]
    );
    assert_eq!(state.canonical_alias, "#room:example.com");
    assert_eq!(state.alt_aliases, vec!["#alias2:example.com".to_string()]);
    assert!(state.is_encrypted);
}

#[test]
fn test_open_panel_messages() {
    let panels = [
        crate::SettingsPanel::RoomProfile,
        crate::SettingsPanel::RoomNotifications,
        crate::SettingsPanel::RoomSecurity,
        crate::SettingsPanel::RoomPacks,
        crate::SettingsPanel::Permissions,
        crate::SettingsPanel::ManageRoomMembers,
    ];

    for panel in panels {
        let msg = Message::OpenPanel(panel.clone());
        match msg {
            Message::OpenPanel(p) => assert_eq!(p, panel),
            _ => panic!("Expected OpenPanel"),
        }
    }
}

#[test]
fn test_room_settings_subpages_view_smoke() {
    let state = State {
        name: "Smoke Test Room".to_string(),
        canonical_alias: "#smoke:example.com".to_string(),
        notification_mode: Some(
            matrix_sdk::notification_settings::RoomNotificationMode::AllMessages,
        ),
        ..Default::default()
    };
    // Exercise all view methods to ensure no panics or invalid element generation
    let _ = state.view_overview();
    let _ = state.view();
    let _ = state.view_profile_page();
    let _ = state.view_notifications_page();
    let _ = state.view_security_page();
    let _ = state.view_permissions_page();
    let _ = state.view_packs_page();
    let _ = state.view_manage();
}
