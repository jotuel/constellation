use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::iced::widget::image;
use cosmic::widget::{Column, Row, Widget, button, container, divider};

use crate::utils::widget::tooltip_button_at;
use cosmic::widget::tooltip::Position;

use crate::{CONSTELLATION_ICON, Constellation, Message, matrix};

impl Constellation {
    pub fn view_app(&self) -> Element<'_, Message> {
        if self.is_initializing {
            return Self::view_initializing();
        }

        if self.user_id.is_none() {
            return self.view_login();
        }

        let main_view = Row::new()
            .push(self.view_space_switcher())
            .push(divider::vertical::default())
            .push(self.view_sidebar())
            .push(divider::vertical::default())
            .push(self.view_main_content())
            .padding(4);

        let mut final_view: Element<'_, Message> = main_view.into();

        if let Some(sync_overlay) = self.view_sync_overlay() {
            final_view = cosmic::iced::widget::stack![final_view, sync_overlay].into();
        }

        if let Some(image_overlay) = self.view_fullscreen_image_overlay() {
            final_view = cosmic::iced::widget::stack![final_view, image_overlay].into();
        }

        if let Some(error_overlay) = self.view_error_overlay() {
            final_view = cosmic::iced::widget::stack![final_view, error_overlay].into();
        }

        final_view
    }

    fn view_initializing() -> Element<'static, Message> {
        let content = Column::new()
            .push(
                cosmic::widget::svg(cosmic::widget::svg::Handle::from_memory(CONSTELLATION_ICON))
                    .width(cosmic::iced::Length::Fixed(128.0))
                    .height(cosmic::iced::Length::Fixed(128.0)),
            )
            .push(cosmic::widget::progress_bar::indeterminate_circular())
            .spacing(32)
            .align_x(Alignment::Center);

        container(content)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }

    fn view_sync_overlay(&self) -> Option<Element<'_, Message>> {
        if !(self.app_settings.show_sync_indicator && self.is_sync_indicator_active) {
            return None;
        }

        let sync_widget: Element<'_, Message> = match self.sync_status {
            matrix::SyncStatus::Syncing => {
                container(cosmic::widget::progress_bar::indeterminate_circular().size(24.0)).into()
            }
            matrix::SyncStatus::Connected => {
                container(cosmic::widget::icon::from_name("network-idle-symbolic").size(24)).into()
            }
            matrix::SyncStatus::Disconnected => {
                container(cosmic::widget::icon::from_name("network-offline-symbolic").size(24))
                    .into()
            }
            matrix::SyncStatus::Error(_) | matrix::SyncStatus::MissingSlidingSyncSupport => {
                container(cosmic::widget::icon::from_name("network-error-symbolic").size(24)).into()
            }
        };

        Some(
            container(sync_widget)
                .padding(20)
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .align_x(Alignment::End)
                .align_y(Alignment::End)
                .into(),
        )
    }

    fn view_fullscreen_image_overlay(&self) -> Option<Element<'_, Message>> {
        let handle = self.fullscreen_image.as_ref()?;

        let image: image::Image<'_> = cosmic::widget::image(handle.clone())
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .content_fit(cosmic::iced::ContentFit::Contain);
        let image_viewer = container(image)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .padding(40)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center);

        let close_button = container(tooltip_button_at(
            button::icon(cosmic::widget::icon::from_name("window-close-symbolic"))
                .on_press(Message::CloseImage),
            crate::fl!("close-image"),
            Position::Bottom,
        ))
        .width(cosmic::iced::Length::Fill)
        .height(cosmic::iced::Length::Fill)
        .padding(10)
        .align_right(image_viewer.size_hint().width)
        .align_top(image_viewer.size_hint().height);

        // Overlay that closes on click
        let dismiss_overlay = button::custom(
            container(cosmic::iced::widget::Space::new())
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill),
        )
        .on_press(Message::CloseImage)
        .padding(0);

        Some(cosmic::iced::widget::stack![dismiss_overlay, image_viewer, close_button].into())
    }

    fn view_error_overlay(&self) -> Option<Element<'_, Message>> {
        let sliding_sync_error = matches!(
            self.sync_status,
            matrix::SyncStatus::MissingSlidingSyncSupport
        );

        if let Some(error) = self.error.as_deref() {
            Some(crate::view::error::view_error(error))
        } else if sliding_sync_error {
            Some(crate::view::error::view_error(crate::fl!(
                "error-no-sliding-sync"
            )))
        } else if let matrix::SyncStatus::Error(e) = &self.sync_status {
            Some(crate::view::error::view_error(e.as_str()))
        } else {
            None
        }
    }
}
