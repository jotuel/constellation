use matrix_sdk::encryption::CrossSigningStatus;
use matrix_sdk::encryption::verification::{SasVerification, VerificationRequest};
use matrix_sdk::ruma::{OwnedDeviceId, OwnedUserId};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct CrossSigningInfo {
    pub status: CrossSigningStatus,
    pub master_key: Option<String>,
    pub self_signing_key: Option<String>,
    pub user_signing_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub device_id: Arc<str>,
    pub display_name: Option<String>,
    pub is_verified: bool,
    pub is_current: bool,
    pub is_renaming: bool,
    pub edit_name: String,
    pub is_deleting: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Threepid {
    pub address: String,
    pub medium: matrix_sdk::ruma::thirdparty::Medium,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub enum VerificationUIState {
    #[default]
    None,
    RequestReceived {
        sender: OwnedUserId,
        device_id: Option<OwnedDeviceId>,
    },
    WaitingForOtherDevice,
    ShowingEmojis(Vec<(String, String)>),
    Done,
    Cancelled,
}

pub struct State {
    pub display_name: String,
    pub original_display_name: String,
    pub is_loading: bool,
    pub is_saving: bool,
    pub error: Option<String>,
    pub avatar_url: Option<String>,
    pub avatar_handle: Option<cosmic::iced::widget::image::Handle>,
    pub is_uploading_avatar: bool,
    pub is_loading_avatar: bool,
    pub current_password: String,
    pub new_password: String,
    pub confirm_new_password: String,
    pub is_changing_password: bool,
    pub password_success: Option<String>,
    pub success_message: Option<String>,
    pub devices: Vec<DeviceInfo>,
    pub is_loading_devices: bool,
    pub active_verification_request: Option<VerificationRequest>,
    pub active_sas: Option<SasVerification>,
    pub verification_ui_state: VerificationUIState,
    pub device_delete_password: String,
    pub global_notification_mode_dm:
        Option<matrix_sdk::notification_settings::RoomNotificationMode>,
    pub global_notification_mode_group:
        Option<matrix_sdk::notification_settings::RoomNotificationMode>,
    pub dm_notification_model: cosmic::widget::segmented_button::SingleSelectModel,
    pub dm_notification_entities: [cosmic::widget::segmented_button::Entity; 3],
    pub group_notification_model: cosmic::widget::segmented_button::SingleSelectModel,
    pub group_notification_entities: [cosmic::widget::segmented_button::Entity; 3],
    pub is_loading_global_notifications: bool,
    pub deactivate_password: String,
    pub is_deactivating: bool,
    pub cross_signing_info: Option<CrossSigningInfo>,
    pub is_loading_cross_signing: bool,
    pub is_bootstrapping: bool,
    pub media_previews_display_policy: bool,
    pub invite_avatars_display_policy: bool,
    pub threepids: Vec<Threepid>,
    pub is_loading_3pids: bool,
    pub new_3pid_email: String,
    pub new_3pid_msisdn: String,
    pub new_3pid_country_code: String,
    pub is_requesting_3pid_token: bool,
    pub adding_3pid_sid: Option<String>,
    pub adding_3pid_client_secret: Option<String>,
    pub add_3pid_password: String,
    pub keywords: Vec<String>,
    pub new_keyword: String,
    pub is_loading_keywords: bool,
    pub ignored_users: Vec<OwnedUserId>,
    pub is_loading_ignored_users: bool,
    pub new_ignore_user_id: String,
    pub subscribed_packs: Vec<(matrix_sdk::ruma::OwnedRoomId, String)>,
    pub is_loading_subscribed_packs: bool,
}

impl Default for State {
    fn default() -> Self {
        let (dm_m, dm_e) = create_notification_mode_model(None);
        let (grp_m, grp_e) = create_notification_mode_model(None);
        Self {
            display_name: String::new(),
            original_display_name: String::new(),
            is_loading: false,
            is_saving: false,
            error: None,
            avatar_url: None,
            avatar_handle: None,
            is_uploading_avatar: false,
            is_loading_avatar: false,
            current_password: String::new(),
            new_password: String::new(),
            confirm_new_password: String::new(),
            is_changing_password: false,
            password_success: None,
            success_message: None,
            devices: Vec::new(),
            is_loading_devices: false,
            active_verification_request: None,
            active_sas: None,
            verification_ui_state: VerificationUIState::default(),
            device_delete_password: String::new(),
            global_notification_mode_dm: None,
            global_notification_mode_group: None,
            dm_notification_model: dm_m,
            dm_notification_entities: dm_e,
            group_notification_model: grp_m,
            group_notification_entities: grp_e,
            is_loading_global_notifications: false,
            deactivate_password: String::new(),
            is_deactivating: false,
            cross_signing_info: None,
            is_loading_cross_signing: false,
            is_bootstrapping: false,
            media_previews_display_policy: true,
            invite_avatars_display_policy: true,
            threepids: Vec::new(),
            is_loading_3pids: false,
            new_3pid_email: String::new(),
            new_3pid_msisdn: String::new(),
            new_3pid_country_code: String::new(),
            is_requesting_3pid_token: false,
            adding_3pid_sid: None,
            adding_3pid_client_secret: None,
            add_3pid_password: String::new(),
            keywords: Vec::new(),
            new_keyword: String::new(),
            is_loading_keywords: false,
            ignored_users: Vec::new(),
            is_loading_ignored_users: false,
            new_ignore_user_id: String::new(),
            subscribed_packs: Vec::new(),
            is_loading_subscribed_packs: false,
        }
    }
}

impl State {
    pub fn from_config(config: &crate::settings::config::Config) -> Self {
        Self {
            media_previews_display_policy: config.media_previews_display_policy,
            invite_avatars_display_policy: config.invite_avatars_display_policy,
            ..Default::default()
        }
    }
}

pub fn create_notification_mode_model(
    current_mode: Option<matrix_sdk::notification_settings::RoomNotificationMode>,
) -> (
    cosmic::widget::segmented_button::SingleSelectModel,
    [cosmic::widget::segmented_button::Entity; 3],
) {
    use matrix_sdk::notification_settings::RoomNotificationMode;
    let mut model = cosmic::widget::segmented_button::SingleSelectModel::default();
    let e_all = model
        .insert()
        .text(crate::fl!("notification-mode-all-messages"))
        .data(RoomNotificationMode::AllMessages)
        .id();
    let e_mentions = model
        .insert()
        .text(crate::fl!("notification-mode-mentions-only"))
        .data(RoomNotificationMode::MentionsAndKeywordsOnly)
        .id();
    let e_mute = model
        .insert()
        .text(crate::fl!("notification-mode-muted"))
        .data(RoomNotificationMode::Mute)
        .id();
    match current_mode {
        Some(RoomNotificationMode::AllMessages) => model.activate(e_all),
        Some(RoomNotificationMode::MentionsAndKeywordsOnly) => model.activate(e_mentions),
        Some(RoomNotificationMode::Mute) => model.activate(e_mute),
        None => {}
    }
    (model, [e_all, e_mentions, e_mute])
}

impl Clone for State {
    fn clone(&self) -> Self {
        let (dm_m, dm_e) = create_notification_mode_model(self.global_notification_mode_dm);
        let (grp_m, grp_e) = create_notification_mode_model(self.global_notification_mode_group);
        Self {
            display_name: self.display_name.clone(),
            original_display_name: self.original_display_name.clone(),
            is_loading: self.is_loading,
            is_saving: self.is_saving,
            error: self.error.clone(),
            avatar_url: self.avatar_url.clone(),
            avatar_handle: self.avatar_handle.clone(),
            is_uploading_avatar: self.is_uploading_avatar,
            is_loading_avatar: self.is_loading_avatar,
            current_password: self.current_password.clone(),
            new_password: self.new_password.clone(),
            confirm_new_password: self.confirm_new_password.clone(),
            is_changing_password: self.is_changing_password,
            password_success: self.password_success.clone(),
            success_message: self.success_message.clone(),
            devices: self.devices.clone(),
            is_loading_devices: self.is_loading_devices,
            active_verification_request: self.active_verification_request.clone(),
            active_sas: self.active_sas.clone(),
            verification_ui_state: self.verification_ui_state.clone(),
            device_delete_password: self.device_delete_password.clone(),
            global_notification_mode_dm: self.global_notification_mode_dm,
            global_notification_mode_group: self.global_notification_mode_group,
            dm_notification_model: dm_m,
            dm_notification_entities: dm_e,
            group_notification_model: grp_m,
            group_notification_entities: grp_e,
            is_loading_global_notifications: self.is_loading_global_notifications,
            deactivate_password: self.deactivate_password.clone(),
            is_deactivating: self.is_deactivating,
            cross_signing_info: self.cross_signing_info.clone(),
            is_loading_cross_signing: self.is_loading_cross_signing,
            is_bootstrapping: self.is_bootstrapping,
            media_previews_display_policy: self.media_previews_display_policy,
            invite_avatars_display_policy: self.invite_avatars_display_policy,
            threepids: self.threepids.clone(),
            is_loading_3pids: self.is_loading_3pids,
            new_3pid_email: self.new_3pid_email.clone(),
            new_3pid_msisdn: self.new_3pid_msisdn.clone(),
            new_3pid_country_code: self.new_3pid_country_code.clone(),
            is_requesting_3pid_token: self.is_requesting_3pid_token,
            adding_3pid_sid: self.adding_3pid_sid.clone(),
            adding_3pid_client_secret: self.adding_3pid_client_secret.clone(),
            add_3pid_password: self.add_3pid_password.clone(),
            keywords: self.keywords.clone(),
            new_keyword: self.new_keyword.clone(),
            is_loading_keywords: self.is_loading_keywords,
            ignored_users: self.ignored_users.clone(),
            is_loading_ignored_users: self.is_loading_ignored_users,
            new_ignore_user_id: self.new_ignore_user_id.clone(),
            subscribed_packs: self.subscribed_packs.clone(),
            is_loading_subscribed_packs: self.is_loading_subscribed_packs,
        }
    }
}

impl std::fmt::Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("State")
            .field("display_name", &self.display_name)
            .field(
                "global_notification_mode_dm",
                &self.global_notification_mode_dm,
            )
            .field(
                "global_notification_mode_group",
                &self.global_notification_mode_group,
            )
            .finish_non_exhaustive()
    }
}
