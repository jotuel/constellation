use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::widget::{Column, Row, button, icon, text};

/// Standardized category overview row with title, live summary, and a drill-down chevron.
pub fn category_row<'a, M: Clone + 'static>(
    title: impl Into<String>,
    summary: impl Into<String>,
    on_press: M,
) -> Element<'a, M> {
    let radii = cosmic::theme::active().cosmic().corner_radii.radius_m;
    button::custom(
        Row::new()
            .align_y(Alignment::Center)
            .spacing(12)
            .push(
                Column::new()
                    .spacing(2)
                    .push(text::body(title.into()))
                    .push(text::caption(summary.into()))
                    .width(cosmic::iced::Length::Fill),
            )
            .push(icon::from_name("go-next-symbolic").symbolic(true)),
    )
    .class(cosmic::theme::Button::ListItem(radii))
    .width(cosmic::iced::Length::Fill)
    .on_press(on_press)
    .into()
}

/// Standardized overview header card displaying an avatar, title, caption, and drill-down chevron.
pub fn header_card<'a, M: Clone + 'static>(
    avatar: impl Into<Element<'a, M>>,
    title: impl Into<String>,
    subtitle: impl Into<String>,
    on_press: M,
) -> Element<'a, M> {
    let radii = cosmic::theme::active().cosmic().corner_radii.radius_m;
    button::custom(
        Row::new()
            .spacing(16)
            .align_y(Alignment::Center)
            .push(avatar.into())
            .push(
                Column::new()
                    .spacing(4)
                    .push(text::title3(title.into()))
                    .push(text::caption(subtitle.into()))
                    .width(cosmic::iced::Length::Fill),
            )
            .push(icon::from_name("go-next-symbolic").symbolic(true)),
    )
    .class(cosmic::theme::Button::ListItem(radii))
    .width(cosmic::iced::Length::Fill)
    .on_press(on_press)
    .into()
}

/// Uniform 64x64 avatar container rendering an image handle if present or a centered fallback text container.
pub fn avatar_box<'a, M: 'a>(
    handle: Option<&cosmic::iced::widget::image::Handle>,
    fallback_text: impl Into<String>,
) -> Element<'a, M> {
    if let Some(h) = handle {
        cosmic::widget::image(h.clone())
            .width(cosmic::iced::Length::Fixed(64.0))
            .height(cosmic::iced::Length::Fixed(64.0))
            .into()
    } else {
        cosmic::widget::container(text::body(fallback_text.into()).size(24))
            .width(cosmic::iced::Length::Fixed(64.0))
            .height(cosmic::iced::Length::Fixed(64.0))
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }
}
