use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::widget::{Row, button, icon, settings, text};

use crate::settings::widgets::category_row;
use crate::utils::widget::tooltip_button;

use super::message::Message;
use super::state::State;

impl State {
    pub fn view(&self) -> Element<'_, Message> {
        self.view_overview()
    }

    pub fn view_overview(&self) -> Element<'_, Message> {
        let markdown_str = if self.render_markdown {
            crate::fl!("enabled")
        } else {
            crate::fl!("disabled")
        };
        let compact_str = if self.compact_mode {
            crate::fl!("enabled")
        } else {
            crate::fl!("disabled")
        };
        let appearance_summary = crate::fl!(
            "app-appearance-summary",
            markdown = markdown_str,
            compact = compact_str
        );

        let typing_str = if self.send_typing_notifications {
            crate::fl!("enabled")
        } else {
            crate::fl!("disabled")
        };
        let notif_summary = if self.session_errors.is_empty() {
            crate::fl!("app-notifications-summary-no-errors", typing = typing_str)
        } else {
            crate::fl!(
                "app-notifications-summary",
                typing = typing_str,
                errors = self.session_errors.len()
            )
        };

        let maintenance_summary = crate::fl!("app-maintenance-summary");

        settings::view_column(vec![
            category_row(
                crate::fl!("app-appearance-display"),
                appearance_summary,
                Message::OpenPanel(crate::SettingsPanel::AppAppearance),
            ),
            category_row(
                crate::fl!("app-notifications-diagnostics"),
                notif_summary,
                Message::OpenPanel(crate::SettingsPanel::AppNotifications),
            ),
            category_row(
                crate::fl!("app-maintenance-shortcuts"),
                maintenance_summary,
                Message::OpenPanel(crate::SettingsPanel::AppMaintenance),
            ),
        ])
        .into()
    }

    pub fn view_appearance_page(&self) -> Element<'_, Message> {
        settings::view_column(vec![
            settings::section()
                .title(crate::fl!("app-appearance-display"))
                .add(settings::item(
                    crate::fl!("compact-mode"),
                    cosmic::widget::toggler(self.compact_mode)
                        .on_toggle(Message::ToggleCompactMode),
                ))
                .add(settings::item(
                    crate::fl!("render-markdown"),
                    cosmic::widget::toggler(self.render_markdown)
                        .on_toggle(Message::ToggleMarkdown),
                ))
                .add(settings::item(
                    crate::fl!("show-sync-indicator"),
                    cosmic::widget::toggler(self.show_sync_indicator)
                        .on_toggle(Message::ToggleSyncIndicator),
                ))
                .add(settings::item(
                    crate::fl!("autoplay-videos"),
                    cosmic::widget::toggler(self.autoplay_videos)
                        .on_toggle(Message::ToggleAutoplayVideos),
                ))
                .add(settings::item(
                    crate::fl!("hide-threaded-messages"),
                    cosmic::widget::toggler(self.hide_threaded_messages)
                        .on_toggle(Message::ToggleHideThreadedMessages),
                ))
                .into(),
        ])
        .into()
    }

    pub fn view_notifications_page(&self) -> Element<'_, Message> {
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

        settings::view_column(vec![notifications_section.into()]).into()
    }

    pub fn view_maintenance_page(&self) -> Element<'_, Message> {
        settings::view_column(vec![
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
