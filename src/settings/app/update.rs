use cosmic::{Action, Task};

use super::message::Message;
use super::state::State;

impl State {
    pub fn update(&mut self, message: Message) -> Task<Action<crate::Message>> {
        match message {
            Message::OpenPanel(panel) => {
                Task::done(Action::from(crate::Message::OpenSettings(panel)))
            }
            Message::ToggleSyncIndicator(show) => {
                self.show_sync_indicator = show;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleTypingNotifications(send) => {
                self.send_typing_notifications = send;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleMarkdown(render) => {
                self.render_markdown = render;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleCompactMode(compact) => {
                self.compact_mode = compact;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleHideThreadedMessages(hide) => {
                self.hide_threaded_messages = hide;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ToggleAutoplayVideos(autoplay) => {
                self.autoplay_videos = autoplay;
                Task::done(Action::from(crate::Message::AppSettingChanged))
            }
            Message::ClearCache => Task::done(Action::from(crate::Message::AppSettings(
                Message::ClearCache,
            ))),
            Message::OpenShortcuts => Task::done(Action::from(crate::Message::OpenSettings(
                crate::SettingsPanel::Shortcuts,
            ))),
            Message::ClearSessionErrors => {
                self.session_errors.clear();
                Task::none()
            }
            Message::DismissSessionError(index) => {
                if index < self.session_errors.len() {
                    self.session_errors.remove(index);
                }
                Task::none()
            }
        }
    }
}
