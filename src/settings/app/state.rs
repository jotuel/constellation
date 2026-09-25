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

impl State {
    pub fn from_config(config: &crate::settings::config::Config) -> Self {
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
}
