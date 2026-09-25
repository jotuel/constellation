use super::*;
use matrix_sdk::notification_settings::RoomNotificationMode;

#[derive(Debug, Clone, PartialEq, Eq)]
enum TestMessage {
    Open,
    Dismiss,
    Save,
    ModeChanged(RoomNotificationMode),
}

#[test]
fn test_category_row_construction() {
    let row = category_row("General", "Manage general settings", TestMessage::Open);
    let _ = row;
}

#[test]
fn test_header_card_construction() {
    let avatar = avatar_box(None, "U");
    let card = header_card(avatar, "Alice", "@alice:example.com", TestMessage::Open);
    let _ = card;
}

#[test]
fn test_avatar_box_variants() {
    let fallback = avatar_box::<TestMessage>(None, "No Avatar");
    let _ = fallback;
}

#[test]
fn test_view_error_rendering() {
    // When error is present
    let error_text = "Failed to update profile".to_string();
    let err_element = view_error(Some(&error_text), TestMessage::Dismiss);
    assert!(err_element.is_some());

    // When error is None
    let none_element = view_error::<TestMessage>(None, TestMessage::Dismiss);
    assert!(none_element.is_none());
}

#[test]
fn test_save_button_states() {
    // Clean and not saving
    let btn_clean = save_button(false, false, TestMessage::Save);
    let _ = btn_clean;

    // Dirty and not saving
    let btn_dirty = save_button(false, true, TestMessage::Save);
    let _ = btn_dirty;

    // In-progress saving
    let btn_saving = save_button(true, true, TestMessage::Save);
    let _ = btn_saving;
}

#[test]
fn test_notification_mode_selector_initialization() {
    let sel_none = NotificationModeSelector::new(None);
    assert_eq!(sel_none.mode(), None);

    let sel_all = NotificationModeSelector::new(Some(RoomNotificationMode::AllMessages));
    assert_eq!(sel_all.mode(), Some(RoomNotificationMode::AllMessages));
    assert_eq!(sel_all.model.active(), sel_all.entities[0]);

    let sel_mentions =
        NotificationModeSelector::new(Some(RoomNotificationMode::MentionsAndKeywordsOnly));
    assert_eq!(
        sel_mentions.mode(),
        Some(RoomNotificationMode::MentionsAndKeywordsOnly)
    );
    assert_eq!(sel_mentions.model.active(), sel_mentions.entities[1]);

    let sel_mute = NotificationModeSelector::new(Some(RoomNotificationMode::Mute));
    assert_eq!(sel_mute.mode(), Some(RoomNotificationMode::Mute));
    assert_eq!(sel_mute.model.active(), sel_mute.entities[2]);
}

#[test]
fn test_notification_mode_selector_set_mode() {
    let mut sel = NotificationModeSelector::default();
    assert_eq!(sel.mode(), None);

    sel.set_mode(Some(RoomNotificationMode::AllMessages));
    assert_eq!(sel.mode(), Some(RoomNotificationMode::AllMessages));
    assert_eq!(sel.model.active(), sel.entities[0]);

    sel.set_mode(Some(RoomNotificationMode::MentionsAndKeywordsOnly));
    assert_eq!(
        sel.mode(),
        Some(RoomNotificationMode::MentionsAndKeywordsOnly)
    );
    assert_eq!(sel.model.active(), sel.entities[1]);

    sel.set_mode(Some(RoomNotificationMode::Mute));
    assert_eq!(sel.mode(), Some(RoomNotificationMode::Mute));
    assert_eq!(sel.model.active(), sel.entities[2]);

    sel.set_mode(None);
    assert_eq!(sel.mode(), None);
}

#[test]
fn test_notification_mode_selector_clone() {
    let mut sel =
        NotificationModeSelector::new(Some(RoomNotificationMode::MentionsAndKeywordsOnly));
    let cloned = sel.clone();

    assert_eq!(cloned.mode(), sel.mode());
    assert_eq!(cloned.model.active(), cloned.entities[1]);
    assert_eq!(sel, cloned);

    // Modifying the original should not affect the clone
    sel.set_mode(Some(RoomNotificationMode::Mute));
    assert_eq!(sel.mode(), Some(RoomNotificationMode::Mute));
    assert_eq!(
        cloned.mode(),
        Some(RoomNotificationMode::MentionsAndKeywordsOnly)
    );
}

#[test]
fn test_notification_mode_selector_control_smoke() {
    let sel = NotificationModeSelector::new(Some(RoomNotificationMode::AllMessages));
    let ctrl_loading = sel.control(true, TestMessage::ModeChanged);
    let _ = ctrl_loading;

    let ctrl_active = sel.control(false, TestMessage::ModeChanged);
    let _ = ctrl_active;
}
