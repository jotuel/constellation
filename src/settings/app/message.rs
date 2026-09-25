#[derive(Debug, Clone)]
pub enum Message {
    OpenPanel(crate::SettingsPanel),
    ToggleSyncIndicator(bool),
    ToggleTypingNotifications(bool),
    ToggleMarkdown(bool),
    ToggleCompactMode(bool),
    ToggleHideThreadedMessages(bool),
    ToggleAutoplayVideos(bool),
    ClearCache,
    /// Navigate to the keyboard-shortcuts page.
    OpenShortcuts,
    ClearSessionErrors,
    DismissSessionError(usize),
}
