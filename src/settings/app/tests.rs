use super::*;

#[test]
fn test_update_toggle_sync_indicator() {
    let mut state = State::default();
    assert!(!state.show_sync_indicator);

    let _ = state.update(Message::ToggleSyncIndicator(true));
    assert!(state.show_sync_indicator);

    let _ = state.update(Message::ToggleSyncIndicator(false));
    assert!(!state.show_sync_indicator);
}

#[test]
fn test_update_toggle_typing_notifications() {
    let mut state = State::default();
    assert!(!state.send_typing_notifications);

    let _ = state.update(Message::ToggleTypingNotifications(true));
    assert!(state.send_typing_notifications);

    let _ = state.update(Message::ToggleTypingNotifications(false));
    assert!(!state.send_typing_notifications);
}

#[test]
fn test_update_toggle_markdown() {
    let mut state = State::default();
    assert!(!state.render_markdown);

    let _ = state.update(Message::ToggleMarkdown(true));
    assert!(state.render_markdown);

    let _ = state.update(Message::ToggleMarkdown(false));
    assert!(!state.render_markdown);
}

#[test]
fn test_update_toggle_compact_mode() {
    let mut state = State::default();
    assert!(!state.compact_mode);

    let _ = state.update(Message::ToggleCompactMode(true));
    assert!(state.compact_mode);

    let _ = state.update(Message::ToggleCompactMode(false));
    assert!(!state.compact_mode);
}

#[test]
fn test_update_toggle_hide_threaded_messages() {
    let mut state = State::default();
    assert!(!state.hide_threaded_messages);

    let _ = state.update(Message::ToggleHideThreadedMessages(true));
    assert!(state.hide_threaded_messages);

    let _ = state.update(Message::ToggleHideThreadedMessages(false));
    assert!(!state.hide_threaded_messages);
}

#[test]
fn test_update_toggle_autoplay_videos() {
    let mut state = State::default();
    assert!(!state.autoplay_videos);

    let _ = state.update(Message::ToggleAutoplayVideos(true));
    assert!(state.autoplay_videos);

    let _ = state.update(Message::ToggleAutoplayVideos(false));
    assert!(!state.autoplay_videos);
}

#[test]
fn test_update_clear_cache() {
    let mut state = State::default();
    let _ = state.update(Message::ClearCache);
    assert!(!state.show_sync_indicator);
    assert!(!state.send_typing_notifications);
    assert!(!state.render_markdown);
    assert!(!state.compact_mode);
}

#[test]
fn test_update_open_panel() {
    let mut state = State::default();
    let _ = state.update(Message::OpenPanel(crate::SettingsPanel::AppAppearance));
    let _ = state.update(Message::OpenShortcuts);
}

#[test]
fn test_push_and_clear_session_errors() {
    let mut state = State::default();
    assert!(state.session_errors.is_empty());

    state.push_session_error("Error 1");
    state.push_session_error("Error 2");
    assert_eq!(state.session_errors.len(), 2);
    assert_eq!(state.session_errors[0].message, "Error 1");
    assert_eq!(state.session_errors[1].message, "Error 2");
    assert!(!state.session_errors[0].formatted_time().is_empty());

    let _ = state.update(Message::DismissSessionError(0));
    assert_eq!(state.session_errors.len(), 1);
    assert_eq!(state.session_errors[0].message, "Error 2");

    let _ = state.update(Message::ClearSessionErrors);
    assert!(state.session_errors.is_empty());
}

#[test]
fn test_dismiss_session_error_out_of_bounds() {
    let mut state = State::default();
    state.push_session_error("Error 1");
    let _ = state.update(Message::DismissSessionError(5));
    assert_eq!(state.session_errors.len(), 1);
}

#[test]
fn test_view_renders_with_and_without_session_errors() {
    let mut state = State::default();
    let _ = state.view();
    let _ = state.view_overview();
    let _ = state.view_appearance_page();
    let _ = state.view_notifications_page();
    let _ = state.view_maintenance_page();

    state.push_session_error("Test error message");
    let _ = state.view();
    let _ = state.view_overview();
    let _ = state.view_appearance_page();
    let _ = state.view_notifications_page();
    let _ = state.view_maintenance_page();
}
