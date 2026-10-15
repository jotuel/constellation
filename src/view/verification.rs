use crate::Message;
use crate::settings::user::VerificationUIState;
use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::widget::{button, container, icon, text, Column, Row};

pub fn view_verification_card(state: &VerificationUIState) -> Element<'static, Message> {
    let mut card_content = Column::new().spacing(12).align_x(Alignment::Center);

    match state {
        VerificationUIState::None => {
            return Column::new().into();
        }
        VerificationUIState::RequestReceived { sender, device_id } => {
            card_content = view_request_received(card_content, sender, device_id);
        }
        VerificationUIState::WaitingForOtherDevice => {
            card_content = view_waiting_for_other_device(card_content);
        }
        VerificationUIState::ShowingEmojis(emojis) => {
            card_content = view_showing_emojis(card_content, emojis);
        }
        VerificationUIState::Done => {
            card_content = view_done(card_content);
        }
        VerificationUIState::Cancelled => {
            card_content = view_cancelled(card_content);
        }
    }

    let card = container(card_content)
        .style(|theme: &cosmic::Theme| {
            use cosmic::iced::widget::container::Catalog;
            let cosmic = theme.cosmic();
            let mut style = theme.style(&cosmic::theme::Container::Dialog(true));
            style.border.color = cosmic.accent.base.into();
            style.border.width = 1.0;
            style
        })
        .padding(20)
        .max_width(600);

    container(card)
        .width(cosmic::iced::Length::Fill)
        .height(cosmic::iced::Length::Fill)
        .padding(20)
        .align_x(Alignment::Center)
        .align_y(Alignment::Start)
        .into()
}

fn card_header(icon_name: &str, title: String) -> Row<'static, Message, cosmic::Theme, cosmic::Renderer> {
    Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(
            icon::from_name(icon_name)
                .symbolic(true)
                .size(24),
        )
        .push(text::title3(title))
}

fn view_request_received(
    content: Column<'static, Message, cosmic::Theme, cosmic::Renderer>,
    sender: &matrix_sdk::ruma::OwnedUserId,
    device_id: &Option<matrix_sdk::ruma::OwnedDeviceId>,
) -> Column<'static, Message, cosmic::Theme, cosmic::Renderer> {
    let desc = if let Some(dev) = device_id {
        crate::fl!(
            "verification-request-from",
            sender = sender.to_string(),
            device = dev.to_string()
        )
    } else {
        crate::fl!(
            "verification-request-from-device",
            device = sender.to_string()
        )
    };

    let header = card_header(
        "security-high-symbolic",
        crate::fl!("verification-request-received"),
    );

    let actions = Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(
            button::suggested(crate::fl!("accept-verification")).on_press(
                Message::UserSettings(crate::settings::user::Message::AcceptVerification),
            ),
        )
        .push(
            button::destructive(crate::fl!("decline")).on_press(Message::UserSettings(
                crate::settings::user::Message::CancelVerification,
            )),
        )
        .push(
            button::standard(crate::fl!("open-settings"))
                .on_press(Message::OpenSettings(crate::SettingsPanel::UserSessions)),
        );

    content.push(header).push(text::body(desc)).push(actions)
}

fn view_waiting_for_other_device(
    content: Column<'static, Message, cosmic::Theme, cosmic::Renderer>,
) -> Column<'static, Message, cosmic::Theme, cosmic::Renderer> {
    let header = card_header("security-high-symbolic", crate::fl!("verification"));

    let actions = Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(
            button::destructive(crate::fl!("cancel")).on_press(Message::UserSettings(
                crate::settings::user::Message::CancelVerification,
            )),
        )
        .push(
            button::standard(crate::fl!("open-settings"))
                .on_press(Message::OpenSettings(crate::SettingsPanel::UserSessions)),
        );

    content
        .push(header)
        .push(text::body(crate::fl!("waiting-for-other-device")))
        .push(actions)
}

fn view_showing_emojis(
    content: Column<'static, Message, cosmic::Theme, cosmic::Renderer>,
    emojis: &[(String, String)],
) -> Column<'static, Message, cosmic::Theme, cosmic::Renderer> {
    let header = card_header("security-high-symbolic", crate::fl!("verification"));

    let mut emoji_row = Row::new().spacing(16).align_y(Alignment::Center);
    for (symbol, desc) in emojis {
        emoji_row = emoji_row.push(
            Column::new()
                .spacing(4)
                .align_x(Alignment::Center)
                .push(text::body(symbol.clone()).size(32))
                .push(text::body(desc.clone()).size(11)),
        );
    }

    let actions = Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(
            button::suggested(crate::fl!("match")).on_press(Message::UserSettings(
                crate::settings::user::Message::ConfirmEmojis,
            )),
        )
        .push(
            button::destructive(crate::fl!("cancel")).on_press(Message::UserSettings(
                crate::settings::user::Message::CancelVerification,
            )),
        )
        .push(
            button::standard(crate::fl!("open-settings"))
                .on_press(Message::OpenSettings(crate::SettingsPanel::UserSessions)),
        );

    content
        .push(header)
        .push(text::body(crate::fl!("do-emojis-match")))
        .push(emoji_row.wrap())
        .push(actions)
}

fn view_done(
    content: Column<'static, Message, cosmic::Theme, cosmic::Renderer>,
) -> Column<'static, Message, cosmic::Theme, cosmic::Renderer> {
    let header = card_header("emblem-ok-symbolic", crate::fl!("verification"));

    let actions = Row::new().spacing(10).align_y(Alignment::Center).push(
        button::suggested(crate::fl!("done")).on_press(Message::UserSettings(
            crate::settings::user::Message::DismissVerification,
        )),
    );

    content
        .push(header)
        .push(text::body(crate::fl!("verification-successful")))
        .push(actions)
}

fn view_cancelled(
    content: Column<'static, Message, cosmic::Theme, cosmic::Renderer>,
) -> Column<'static, Message, cosmic::Theme, cosmic::Renderer> {
    let header = card_header("dialog-warning-symbolic", crate::fl!("verification"));

    let actions = Row::new().spacing(10).align_y(Alignment::Center).push(
        button::standard(crate::fl!("dismiss")).on_press(Message::UserSettings(
            crate::settings::user::Message::DismissVerification,
        )),
    );

    content
        .push(header)
        .push(text::body(crate::fl!("verification-cancelled")))
        .push(actions)
}
