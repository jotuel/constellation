use super::*;

use matrix_sdk::encryption::verification::{SasState, VerificationRequestState};
use std::sync::Arc;

#[test]
fn test_display_name_changed() {
    let mut state = State::default();
    let _ = state.update(Message::DisplayNameChanged("John Doe".to_string()), &None);
    assert_eq!(state.display_name, "John Doe");
}

#[test]
fn test_dismiss_error() {
    let mut state = State {
        error: Some("Error".to_string()),
        ..Default::default()
    };
    let _ = state.update(Message::DismissError, &None);
    assert_eq!(state.error, None);
}

#[test]
fn test_devices_loaded() {
    let mut state = State {
        is_loading_devices: true,
        ..Default::default()
    };

    let devices = vec![DeviceInfo {
        device_id: Arc::from("DEV1"),
        display_name: Some("Test Device".to_string()),
        is_verified: true,
        is_current: true,
        is_renaming: false,
        edit_name: String::new(),
        is_deleting: false,
    }];

    let _ = state.update(Message::DevicesLoaded(Ok(devices.clone())), &None);
    assert!(!state.is_loading_devices);
    assert_eq!(state.devices.len(), 1);
    assert_eq!(state.devices[0].device_id.as_ref(), "DEV1");
    assert!(state.devices[0].is_verified);
    assert!(state.devices[0].is_current);

    state.is_loading_devices = true;
    let _ = state.update(
        Message::DevicesLoaded(Err("network error".to_string())),
        &None,
    );
    assert!(!state.is_loading_devices);
    assert_eq!(
        state.error,
        Some("Failed to load devices: network error".to_string())
    );
}

#[test]
fn test_verification_dismiss() {
    let mut state = State {
        verification_ui_state: VerificationUIState::Done,
        ..Default::default()
    };
    let _ = state.update(Message::DismissVerification, &None);
    assert_eq!(state.verification_ui_state, VerificationUIState::None);
    assert!(state.active_sas.is_none());
    assert!(state.active_verification_request.is_none());
}

#[test]
fn test_verification_cancel() {
    let mut state = State {
        verification_ui_state: VerificationUIState::WaitingForOtherDevice,
        ..Default::default()
    };
    let _ = state.update(Message::CancelVerification, &None);
    assert_eq!(state.verification_ui_state, VerificationUIState::Cancelled);
    assert!(state.active_sas.is_none());
    assert!(state.active_verification_request.is_none());
}

#[test]
fn test_verification_request_state_changed_done() {
    let mut state = State {
        verification_ui_state: VerificationUIState::WaitingForOtherDevice,
        ..Default::default()
    };
    let _ = state.update(
        Message::VerificationRequestStateChanged(VerificationRequestState::Done),
        &None,
    );
    assert_eq!(state.verification_ui_state, VerificationUIState::Done);
    assert!(state.active_sas.is_none());
    assert!(state.active_verification_request.is_none());
}

#[test]
fn test_sas_state_changed_done() {
    let mut state = State {
        verification_ui_state: VerificationUIState::ShowingEmojis(vec![(
            "🚀".to_string(),
            "Rocket".to_string(),
        )]),
        ..Default::default()
    };
    let _ = state.update(
        Message::SasStateChanged(SasState::Done {
            verified_devices: vec![],
            verified_identities: vec![],
        }),
        &None,
    );
    assert_eq!(state.verification_ui_state, VerificationUIState::Done);
    assert!(state.active_sas.is_none());
    assert!(state.active_verification_request.is_none());
}

#[test]
fn test_emojis_confirmed_error() {
    let mut state = State {
        verification_ui_state: VerificationUIState::ShowingEmojis(vec![]),
        ..Default::default()
    };
    let _ = state.update(
        Message::EmojisConfirmed(Err("confirmation failed".to_string())),
        &None,
    );
    assert_eq!(
        state.error,
        Some("Failed to confirm emojis: confirmation failed".to_string())
    );
}

#[test]
fn test_accept_verification_no_active_req() {
    let mut state = State {
        verification_ui_state: VerificationUIState::RequestReceived {
            sender: matrix_sdk::ruma::user_id!("@alice:example.com").to_owned(),
            device_id: None,
        },
        ..Default::default()
    };
    let _ = state.update(Message::AcceptVerification, &None);
    // Without active_verification_request, remains in current state
    assert!(matches!(
        state.verification_ui_state,
        VerificationUIState::RequestReceived { .. }
    ));
}

#[test]
fn test_open_panel_message_handling() {
    let mut state = State::default();
    // OpenPanel message is a pure routing message handled by the parent constellation
    // update handler; state.update handles it without panicking or modifying internal state.
    let task = state.update(Message::OpenPanel(crate::SettingsPanel::UserProfile), &None);
    // Task is empty (Task::none)
    drop(task);
}

#[test]
fn test_subpage_views_render() {
    let state = State {
        display_name: "Alice".to_string(),
        threepids: vec![Threepid {
            address: "alice@example.com".to_string(),
            medium: matrix_sdk::ruma::thirdparty::Medium::Email,
        }],
        global_notification_mode_dm: Some(
            matrix_sdk::notification_settings::RoomNotificationMode::AllMessages,
        ),
        global_notification_mode_group: Some(
            matrix_sdk::notification_settings::RoomNotificationMode::MentionsAndKeywordsOnly,
        ),
        keywords: vec!["urgent".to_string(), "review".to_string()],
        devices: vec![DeviceInfo {
            device_id: Arc::from("DEV1"),
            display_name: Some("Laptop".to_string()),
            is_verified: true,
            is_current: true,
            is_renaming: false,
            edit_name: String::new(),
            is_deleting: false,
        }],
        ..Default::default()
    };

    // Verify all subpage view methods return Elements without panicking
    let _ = state.view_overview();
    let _ = state.view_profile_page();
    let _ = state.view_notifications_page();
    let _ = state.view_privacy_page();
    let _ = state.view_sessions_page();
    let _ = state.view_account_page();
    let _ = state.view_packs_page();
    let _ = state.view();
}

#[test]
fn test_notification_mode_changed_updates_models() {
    let mut state = State::default();
    assert_eq!(state.global_notification_mode_dm, None);

    let _ = state.update(
        Message::GlobalNotificationModeChanged(
            true,
            matrix_sdk::notification_settings::RoomNotificationMode::Mute,
        ),
        &None,
    );
    assert_eq!(
        state.global_notification_mode_dm,
        Some(matrix_sdk::notification_settings::RoomNotificationMode::Mute)
    );

    let _ = state.update(
        Message::GlobalNotificationModeLoaded(
            false,
            matrix_sdk::notification_settings::RoomNotificationMode::AllMessages,
        ),
        &None,
    );
    assert_eq!(
        state.global_notification_mode_group,
        Some(matrix_sdk::notification_settings::RoomNotificationMode::AllMessages)
    );
}
