use crate::Message;
use crate::settings::user::VerificationUIState;
use cosmic::Element;
use cosmic::iced::Alignment;
use cosmic::widget::{Column, Row, button, container, icon, text};

pub fn view_verification_card(state: &VerificationUIState) -> Element<'static, Message> {
    let mut card_content = Column::new().spacing(12).align_x(Alignment::Center);

    match state {
        VerificationUIState::None => {
            return Column::new().into();
        }
        VerificationUIState::RequestReceived { sender, device_id } => {
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

            let header = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    icon::from_name("security-high-symbolic")
                        .symbolic(true)
                        .size(24),
                )
                .push(text::title3(crate::fl!("verification-request-received")));

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
                        .on_press(Message::OpenSettings(crate::SettingsPanel::User)),
                );

            card_content = card_content
                .push(header)
                .push(text::body(desc))
                .push(actions);
        }
        VerificationUIState::WaitingForOtherDevice => {
            let header = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    icon::from_name("security-high-symbolic")
                        .symbolic(true)
                        .size(24),
                )
                .push(text::title3(crate::fl!("verification")));

            let actions =
                Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(button::destructive(crate::fl!("cancel")).on_press(
                        Message::UserSettings(crate::settings::user::Message::CancelVerification),
                    ))
                    .push(
                        button::standard(crate::fl!("open-settings"))
                            .on_press(Message::OpenSettings(crate::SettingsPanel::User)),
                    );

            card_content = card_content
                .push(header)
                .push(text::body(crate::fl!("waiting-for-other-device")))
                .push(actions);
        }
        VerificationUIState::ShowingEmojis(emojis) => {
            let header = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    icon::from_name("security-high-symbolic")
                        .symbolic(true)
                        .size(24),
                )
                .push(text::title3(crate::fl!("verification")));

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

            let actions =
                Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(
                        button::suggested(crate::fl!("match")).on_press(Message::UserSettings(
                            crate::settings::user::Message::ConfirmEmojis,
                        )),
                    )
                    .push(button::destructive(crate::fl!("cancel")).on_press(
                        Message::UserSettings(crate::settings::user::Message::CancelVerification),
                    ))
                    .push(
                        button::standard(crate::fl!("open-settings"))
                            .on_press(Message::OpenSettings(crate::SettingsPanel::User)),
                    );

            card_content = card_content
                .push(header)
                .push(text::body(crate::fl!("do-emojis-match")))
                .push(emoji_row.wrap())
                .push(actions);
        }
        VerificationUIState::Done => {
            let header = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    icon::from_name("emblem-ok-symbolic")
                        .symbolic(true)
                        .size(24),
                )
                .push(text::title3(crate::fl!("verification")));

            let actions = Row::new().spacing(10).align_y(Alignment::Center).push(
                button::suggested(crate::fl!("done")).on_press(Message::UserSettings(
                    crate::settings::user::Message::DismissVerification,
                )),
            );

            card_content = card_content
                .push(header)
                .push(text::body(crate::fl!("verification-successful")))
                .push(actions);
        }
        VerificationUIState::Cancelled => {
            let header = Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(
                    icon::from_name("dialog-warning-symbolic")
                        .symbolic(true)
                        .size(24),
                )
                .push(text::title3(crate::fl!("verification")));

            let actions = Row::new().spacing(10).align_y(Alignment::Center).push(
                button::standard(crate::fl!("dismiss")).on_press(Message::UserSettings(
                    crate::settings::user::Message::DismissVerification,
                )),
            );

            card_content = card_content
                .push(header)
                .push(text::body(crate::fl!("verification-cancelled")))
                .push(actions);
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
