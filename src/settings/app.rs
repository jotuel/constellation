use crate::utils::widget::tooltip_button;
use cosmic::iced::Alignment;
use cosmic::widget::{Row, button, icon, settings, text};
use cosmic::{Action, Element, Task};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionError {
    pub timestamp: chrono::DateTime<chrono::Local>,
    pub message: String,
}

impl SessionError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Local::now(),
            message: message.into(),
        }
    }

    pub fn formatted_time(&self) -> String {
        self.timestamp.format("%Y-%m-%d %H:%M:%S").to_string()
    }
}

#[derive(Debug, Clone, Default)]
pub struct State {
    pub show_sync_indicator: bool,
    pub send_typing_notifications: bool,
    pub render_markdown: bool,
    pub compact_mode: bool,
    pub hide_threaded_messages: bool,
    pub autoplay_videos: bool,
    pub session_errors: Vec<SessionError>,
}
#[derive(Debug, Clone)]
pub enum Message {
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
impl State {
    pub fn from_config(config: &super::config::Config) -> Self {
        Self {
            show_sync_indicator: config.show_sync_indicator,
            send_typing_notifications: config.send_typing_notifications,
            render_markdown: config.render_markdown,
            compact_mode: config.compact_mode,
            hide_threaded_messages: config.hide_threaded_messages,
            autoplay_videos: config.autoplay_videos,
            session_errors: Vec::new(),
        }
    }

    pub fn push_session_error(&mut self, message: impl Into<String>) {
        self.session_errors.push(SessionError::new(message));
    }
    pub fn update(&mut self, message: Message) -> Task<Action<crate::Message>> {
        match message {
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

    pub fn view(&self) -> Element<'_, Message> {
        let mut notifications_section =
            settings::section()
                .title(crate::fl!("notifications"))
                .add(settings::item(
                    crate::fl!("send-typing-notifications"),
                    cosmic::widget::toggler(self.send_typing_notifications)
                        .on_toggle(Message::ToggleTypingNotifications),
                ));

        if self.session_errors.is_empty() {
            notifications_section = notifications_section.add(settings::item(
                crate::fl!("session-errors"),
                text::body(crate::fl!("no-session-errors")),
            ));
        } else {
            notifications_section = notifications_section.add(settings::item(
                crate::fl!("session-errors"),
                button::destructive(crate::fl!("clear-all")).on_press(Message::ClearSessionErrors),
            ));
            for (idx, err) in self.session_errors.iter().enumerate().rev() {
                let control = Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(text::caption(err.formatted_time()))
                    .push(tooltip_button(
                        button::custom(icon::from_name("window-close-symbolic").symbolic(true))
                            .class(cosmic::theme::Button::Destructive)
                            .on_press(Message::DismissSessionError(idx)),
                        crate::fl!("dismiss"),
                    ));

                notifications_section =
                    notifications_section.add(settings::item(err.message.clone(), control));
            }
        }

        settings::view_column(vec![
            settings::section()
                .title(crate::fl!("general-settings"))
                .add(settings::item(
                    crate::fl!("show-sync-indicator"),
                    cosmic::widget::toggler(self.show_sync_indicator)
                        .on_toggle(Message::ToggleSyncIndicator),
                ))
                .add(settings::item(
                    crate::fl!("render-markdown"),
                    cosmic::widget::toggler(self.render_markdown)
                        .on_toggle(Message::ToggleMarkdown),
                ))
                .add(settings::item(
                    crate::fl!("compact-mode"),
                    cosmic::widget::toggler(self.compact_mode)
                        .on_toggle(Message::ToggleCompactMode),
                ))
                .add(settings::item(
                    crate::fl!("hide-threaded-messages"),
                    cosmic::widget::toggler(self.hide_threaded_messages)
                        .on_toggle(Message::ToggleHideThreadedMessages),
                ))
                .add(settings::item(
                    crate::fl!("autoplay-videos"),
                    cosmic::widget::toggler(self.autoplay_videos)
                        .on_toggle(Message::ToggleAutoplayVideos),
                ))
                .into(),
            notifications_section.into(),
            settings::section()
                .title(crate::fl!("maintenance"))
                .add(settings::item(
                    crate::fl!("media-cache"),
                    button::text(crate::fl!("clear-cache")).on_press(Message::ClearCache),
                ))
                .add(settings::item(
                    crate::fl!("shortcuts-open-page"),
                    button::text(crate::fl!("shortcuts-open")).on_press(Message::OpenShortcuts),
                ))
                .into(),
        ])
        .into()
    }
}

#[cfg(test)]
mod tests {
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
        // State doesn't change for ClearCache, just returns a task
        assert!(!state.show_sync_indicator);
        assert!(!state.send_typing_notifications);
        assert!(!state.render_markdown);
        assert!(!state.compact_mode);
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

        state.push_session_error("Test error message");
        let _ = state.view();
    }
}
