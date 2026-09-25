use cosmic::Element;
use matrix_sdk::notification_settings::RoomNotificationMode;

/// Encapsulated 3-way notification mode selector (`All Messages`, `Mentions and Keywords Only`, `Mute`)
/// for room and user settings panels.
pub struct NotificationModeSelector {
    pub mode: Option<RoomNotificationMode>,
    pub model: cosmic::widget::segmented_button::SingleSelectModel,
    pub entities: [cosmic::widget::segmented_button::Entity; 3],
}

impl std::fmt::Debug for NotificationModeSelector {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NotificationModeSelector")
            .field("mode", &self.mode)
            .finish()
    }
}

impl NotificationModeSelector {
    /// Creates a new selector initialized to the given notification mode.
    pub fn new(mode: Option<RoomNotificationMode>) -> Self {
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

        match mode {
            Some(RoomNotificationMode::AllMessages) => model.activate(e_all),
            Some(RoomNotificationMode::MentionsAndKeywordsOnly) => model.activate(e_mentions),
            Some(RoomNotificationMode::Mute) => model.activate(e_mute),
            None => {}
        }

        Self {
            mode,
            model,
            entities: [e_all, e_mentions, e_mute],
        }
    }

    /// Returns the currently selected notification mode, if any.
    pub fn mode(&self) -> Option<RoomNotificationMode> {
        self.mode
    }

    /// Synchronizes the selector to a new notification mode and updates the active segmented button.
    pub fn set_mode(&mut self, mode: Option<RoomNotificationMode>) {
        self.mode = mode;
        match mode {
            Some(RoomNotificationMode::AllMessages) => self.model.activate(self.entities[0]),
            Some(RoomNotificationMode::MentionsAndKeywordsOnly) => {
                self.model.activate(self.entities[1])
            }
            Some(RoomNotificationMode::Mute) => self.model.activate(self.entities[2]),
            None => {}
        }
    }

    /// Returns the segmented control widget.
    pub fn control<M: Clone + 'static>(
        &self,
        is_loading: bool,
        on_change: impl Fn(RoomNotificationMode) -> M + 'static,
    ) -> Element<'_, M> {
        let mut ctrl = cosmic::widget::segmented_control::horizontal(&self.model);
        if !is_loading {
            let entities = self.entities;
            ctrl = ctrl.on_activate(move |entity| {
                let mode = if entity == entities[0] {
                    RoomNotificationMode::AllMessages
                } else if entity == entities[1] {
                    RoomNotificationMode::MentionsAndKeywordsOnly
                } else {
                    RoomNotificationMode::Mute
                };
                on_change(mode)
            });
        }
        ctrl.into()
    }
}

impl Default for NotificationModeSelector {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Clone for NotificationModeSelector {
    fn clone(&self) -> Self {
        Self::new(self.mode)
    }
}

impl PartialEq for NotificationModeSelector {
    fn eq(&self, other: &Self) -> bool {
        self.mode == other.mode
    }
}
