use chrono::{DateTime, DurationRound, TimeDelta};
use cosmic::{
    Element, Theme,
    iced::{Alignment, widget::scrollable},
    widget::{
        Column, RcElementWrapper, Row,
        button::{self, icon},
        container, divider,
        icon::Named,
        menu, tab_bar,
        text::{self, body},
        text_editor::text_editor,
        text_input,
        tooltip::Position,
    },
};
use matrix_sdk::ruma::events::room::{MediaSource, message::MessageType};
use matrix_sdk_ui::timeline::{TimelineDetails, TimelineEventItemId};

use crate::constellation::{Tab, scroll};
#[cfg(feature = "video-player")]
use crate::view::PLAY_VIDEO;
use crate::{
    Constellation, MenuAct, Message, PreviewEvent, fl, matrix,
    utils::widget::{disabled_or_tooltip, tooltip_button, tooltip_button_at},
    view::{
        ADD_REACTION, DOWNLOAD_AUDIO, DOWNLOAD_FILE, DOWNLOAD_IMAGE, DOWNLOAD_VIDEO, IGNORE,
        REPLIES, REPLY, TOOLTIP_ATTACH, TOOLTIP_COPY_LINK, TOOLTIP_COPY_ROOM_LINK, TOOLTIP_DELETE,
        TOOLTIP_EDIT, TOOLTIP_EMOJIS, TOOLTIP_FIND, TOOLTIP_LOCATION, TOOLTIP_REPLY,
        TOOLTIP_THREAD, UNIGNORE_USER,
    },
};

/// Wrap a timeline row in a container tagged with its stable anchor key so
/// `constellation::scroll` can measure and address it.
fn tagged_row<'a>(
    element: Element<'a, Message>,
    item: &crate::ConstellationItem,
    prefix: &'static str,
) -> Element<'a, Message> {
    match item.scroll_key() {
        Some(key) => container(element).id(scroll::row_id(prefix, &key)).into(),
        None => element,
    }
}

impl<'chat> Constellation {
    pub(crate) fn is_search_filtering(&self) -> bool {
        self.active_tab().is_some_and(|t| t.is_search())
    }

    pub fn view_timeline(&self) -> Element<'_, Message> {
        let selected_room_data = self
            .selected_room
            .as_ref()
            .and_then(|id| self.room_by_id(id));

        let is_video_room = selected_room_data
            .map(|r| {
                r.room_type
                    .as_ref()
                    .is_some_and(|t| t.as_str() == "org.matrix.msc3401.call.room")
            })
            .unwrap_or(false);

        if is_video_room && let Some(room) = selected_room_data {
            return self.view_video_room(room);
        }

        let mut timeline = Column::new().spacing(10).width(cosmic::iced::Length::Fill);

        let mut pending_date_divider: Option<matrix_sdk::ruma::MilliSecondsSinceUnixEpoch> = None;

        for item in &self.timeline_items {
            if item.item.is_none() {
                // Render simulated/mock items!
                let element = self.view_item(
                    item,
                    &self.thread_counts,
                    &self.event_id_to_index,
                    &self.thread_root_to_last_index,
                );
                timeline = timeline.push(tagged_row(element, item, scroll::MAIN_ROW_PREFIX));
                continue;
            }

            if let Some(timeline_item) = &item.item
                && let Some(event) = timeline_item.as_event()
                && event.content().as_message().is_some()
            {
                // View-side thread filtering
                if self.app_settings.hide_threaded_messages && item.thread_root_id.is_some() {
                    continue;
                }

                if let Some(date) = pending_date_divider.take() {
                    timeline = timeline.push(
                        container(
                            Row::new()
                                .push(divider::horizontal::default())
                                .push(body(
                                    DateTime::from_timestamp_secs(date.as_secs().into())
                                        .unwrap_or_default()
                                        .duration_trunc(TimeDelta::try_days(1).unwrap_or_default())
                                        .unwrap_or_default()
                                        .to_rfc2822()
                                        .trim_end_matches(" 00:00:00 +0000")
                                        .to_owned(),
                                ))
                                .push(divider::horizontal::default())
                                .align_y(Alignment::Center),
                        )
                        .id(scroll::row_id(
                            scroll::MAIN_ROW_PREFIX,
                            &format!("d:{}", date.as_secs()),
                        )),
                    );
                }

                let element = self.view_item(
                    item,
                    &self.thread_counts,
                    &self.event_id_to_index,
                    &self.thread_root_to_last_index,
                );
                timeline = timeline.push(tagged_row(element, item, scroll::MAIN_ROW_PREFIX));
            } else if let Some(timeline_item) = &item.item
                && let Some(matrix::VirtualTimelineItem::DateDivider(date)) =
                    timeline_item.as_virtual()
            {
                pending_date_divider = Some(*date);
            }
        }

        scrollable(timeline)
            .id(crate::TIMELINE_ID.clone())
            .height(cosmic::iced::Length::Fill)
            .on_scroll(|viewport| Message::TimelineScrolled(viewport, false))
            .into()
    }

    /// Persistent banner shown while the room is viewed through an
    /// event-focused (permalink context) timeline. Explains that the user is
    /// not at the newest messages and offers a one-click return to live.
    fn view_older_messages_banner(&self) -> Element<'_, Message> {
        let label = body(fl!("viewing-older-messages"));
        let jump_btn = button::text(fl!("jump-to-newest")).on_press(Message::ReturnToLive);
        // Wrap in a container so the banner reads as a distinct, themed strip
        // (uses the default theme surface rather than a hardcoded color).
        container(
            Row::new()
                .spacing(10)
                .align_y(Alignment::Center)
                .push(label)
                .push(jump_btn),
        )
        .padding([6, 10])
        .width(cosmic::iced::Length::Fill)
        .into()
    }

    fn view_reactions<'reaction>(
        &'reaction self,
        event: &'reaction matrix_sdk_ui::timeline::EventTimelineItem,
        item_id: &matrix::TimelineEventItemId,
    ) -> Row<'reaction, Message, cosmic::Theme> {
        let mut reaction_row = Row::new().spacing(5).align_y(Alignment::Center);
        let reactions = event.content().reactions();

        if let Some(reaction) = reactions {
            for (key, senders) in reaction.iter() {
                let count = senders.len();

                let is_me_reacted = if let Some(me) = &self.user_id {
                    senders.keys().any(|user_id| user_id.as_str() == me)
                } else {
                    false
                };

                let btn_content =
                    container(body(format!("{} {}", key, count)).size(10)).padding([2, 4]);

                // We can differentiate style if reacted, but for now we just wrap in button.
                let btn = button::custom(btn_content)
                    .on_press(Message::ToggleReaction(item_id.to_owned(), key.clone()));

                // If `is_me_reacted` is true, we could style it differently
                if is_me_reacted {
                    // Use standard button to give it some background highlight, or specific style
                    // But custom button with standard background works nicely if we could pass theme
                }

                reaction_row = reaction_row.push(btn);
            }
        }

        reaction_row
    }

    fn view_emoji_picker<'emoji>(
        &'emoji self,
        item_id: Option<matrix::TimelineEventItemId>,
    ) -> Element<'emoji, Message> {
        let search_placeholder = if item_id.is_none()
            && self.emoji_picker_tab == crate::constellation::EmojiPickerTab::Stickers
        {
            fl!("search-stickers")
        } else {
            fl!("search-emojis")
        };

        let search_input = text_input(search_placeholder, &self.emoji_search_query)
            .on_input(Message::EmojiSearchQueryChanged)
            .width(cosmic::iced::Length::Fill);

        let close_btn = icon(Named::new("window-close-symbolic")).on_press(if item_id.is_some() {
            Message::OpenReactionPicker(None)
        } else {
            Message::ToggleEmojiPicker
        });

        let close_btn_tooltip = tooltip_button_at(close_btn, fl!("close-picker"), Position::Bottom);

        let top_row = Row::new()
            .spacing(5)
            .align_y(Alignment::Center)
            .push(search_input)
            .push(close_btn_tooltip);

        let mut picker_col = Column::new().spacing(8);
        picker_col = picker_col.push(top_row);

        // Tabs for composer picker (Emojis vs Stickers)
        if item_id.is_none() {
            let is_emojis = self.emoji_picker_tab == crate::constellation::EmojiPickerTab::Emojis;
            let emojis_btn = if is_emojis {
                button::suggested(fl!("emojis"))
            } else {
                button::text(fl!("emojis")).on_press(Message::SelectEmojiPickerTab(
                    crate::constellation::EmojiPickerTab::Emojis,
                ))
            };
            let stickers_btn = if !is_emojis {
                button::suggested(fl!("stickers"))
            } else {
                button::text(fl!("stickers")).on_press(Message::SelectEmojiPickerTab(
                    crate::constellation::EmojiPickerTab::Stickers,
                ))
            };
            let tabs_row = Row::new().spacing(8).push(emojis_btn).push(stickers_btn);
            picker_col = picker_col.push(tabs_row);
        }

        if item_id.is_none()
            && self.emoji_picker_tab == crate::constellation::EmojiPickerTab::Stickers
        {
            // --- STICKERS TAB ---
            let mut sticker_grid = Row::new().spacing(6);
            let mut has_stickers = false;
            let mut count = 0;

            let filter_is_ascii = self.emoji_search_query.is_ascii();
            let filter_lower_fallback =
                (!filter_is_ascii).then(|| self.emoji_search_query.to_lowercase());

            for sticker in &self.active_stickers {
                if self.emoji_search_query.is_empty()
                    || crate::contains_ignore_ascii_case(
                        &sticker.shortcode,
                        &self.emoji_search_query,
                        filter_lower_fallback.as_deref(),
                    )
                    || crate::contains_ignore_ascii_case(
                        &sticker.body,
                        &self.emoji_search_query,
                        filter_lower_fallback.as_deref(),
                    )
                {
                    let content: Element<'emoji, Message> =
                        if let Some(handle) = self.media_cache.get(&sticker.url) {
                            cosmic::widget::image(handle.clone())
                                .width(cosmic::iced::Length::Fixed(60.0))
                                .height(cosmic::iced::Length::Fixed(60.0))
                                .into()
                        } else {
                            container(
                                Column::new()
                                    .spacing(2)
                                    .align_x(Alignment::Center)
                                    .push(Named::new("image-x-generic-symbolic").size(24))
                                    .push(body(&sticker.shortcode).size(10)),
                            )
                            .width(60)
                            .height(60)
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center)
                            .class(cosmic::theme::Container::Card)
                            .into()
                        };

                    let btn = button::custom(content)
                        .padding(2)
                        .class(cosmic::theme::Button::Text)
                        .on_press(Message::SendSticker {
                            body: sticker.body.clone(),
                            url: sticker.url.clone(),
                            width: sticker.width,
                            height: sticker.height,
                        });
                    let btn_tooltip = tooltip_button_at(btn, &sticker.body, Position::Top);
                    sticker_grid = sticker_grid.push(btn_tooltip);
                    has_stickers = true;
                    count += 1;
                    if count >= 80 {
                        break;
                    }
                }
            }

            let scroll_grid = if !has_stickers {
                let no_found = container(
                    Column::new()
                        .spacing(10)
                        .align_x(Alignment::Center)
                        .push(Named::new("image-x-generic-symbolic").size(48))
                        .push(body(fl!("no-stickers-found")).size(14)),
                )
                .width(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .padding(20);

                scrollable(no_found)
                    .height(200)
                    .width(cosmic::iced::Length::Fill)
            } else {
                scrollable(sticker_grid.wrap())
                    .height(200)
                    .width(cosmic::iced::Length::Fill)
            };

            picker_col = picker_col.push(scroll_grid);
        } else {
            // --- EMOJIS TAB ---
            if self.emoji_search_query.is_empty() {
                let categories = [
                    (emojis::Group::SmileysAndEmotion, "😄"),
                    (emojis::Group::PeopleAndBody, "👋"),
                    (emojis::Group::AnimalsAndNature, "🌲"),
                    (emojis::Group::FoodAndDrink, "🍔"),
                    (emojis::Group::TravelAndPlaces, "✈️"),
                    (emojis::Group::Activities, "⚽"),
                    (emojis::Group::Objects, "💡"),
                    (emojis::Group::Symbols, "🔣"),
                    (emojis::Group::Flags, "🏁"),
                ];

                let mut cat_row = Row::new().spacing(4).align_y(Alignment::Center);

                if !self.active_custom_emojis.is_empty() {
                    let is_custom_selected = self.selected_emoji_group.is_none();
                    if is_custom_selected {
                        let custom_btn =
                            button::suggested("✨").on_press(Message::SelectEmojiGroup(None));
                        cat_row = cat_row.push(tooltip_button_at(
                            custom_btn,
                            fl!("custom-emojis"),
                            Position::Bottom,
                        ));
                    } else {
                        let btn_content = container(body("✨").size(16))
                            .padding([2, 4])
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center);
                        let custom_btn =
                            button::custom(btn_content).on_press(Message::SelectEmojiGroup(None));
                        cat_row = cat_row.push(tooltip_button_at(
                            custom_btn,
                            fl!("custom-emojis"),
                            Position::Bottom,
                        ));
                    }
                }

                for (group, symbol) in categories {
                    let is_selected = self.selected_emoji_group == Some(group);

                    if is_selected {
                        let btn = button::suggested(symbol)
                            .on_press(Message::SelectEmojiGroup(Some(group)));
                        cat_row = cat_row.push(btn);
                    } else {
                        let btn_content = container(body(symbol).size(16))
                            .padding([2, 4])
                            .align_x(Alignment::Center)
                            .align_y(Alignment::Center);
                        let btn = button::custom(btn_content)
                            .on_press(Message::SelectEmojiGroup(Some(group)));
                        cat_row = cat_row.push(btn);
                    }
                }
                picker_col = picker_col.push(cat_row);
            }

            let mut emoji_grid = Row::new().spacing(4);
            let mut has_elements = false;
            let mut no_results = false;

            if self.emoji_search_query.is_empty() {
                if let Some(group) = self.selected_emoji_group {
                    for emoji in group.emojis() {
                        let emoji_str = emoji.as_str();
                        let btn = button::custom(
                            container(body(emoji.as_str()).size(18))
                                .padding(4)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::EmojiPickerSelected(emoji_str));
                        emoji_grid = emoji_grid.push(btn);
                        has_elements = true;
                    }
                } else if !self.active_custom_emojis.is_empty() {
                    // Custom emojis category view
                    for custom in &self.active_custom_emojis {
                        let on_press_msg = if let Some(ref id) = item_id {
                            Message::ToggleReaction(id.clone(), format!(":{}:", custom.shortcode))
                        } else {
                            Message::InsertEmoji(format!(":{}:", custom.shortcode))
                        };

                        let content: Element<'emoji, Message> =
                            if let Some(handle) = self.media_cache.get(&custom.url) {
                                cosmic::widget::image(handle.clone())
                                    .width(cosmic::iced::Length::Fixed(22.0))
                                    .height(cosmic::iced::Length::Fixed(22.0))
                                    .into()
                            } else {
                                body(format!(":{}", custom.shortcode)).size(12).into()
                            };

                        let btn = button::custom(
                            container(content)
                                .padding(2)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(on_press_msg);
                        let btn_tooltip = tooltip_button_at(btn, &custom.shortcode, Position::Top);
                        emoji_grid = emoji_grid.push(btn_tooltip);
                        has_elements = true;
                    }
                }
            } else {
                let filter_is_ascii = self.emoji_search_query.is_ascii();
                let filter_lower_fallback =
                    (!filter_is_ascii).then(|| self.emoji_search_query.to_lowercase());
                let mut count = 0;

                // Search custom emojis first
                for custom in &self.active_custom_emojis {
                    if crate::contains_ignore_ascii_case(
                        &custom.shortcode,
                        &self.emoji_search_query,
                        filter_lower_fallback.as_deref(),
                    ) || crate::contains_ignore_ascii_case(
                        &custom.body,
                        &self.emoji_search_query,
                        filter_lower_fallback.as_deref(),
                    ) {
                        let on_press_msg = if let Some(ref id) = item_id {
                            Message::ToggleReaction(id.clone(), format!(":{}:", custom.shortcode))
                        } else {
                            Message::InsertEmoji(format!(":{}:", custom.shortcode))
                        };

                        let content: Element<'emoji, Message> =
                            if let Some(handle) = self.media_cache.get(&custom.url) {
                                cosmic::widget::image(handle.clone())
                                    .width(cosmic::iced::Length::Fixed(22.0))
                                    .height(cosmic::iced::Length::Fixed(22.0))
                                    .into()
                            } else {
                                body(format!(":{}", custom.shortcode)).size(12).into()
                            };

                        let btn = button::custom(
                            container(content)
                                .padding(2)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(on_press_msg);
                        let btn_tooltip = tooltip_button_at(btn, &custom.shortcode, Position::Top);
                        emoji_grid = emoji_grid.push(btn_tooltip);
                        has_elements = true;
                        count += 1;
                    }
                }

                // Search standard Unicode emojis
                for emoji in emojis::iter() {
                    if crate::contains_ignore_ascii_case(
                        emoji.name(),
                        &self.emoji_search_query,
                        filter_lower_fallback.as_deref(),
                    ) || emoji.shortcodes().any(|s| {
                        crate::contains_ignore_ascii_case(
                            s,
                            &self.emoji_search_query,
                            filter_lower_fallback.as_deref(),
                        )
                    }) {
                        let emoji_str = emoji.as_str();
                        let btn = button::custom(
                            container(body(emoji.as_str()).size(18))
                                .padding(4)
                                .align_x(Alignment::Center)
                                .align_y(Alignment::Center),
                        )
                        .on_press(Message::EmojiPickerSelected(emoji_str));
                        emoji_grid = emoji_grid.push(btn);
                        count += 1;
                        has_elements = true;
                        if count >= 100 {
                            break;
                        }
                    }
                }
                if count == 0 {
                    no_results = true;
                }
            }

            let scroll_grid = if no_results {
                let no_found = container(
                    Column::new()
                        .spacing(10)
                        .align_x(Alignment::Center)
                        .push(Named::new("edit-find-symbolic").size(64))
                        .push(body(fl!("no-results-found")).size(16)),
                )
                .width(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .padding(20);

                scrollable(no_found)
                    .height(200)
                    .width(cosmic::iced::Length::Fill)
            } else if has_elements {
                scrollable(emoji_grid.wrap())
                    .height(200)
                    .width(cosmic::iced::Length::Fill)
            } else {
                scrollable(cosmic::widget::space().height(0))
                    .height(200)
                    .width(cosmic::iced::Length::Fill)
            };

            picker_col = picker_col.push(scroll_grid);
        }

        container(picker_col)
            .padding(10)
            .width(cosmic::iced::Length::Fill)
            .max_width(340)
            .into()
    }

    fn view_sender_info<'sender>(
        &'sender self,
        avatar_url: Option<&'sender str>,
        sender_name: &'sender str,
        timestamp: &'sender str,
        is_pinned: bool,
    ) -> Row<'sender, Message, cosmic::Theme> {
        let mut sender_info = Row::new().spacing(5).align_y(Alignment::Center);

        if let Some(mxc_url) = avatar_url {
            if let Some(handle) = self.media_cache.get(mxc_url) {
                sender_info =
                    sender_info.push(cosmic::widget::image(handle.clone()).width(20).height(20));
            } else {
                sender_info = sender_info
                    .push(container(Named::new("avatar-default-symbolic").size(12)).padding(2));
            }
        } else {
            sender_info = sender_info
                .push(container(Named::new("avatar-default-symbolic").size(12)).padding(2));
        }

        sender_info = sender_info.push(body(sender_name).size(10));

        sender_info = sender_info.push(body(timestamp).size(10));

        if is_pinned {
            sender_info = sender_info.push(Named::new("pin-symbolic").size(12));
        }

        sender_info
    }

    fn view_message_image<'image>(
        &'image self,
        image: &'image matrix_sdk::ruma::events::room::message::ImageMessageEventContent,
    ) -> Column<'image, Message, cosmic::Theme> {
        let mut bubble_col = Column::new();
        if self.user_settings.media_previews_display_policy {
            let mxc_url = match &image.source {
                MediaSource::Plain(uri) => uri.as_str(),
                MediaSource::Encrypted(file) => file.url.as_str(),
            };
            if let Some(handle) = self.media_cache.get(mxc_url) {
                bubble_col = bubble_col.push(
                    button::custom(cosmic::widget::image(handle.clone()).width(
                        if self.app_settings.compact_mode {
                            150
                        } else {
                            300
                        },
                    ))
                    .padding(0)
                    .on_press(Message::OpenImage(handle.clone())),
                );
            }
        }
        bubble_col
    }

    fn view_message_file<'a>(
        &'a self,
        file: &'a matrix_sdk::ruma::events::room::message::FileMessageEventContent,
    ) -> Column<'a, Message, cosmic::Theme> {
        let mut bubble_col = Column::new();
        bubble_col = bubble_col.push(body(fl!("file-message", name = file.body.clone())));
        bubble_col
    }

    fn view_message_video<'a>(
        &'a self,
        video: &'a matrix_sdk::ruma::events::room::message::VideoMessageEventContent,
    ) -> Column<'a, Message, cosmic::Theme> {
        let mut bubble_col = Column::new();

        #[cfg(feature = "video-player")]
        if self.user_settings.media_previews_display_policy {
            let mxc_url = match &video.source {
                MediaSource::Plain(uri) => uri.to_string(),
                MediaSource::Encrypted(file) => file.url.to_string(),
            };
            if let Some(entry) = self.video_cache.get(&mxc_url) {
                let width = if self.app_settings.compact_mode {
                    150
                } else {
                    300
                };
                let player = iced_video_player::VideoPlayer::new(&entry.video)
                    .width(cosmic::iced::Length::Fixed(width as f32))
                    .on_error(|e| Message::VideoPlaybackError(e.to_string()));
                let pause_icon = if entry.video.paused() {
                    "media-playback-start-symbolic"
                } else {
                    "media-playback-pause-symbolic"
                };
                bubble_col = bubble_col.push(player);
                bubble_col = bubble_col.push(
                    button::custom(Named::new(pause_icon))
                        .padding(4)
                        .class(cosmic::theme::Button::Icon)
                        .on_press(Message::ToggleVideoPause(mxc_url.clone())),
                );
            } else {
                let play_msg = Message::PlayVideo {
                    source: video.source.clone(),
                    mxc_url,
                    filename: video.body.clone(),
                    autoplay: false,
                };
                let thumbnail_source = video
                    .info
                    .as_ref()
                    .and_then(|info| info.thumbnail_source.clone());
                if let Some(thumb_src) = thumbnail_source
                    && let Some(thumb_url) = source_to_mxc(&thumb_src)
                    && let Some(handle) = self.media_cache.get(&thumb_url)
                {
                    let width = if self.app_settings.compact_mode {
                        150
                    } else {
                        300
                    };
                    let thumb = cosmic::widget::image(handle.clone()).width(width);
                    let play_icon = container(Named::new("media-playback-start-symbolic").size(32))
                        .width(cosmic::iced::Length::Fill)
                        .height(cosmic::iced::Length::Fill)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center);
                    bubble_col = bubble_col.push(
                        button::custom(cosmic::iced::widget::stack![thumb, play_icon])
                            .padding(0)
                            .on_press(play_msg),
                    );
                } else {
                    bubble_col =
                        bubble_col.push(button::text(PLAY_VIDEO.as_str()).on_press(play_msg));
                }
            }
        }
        bubble_col
    }

    fn view_message_audio<'a>(
        &'a self,
        audio: &'a matrix_sdk::ruma::events::room::message::AudioMessageEventContent,
    ) -> Column<'a, Message, cosmic::Theme> {
        let mut bubble_col = Column::new();
        bubble_col = bubble_col.push(body(fl!("audio-message", name = audio.body.clone())));
        bubble_col
    }

    fn view_og_preview<'a>(&'a self, og: &'a crate::utils::og::OgPreview) -> Element<'a, Message> {
        let site_label = og.site_name.as_deref().unwrap_or(&og.domain);
        let header_text = text::caption(site_label);

        let mut text_col = Column::new().spacing(2).width(cosmic::iced::Length::Fill);

        if let Some(title) = &og.title {
            text_col = text_col.push(text::body(title).font(cosmic::iced::Font {
                weight: cosmic::iced::font::Weight::Bold,
                ..Default::default()
            }));
        }

        if let Some(desc) = &og.description {
            let desc_trimmed = if desc.chars().count() > 180 {
                let truncated: String = desc.chars().take(180).collect();
                format!("{truncated}…")
            } else {
                desc.clone()
            };
            text_col = text_col.push(text::caption(desc_trimmed));
        }

        let main_content: Element<'a, Message> = if let Some(image_handle) = &og.image {
            let img = cosmic::widget::image(image_handle.clone())
                .width(80)
                .height(80)
                .content_fit(cosmic::iced::ContentFit::Cover);

            Row::new()
                .spacing(12)
                .align_y(Alignment::Center)
                .width(cosmic::iced::Length::Fill)
                .push(img)
                .push(text_col)
                .into()
        } else {
            text_col.into()
        };

        let card_col = Column::new()
            .spacing(6)
            .width(cosmic::iced::Length::Fill)
            .push(header_text)
            .push(main_content);

        let card_container = container(card_col)
            .padding(10)
            .width(cosmic::iced::Length::Fill)
            .style(|theme: &cosmic::Theme| {
                use cosmic::iced::widget::container::Catalog;
                let cosmic = theme.cosmic();
                let mut style = theme.style(&cosmic::theme::Container::Card);
                style.border.radius = cosmic.corner_radii.radius_xs.into();
                style
            });

        button::custom(card_container)
            .on_press(Message::OpenMatrixLink(og.url.clone()))
            .padding(0)
            .width(cosmic::iced::Length::Fill)
            .class(cosmic::theme::Button::ListItem(
                self.core.system_theme().cosmic().corner_radii.radius_m,
            ))
            .into()
    }

    fn view_message_text<'message>(
        &'message self,
        events: &'message [PreviewEvent],
        links: &'message [(String, String)],
    ) -> Column<'message, Message, Theme> {
        let mut bubble_col: Column<'message, Message, Theme> =
            Column::new().spacing(6).width(cosmic::iced::Length::Fill);

        let has_custom_emoji = events
            .iter()
            .any(|e| matches!(e, PreviewEvent::CustomEmoji { .. }));

        if !has_custom_emoji {
            let text = crate::rich_text::events_to_string(events);
            bubble_col = bubble_col.push(cosmic::widget::selectable_text::body(text));
        } else {
            let is_emoji_only = events.iter().all(|e| match e {
                PreviewEvent::CustomEmoji { .. }
                | PreviewEvent::Break
                | PreviewEvent::EndBlock
                | PreviewEvent::StartHeading => true,
                PreviewEvent::Text(t) => t.trim().is_empty(),
                _ => false,
            }) && events
                .iter()
                .any(|e| matches!(e, PreviewEvent::CustomEmoji { .. }));

            let emoji_size = if is_emoji_only { 40.0 } else { 20.0 };

            let mut wrap_row = Row::new().spacing(4).align_y(Alignment::Center);

            for event in events {
                match event {
                    PreviewEvent::CustomEmoji { url, alt } => {
                        if let Some(handle) = self.media_cache.get(url) {
                            let img = cosmic::widget::image(handle.clone())
                                .width(cosmic::iced::Length::Fixed(emoji_size))
                                .height(cosmic::iced::Length::Fixed(emoji_size));
                            let btn = button::custom(img)
                                .padding(0)
                                .class(cosmic::theme::Button::Text)
                                .on_press(Message::OpenImage(handle.clone()));
                            wrap_row = wrap_row.push(tooltip_button_at(btn, alt, Position::Top));
                        } else {
                            wrap_row =
                                wrap_row.push(body(alt).size(if is_emoji_only { 28 } else { 14 }));
                        }
                    }
                    PreviewEvent::Text(s) | PreviewEvent::Code(s) => {
                        wrap_row = wrap_row.push(body(s.clone()).size(if is_emoji_only {
                            28
                        } else {
                            14
                        }));
                    }
                    _ => {}
                }
            }

            bubble_col = bubble_col.push(wrap_row.wrap());
        }
        if !links.is_empty() {
            let mut seen_urls = std::collections::HashSet::new();
            let mut link_buttons = Row::new().spacing(8);
            let mut has_fallback_links = false;

            for (label, url) in links {
                if seen_urls.insert(url.as_str()) {
                    if self.user_settings.media_previews_display_policy
                        && let Some(crate::utils::og::OgState::Loaded(og)) = self.og_cache.get(url)
                    {
                        bubble_col = bubble_col.push(self.view_og_preview(og));
                    } else {
                        link_buttons = link_buttons.push(
                            cosmic::widget::button::link(label.clone())
                                .on_press(Message::OpenMatrixLink(url.clone())),
                        );
                        has_fallback_links = true;
                    }
                }
            }
            if has_fallback_links {
                bubble_col = bubble_col.push(link_buttons);
            }
        }

        bubble_col
    }

    pub fn view_threaded_timeline(&self) -> Element<'_, Message> {
        let mut timeline_col = Column::new().spacing(10).width(cosmic::iced::Length::Fill);

        let mut pending_date_divider: Option<matrix_sdk::ruma::MilliSecondsSinceUnixEpoch> = None;

        for item in &self.threaded_timeline_items {
            if item.item.is_none() {
                let element = self.view_item(
                    item,
                    &self.thread_counts,
                    &self.event_id_to_index,
                    &self.thread_root_to_last_index,
                );
                timeline_col =
                    timeline_col.push(tagged_row(element, item, scroll::THREAD_ROW_PREFIX));
                continue;
            }

            if let Some(timeline_item) = &item.item
                && let Some(event) = timeline_item.as_event()
                && event.content().as_message().is_some()
            {
                if let Some(date) = pending_date_divider.take() {
                    timeline_col = timeline_col.push(
                        container(
                            Row::new()
                                .push(divider::horizontal::default())
                                .push(body(
                                    DateTime::from_timestamp_secs(date.as_secs().into())
                                        .unwrap_or_default()
                                        .duration_trunc(TimeDelta::try_days(1).unwrap_or_default())
                                        .unwrap_or_default()
                                        .to_rfc2822()
                                        .trim_end_matches(" 00:00:00 +0000")
                                        .to_owned(),
                                ))
                                .push(divider::horizontal::default())
                                .align_y(Alignment::Center),
                        )
                        .id(scroll::row_id(
                            scroll::THREAD_ROW_PREFIX,
                            &format!("d:{}", date.as_secs()),
                        )),
                    );
                }

                let element = self.view_item(
                    item,
                    &self.thread_counts,
                    &self.event_id_to_index,
                    &self.thread_root_to_last_index,
                );
                timeline_col =
                    timeline_col.push(tagged_row(element, item, scroll::THREAD_ROW_PREFIX));
            } else if let Some(timeline_item) = &item.item
                && let Some(matrix::VirtualTimelineItem::DateDivider(date)) =
                    timeline_item.as_virtual()
            {
                pending_date_divider = Some(*date);
            }
        }

        let scrollable_timeline = scrollable(timeline_col)
            .id(crate::THREADED_TIMELINE_ID.clone())
            .height(cosmic::iced::Length::Fill)
            .on_scroll(|viewport| Message::TimelineScrolled(viewport, true));

        scrollable_timeline.into()
    }

    pub fn view_preview(&self) -> Element<'_, Message> {
        container(
            scrollable(
                self.view_message_text(&self.composer_preview_events, &self.composer_preview_links),
            )
            .height(100),
        )
        .padding(0)
        .into()
    }

    fn view_item<'item>(
        &'item self,
        item: &'item crate::ConstellationItem,
        thread_counts: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, u32>,
        event_id_to_index: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, usize>,
        thread_root_to_last_index: &std::collections::HashMap<
            matrix_sdk::ruma::OwnedEventId,
            usize,
        >,
    ) -> Element<'item, Message> {
        if let Some(timeline_item) = &item.item
            && let Some(event) = timeline_item.as_event()
            && let Some(message) = event.content().as_message()
        {
            self.view_message_item(
                item,
                event,
                message,
                thread_counts,
                event_id_to_index,
                thread_root_to_last_index,
            )
        } else if let Some(timeline_item) = &item.item
            && let Some(event) = timeline_item.as_event()
            && let Some(sticker) = &item.sticker
        {
            self.view_sticker_item(
                item,
                event,
                sticker,
                thread_counts,
                event_id_to_index,
                thread_root_to_last_index,
            )
        } else {
            self.view_state_item(item)
        }
    }

    fn view_message_reply_preview<'item>(
        &'item self,
        event: &'item matrix_sdk_ui::timeline::EventTimelineItem,
        is_me: bool,
    ) -> Option<Element<'item, Message>> {
        let in_reply_to = event.content().in_reply_to()?;
        let mut reply_sender = "";
        let mut reply_body = "";

        if let TimelineDetails::Ready(replied_ev) = &in_reply_to.event {
            reply_sender = replied_ev.sender.as_str();
            if let Some(msg) = replied_ev.content.as_message() {
                reply_body = msg.body();
            }
        }

        let mut reply_snippet = String::with_capacity(64);
        if !reply_sender.is_empty() {
            reply_snippet.push_str(reply_sender);
            reply_snippet.push_str(": ");
        }

        if reply_body.len() <= 50 {
            if !reply_body.is_empty() {
                reply_snippet.push_str(reply_body);
            } else {
                reply_snippet.push_str(&fl!("replying"));
            }
        } else {
            let mut char_indices = reply_body.char_indices();
            if let Some((idx_47, _)) = char_indices.nth(47) {
                if char_indices.nth(2).is_some() {
                    // 50th char
                    reply_snippet.push_str(&reply_body[..idx_47]);
                    reply_snippet.push_str("...");
                } else {
                    reply_snippet.push_str(reply_body);
                }
            } else {
                reply_snippet.push_str(reply_body);
            }
        }

        let reply_event_id = in_reply_to.event_id.clone();
        let reply_indicator = Row::new()
            .spacing(5)
            .push(body("⤴").size(10))
            .push(body(reply_snippet).size(10));

        let reply_btn = button::custom(reply_indicator)
            .on_press(Message::JumpToMessage(reply_event_id))
            .class(cosmic::theme::Button::ListItem(
                self.core.system_theme().cosmic().corner_radii.radius_m,
            ));

        let reply_indicator_wrap = container(reply_btn)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            })
            .padding([0, 0, 5, 10]);

        Some(reply_indicator_wrap.into())
    }

    #[allow(clippy::too_many_arguments)]
    fn view_message_action_row<'item>(
        &'item self,
        item: &'item crate::ConstellationItem,
        event: &'item matrix_sdk_ui::timeline::EventTimelineItem,
        message: &'item matrix_sdk_ui::timeline::Message,
        item_id: &TimelineEventItemId,
        is_ignored: bool,
        thread_counts: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, u32>,
        event_id_to_index: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, usize>,
        thread_root_to_last_index: &std::collections::HashMap<
            matrix_sdk::ruma::OwnedEventId,
            usize,
        >,
    ) -> Element<'item, Message> {
        let is_me = item.is_me;
        let mut action_row = Row::new().spacing(5).align_y(Alignment::Center);

        // "Add reaction" button
        let is_picker_open = self.active_reaction_picker.as_ref() == Some(item_id);
        let btn = icon(Named::new("face-smile-symbolic")).on_press(if is_picker_open {
            Message::OpenReactionPicker(None)
        } else {
            Message::OpenReactionPicker(Some(item_id.clone()))
        });
        let btn_tooltip = tooltip_button_at(btn, ADD_REACTION.as_str(), Position::Bottom);
        action_row = action_row.push(btn_tooltip);

        // Reply button
        let reply_btn = icon(Named::new("mail-replied-symbolic"))
            .on_press(Message::StartReply(item_id.clone()));
        let reply_tooltip = tooltip_button_at(reply_btn, TOOLTIP_REPLY.as_str(), Position::Bottom);
        action_row = action_row.push(reply_tooltip);

        // Download button for media attachments
        let download_msg = match message.msgtype() {
            MessageType::Image(image) => Some((
                DOWNLOAD_IMAGE.as_str(),
                Message::SaveMedia {
                    source: image.source.clone(),
                    filename: image.body.clone(),
                },
            )),
            MessageType::File(file) => Some((
                DOWNLOAD_FILE.as_str(),
                Message::SaveMedia {
                    source: file.source.clone(),
                    filename: file.body.clone(),
                },
            )),
            MessageType::Video(video) => Some((
                DOWNLOAD_VIDEO.as_str(),
                Message::SaveMedia {
                    source: video.source.clone(),
                    filename: video.body.clone(),
                },
            )),
            MessageType::Audio(audio) => Some((
                DOWNLOAD_AUDIO.as_str(),
                Message::SaveMedia {
                    source: audio.source.clone(),
                    filename: audio.body.clone(),
                },
            )),
            _ => None,
        };
        if let Some((tooltip, msg)) = download_msg {
            let download_btn = icon(Named::new("document-save-symbolic")).on_press(msg);
            let download_tooltip = tooltip_button_at(download_btn, tooltip, Position::Bottom);
            action_row = action_row.push(download_tooltip);
        }

        // Thread button (either summary or start thread button)
        let has_thread_root = item.thread_root_id.is_some();
        let mut num_replies = event
            .content()
            .thread_summary()
            .map(|s| s.num_replies)
            .unwrap_or_default();

        if let Some(event_id) = event.event_id() {
            let manual_count = thread_counts.get(event_id).copied().unwrap_or(0);
            if manual_count > num_replies {
                num_replies = manual_count;
            }
        }

        let has_thread_summary =
            num_replies > 0 && !has_thread_root && self.active_thread_root.is_none();

        if has_thread_summary {
            action_row = action_row.push(self.view_thread_summary(
                item,
                event,
                thread_counts,
                event_id_to_index,
                thread_root_to_last_index,
            ));
        } else if self.active_thread_root.is_none() && !has_thread_root {
            let root_id = item_id.clone();
            let start_thread_btn = icon(Named::new("view-list-symbolic")).on_press(match root_id {
                TimelineEventItemId::EventId(id) => Message::OpenThread(id.to_owned()),
                _ => Message::NoOp,
            });
            let action_tooltip =
                tooltip_button_at(start_thread_btn, TOOLTIP_THREAD.as_str(), Position::Bottom);
            action_row = action_row.push(action_tooltip);
        }

        if matches!(item_id, TimelineEventItemId::EventId(_)) {
            let copy_btn = icon(Named::new("edit-copy-symbolic"))
                .on_press(Message::CopyMessageLink(item_id.clone()));
            let copy_tooltip =
                tooltip_button_at(copy_btn, TOOLTIP_COPY_LINK.as_str(), Position::Bottom);
            action_row = action_row.push(copy_tooltip);
        }

        if is_me {
            let edit_btn =
                icon(Named::new("edit-symbolic")).on_press(Message::StartEdit(item_id.clone()));
            let edit_tooltip = tooltip_button_at(edit_btn, TOOLTIP_EDIT.as_str(), Position::Bottom);
            action_row = action_row.push(edit_tooltip);

            let delete_btn = button::custom(Named::new("user-trash-symbolic"))
                .class(cosmic::theme::Button::Destructive)
                .on_press(Message::RedactMessage(item_id.clone()));
            let delete_tooltip =
                tooltip_button_at(delete_btn, TOOLTIP_DELETE.as_str(), Position::Bottom);
            action_row = action_row.push(delete_tooltip);
        } else {
            if is_ignored {
                let ignore_btn = tooltip_button(
                    icon(Named::new("dialog-error-symbolic")).on_press(Message::UserSettings(
                        crate::settings::user::Message::UnignoreUserById(item.sender_id.to_owned()),
                    )),
                    UNIGNORE_USER.as_str(),
                );
                action_row = action_row.push(ignore_btn);
            } else {
                let ignore_btn = tooltip_button(
                    icon(Named::new("dialog-error-symbolic")).on_press(Message::UserSettings(
                        crate::settings::user::Message::IgnoreUserById(item.sender_id.to_owned()),
                    )),
                    IGNORE.as_str(),
                );
                action_row = action_row.push(ignore_btn);
            }
        }

        action_row.into()
    }

    fn view_message_item<'item>(
        &'item self,
        item: &'item crate::ConstellationItem,
        event: &'item matrix_sdk_ui::timeline::EventTimelineItem,
        message: &'item matrix_sdk_ui::timeline::Message,
        thread_counts: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, u32>,
        event_id_to_index: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, usize>,
        thread_root_to_last_index: &std::collections::HashMap<
            matrix_sdk::ruma::OwnedEventId,
            usize,
        >,
    ) -> Element<'item, Message> {
        let is_me = item.is_me;

        let fallback_id;
        let item_id = if let Some(id) = item.item_id.as_ref() {
            id
        } else {
            fallback_id = event.identifier();
            &fallback_id
        };
        let reaction_row = self.view_reactions(event, item_id);
        let is_ignored = self.user_settings.ignored_users.contains(&item.sender_id);
        let is_pinned = if let Some(TimelineEventItemId::EventId(id)) = &item.item_id {
            self.pinned_events.contains(id)
        } else {
            false
        };
        let sender_info = self.view_sender_info(
            item.avatar_url.as_deref(),
            item.sender_name.as_str(),
            item.timestamp.as_str(),
            is_pinned,
        );

        let sender_info_wrap = container(sender_info)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });

        let mut bubble_col = Column::new()
            .spacing(if self.app_settings.compact_mode { 0 } else { 2 })
            .push(sender_info_wrap);

        if let Some(reply_wrap) = self.view_message_reply_preview(event, is_me) {
            bubble_col = bubble_col.push(reply_wrap);
        }

        match message.msgtype() {
            MessageType::Image(image) => {
                bubble_col = bubble_col.push(self.view_message_image(image));
            }
            MessageType::File(file) => {
                bubble_col = bubble_col.push(self.view_message_file(file));
            }
            MessageType::Video(video) => {
                bubble_col = bubble_col.push(self.view_message_video(video));
            }
            MessageType::Audio(audio) => {
                bubble_col = bubble_col.push(self.view_message_audio(audio));
            }
            _ => {
                let (events, links) = if self.app_settings.render_markdown {
                    (&item.markdown, &item.markdown_links)
                } else {
                    (&item.plain_text, &item.plain_links)
                };
                bubble_col = bubble_col.push(self.view_message_text(events, links));
            }
        }

        let action_row = self.view_message_action_row(
            item,
            event,
            message,
            item_id,
            is_ignored,
            thread_counts,
            event_id_to_index,
            thread_root_to_last_index,
        );

        let reaction_row_wrap = container(reaction_row)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });
        bubble_col = bubble_col.push(reaction_row_wrap);

        if self.active_reaction_picker.as_ref() == Some(item_id) {
            bubble_col = bubble_col.push(self.view_emoji_picker(Some(item_id.clone())));
        }

        let action_row_wrap = container(action_row)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });
        bubble_col = bubble_col.push(action_row_wrap);

        let bubble = container(bubble_col)
            .style(move |theme: &cosmic::Theme| {
                use cosmic::iced::widget::container::Catalog;
                let cosmic = theme.cosmic();
                let mut style = theme.style(&cosmic::theme::Container::Card);
                if is_me {
                    style.border.color = cosmic.accent.base.into();
                    style.border.width = 1.0;
                }
                style
            })
            .padding(if self.app_settings.compact_mode {
                5
            } else {
                10
            })
            .max_width(600);

        let bubble_wrap = container(bubble)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });

        bubble_wrap.into()
    }

    #[allow(clippy::too_many_arguments)]
    fn view_sticker_action_row<'item>(
        &'item self,
        item: &'item crate::ConstellationItem,
        event: &'item matrix_sdk_ui::timeline::EventTimelineItem,
        sticker: &'item crate::utils::item::StickerItem,
        item_id: &TimelineEventItemId,
        is_ignored: bool,
        thread_counts: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, u32>,
        event_id_to_index: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, usize>,
        thread_root_to_last_index: &std::collections::HashMap<
            matrix_sdk::ruma::OwnedEventId,
            usize,
        >,
    ) -> Element<'item, Message> {
        let is_me = item.is_me;
        let mut action_row = Row::new().spacing(5).align_y(Alignment::Center);

        // "Add reaction" button
        let is_picker_open = self.active_reaction_picker.as_ref() == Some(item_id);
        let btn = icon(Named::new("face-smile-symbolic")).on_press(if is_picker_open {
            Message::OpenReactionPicker(None)
        } else {
            Message::OpenReactionPicker(Some(item_id.clone()))
        });
        let btn_tooltip = tooltip_button_at(btn, ADD_REACTION.as_str(), Position::Bottom);
        action_row = action_row.push(btn_tooltip);

        // Reply button
        let reply_btn = icon(Named::new("mail-replied-symbolic"))
            .on_press(Message::StartReply(item_id.clone()));
        let reply_tooltip = tooltip_button_at(reply_btn, TOOLTIP_REPLY.as_str(), Position::Bottom);
        action_row = action_row.push(reply_tooltip);

        // Download button
        let download_filename = if sticker.body.ends_with(".png")
            || sticker.body.ends_with(".webp")
            || sticker.body.ends_with(".gif")
        {
            sticker.body.clone()
        } else {
            format!("{}.png", sticker.body)
        };
        let download_btn =
            icon(Named::new("document-save-symbolic")).on_press(Message::SaveMedia {
                source: sticker.source.clone(),
                filename: download_filename,
            });
        let download_tooltip =
            tooltip_button_at(download_btn, DOWNLOAD_IMAGE.as_str(), Position::Bottom);
        action_row = action_row.push(download_tooltip);

        // Thread button
        let has_thread_root = item.thread_root_id.is_some();
        let mut num_replies = event
            .content()
            .thread_summary()
            .map(|s| s.num_replies)
            .unwrap_or_default();

        if let Some(event_id) = event.event_id() {
            let manual_count = thread_counts.get(event_id).copied().unwrap_or(0);
            if manual_count > num_replies {
                num_replies = manual_count;
            }
        }

        let has_thread_summary =
            num_replies > 0 && !has_thread_root && self.active_thread_root.is_none();

        if has_thread_summary {
            action_row = action_row.push(self.view_thread_summary(
                item,
                event,
                thread_counts,
                event_id_to_index,
                thread_root_to_last_index,
            ));
        } else if self.active_thread_root.is_none() && !has_thread_root {
            let root_id = item_id.clone();
            let start_thread_btn = icon(Named::new("view-list-symbolic")).on_press(match root_id {
                TimelineEventItemId::EventId(id) => Message::OpenThread(id.to_owned()),
                _ => Message::NoOp,
            });
            let action_tooltip =
                tooltip_button_at(start_thread_btn, TOOLTIP_THREAD.as_str(), Position::Bottom);
            action_row = action_row.push(action_tooltip);
        }

        if matches!(item_id, TimelineEventItemId::EventId(_)) {
            let copy_btn = icon(Named::new("edit-copy-symbolic"))
                .on_press(Message::CopyMessageLink(item_id.clone()));
            let copy_tooltip =
                tooltip_button_at(copy_btn, TOOLTIP_COPY_LINK.as_str(), Position::Bottom);
            action_row = action_row.push(copy_tooltip);
        }

        if is_me {
            let delete_btn = button::custom(Named::new("user-trash-symbolic"))
                .class(cosmic::theme::Button::Destructive)
                .on_press(Message::RedactMessage(item_id.clone()));
            let delete_tooltip =
                tooltip_button_at(delete_btn, TOOLTIP_DELETE.as_str(), Position::Bottom);
            action_row = action_row.push(delete_tooltip);
        } else {
            if is_ignored {
                let ignore_btn = tooltip_button(
                    icon(Named::new("dialog-error-symbolic")).on_press(Message::UserSettings(
                        crate::settings::user::Message::UnignoreUserById(item.sender_id.to_owned()),
                    )),
                    UNIGNORE_USER.as_str(),
                );
                action_row = action_row.push(ignore_btn);
            } else {
                let ignore_btn = tooltip_button(
                    icon(Named::new("dialog-error-symbolic")).on_press(Message::UserSettings(
                        crate::settings::user::Message::IgnoreUserById(item.sender_id.to_owned()),
                    )),
                    IGNORE.as_str(),
                );
                action_row = action_row.push(ignore_btn);
            }
        }

        action_row.into()
    }

    #[allow(clippy::too_many_arguments)]
    fn view_sticker_item<'item>(
        &'item self,
        item: &'item crate::ConstellationItem,
        event: &'item matrix_sdk_ui::timeline::EventTimelineItem,
        sticker: &'item crate::utils::item::StickerItem,
        thread_counts: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, u32>,
        event_id_to_index: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, usize>,
        thread_root_to_last_index: &std::collections::HashMap<
            matrix_sdk::ruma::OwnedEventId,
            usize,
        >,
    ) -> Element<'item, Message> {
        let is_me = item.is_me;

        let fallback_id;
        let item_id = if let Some(id) = item.item_id.as_ref() {
            id
        } else {
            fallback_id = event.identifier();
            &fallback_id
        };
        let reaction_row = self.view_reactions(event, item_id);
        let is_ignored = self.user_settings.ignored_users.contains(&item.sender_id);
        let is_pinned = if let Some(TimelineEventItemId::EventId(id)) = &item.item_id {
            self.pinned_events.contains(id)
        } else {
            false
        };
        let sender_info = self.view_sender_info(
            item.avatar_url.as_deref(),
            item.sender_name.as_str(),
            item.timestamp.as_str(),
            is_pinned,
        );

        let sender_info_wrap = container(sender_info)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });

        let mut sticker_col = Column::new()
            .spacing(if self.app_settings.compact_mode { 0 } else { 2 })
            .push(sender_info_wrap);

        if let Some(reply_wrap) = self.view_message_reply_preview(event, is_me) {
            sticker_col = sticker_col.push(reply_wrap);
        }

        // Sticker image presentation: max 200x200px, borderless, with tooltip
        if self.user_settings.media_previews_display_policy {
            let sticker_element: Element<'item, Message> =
                if let Some(handle) = self.media_cache.get(&sticker.url) {
                    let (display_w, display_h) = match (sticker.width, sticker.height) {
                        (Some(w), Some(h)) if w > 0 && h > 0 => {
                            let max_dim = 200.0f32;
                            let wf = w as f32;
                            let hf = h as f32;
                            let scale = (max_dim / wf).min(max_dim / hf).min(1.0);
                            ((wf * scale) as u16, (hf * scale) as u16)
                        }
                        _ => (200, 200),
                    };
                    let img_widget = cosmic::widget::image(handle.clone())
                        .width(display_w)
                        .height(display_h);
                    let img_btn = button::custom(img_widget)
                        .padding(0)
                        .class(cosmic::theme::Button::Text)
                        .on_press(Message::OpenImage(handle.clone()));
                    tooltip_button_at(img_btn, &sticker.body, Position::Top)
                } else {
                    let placeholder = container(
                        Column::new()
                            .spacing(4)
                            .align_x(Alignment::Center)
                            .push(Named::new("image-x-generic-symbolic").size(32))
                            .push(body(&sticker.body).size(12)),
                    )
                    .width(120)
                    .height(120)
                    .align_x(Alignment::Center)
                    .align_y(Alignment::Center)
                    .class(cosmic::theme::Container::Card);
                    placeholder.into()
                };

            let sticker_align = container(sticker_element)
                .width(cosmic::iced::Length::Fill)
                .align_x(if is_me {
                    Alignment::End
                } else {
                    Alignment::Start
                });
            sticker_col = sticker_col.push(sticker_align);
        }

        let reaction_row_wrap = container(reaction_row)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });
        sticker_col = sticker_col.push(reaction_row_wrap);

        if self.active_reaction_picker.as_ref() == Some(item_id) {
            sticker_col = sticker_col.push(self.view_emoji_picker(Some(item_id.clone())));
        }

        let action_row = self.view_sticker_action_row(
            item,
            event,
            sticker,
            item_id,
            is_ignored,
            thread_counts,
            event_id_to_index,
            thread_root_to_last_index,
        );

        let action_row_wrap = container(action_row)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });
        sticker_col = sticker_col.push(action_row_wrap);

        // Borderless presentation (no Card background or border)
        let bubble = container(sticker_col)
            .padding(if self.app_settings.compact_mode { 4 } else { 8 })
            .max_width(600);

        let bubble_wrap = container(bubble)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });

        bubble_wrap.into()
    }

    fn view_state_item<'item>(
        &'item self,
        item: &'item crate::ConstellationItem,
    ) -> Element<'item, Message> {
        let is_me = item.is_me;
        let is_pinned = if let Some(TimelineEventItemId::EventId(id)) = &item.item_id {
            self.pinned_events.contains(id)
        } else {
            false
        };

        let sender_info = self.view_sender_info(
            item.avatar_url.as_deref(),
            item.sender_name.as_str(),
            item.timestamp.as_str(),
            is_pinned,
        );

        let sender_info_wrap = container(sender_info)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });

        let mut bubble_col = Column::new()
            .spacing(if self.app_settings.compact_mode { 0 } else { 2 })
            .push(sender_info_wrap);

        let (events, links) = if self.app_settings.render_markdown {
            (&item.markdown, &item.markdown_links)
        } else {
            (&item.plain_text, &item.plain_links)
        };
        bubble_col = bubble_col.push(self.view_message_text(events, links));

        let bubble = container(bubble_col)
            .style(move |theme: &cosmic::Theme| {
                use cosmic::iced::widget::container::Catalog;
                let cosmic = theme.cosmic();
                let mut style = theme.style(&cosmic::theme::Container::Card);
                if is_me {
                    style.border.color = cosmic.accent.base.into();
                    style.border.width = 1.0;
                }
                style
            })
            .padding(if self.app_settings.compact_mode {
                5
            } else {
                10
            })
            .max_width(600);

        let bubble_wrap = container(bubble)
            .width(cosmic::iced::Length::Fill)
            .align_x(if is_me {
                Alignment::End
            } else {
                Alignment::Start
            });

        bubble_wrap.into()
    }

    fn resolve_thread_latest_sender_and_body(
        &self,
        event: &matrix_sdk_ui::timeline::EventTimelineItem,
        event_id_to_index: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, usize>,
        thread_root_to_last_index: &std::collections::HashMap<
            matrix_sdk::ruma::OwnedEventId,
            usize,
        >,
    ) -> (Option<String>, Option<String>) {
        let mut latest_sender = None;
        let mut latest_body = None;

        if let Some(summary) = event.content().thread_summary()
            && let TimelineDetails::Ready(latest_ev) = &summary.latest_event
        {
            let mut found_item = None;
            if let TimelineEventItemId::EventId(eid) = &latest_ev.identifier
                && let Some(&idx) = event_id_to_index.get(eid)
            {
                found_item = self.timeline_items.get(idx);
            }

            if let Some(item) = found_item {
                latest_sender = Some(item.sender_name.clone());
                if let Some(timeline_item) = &item.item
                    && let Some(ev) = timeline_item.as_event()
                    && let Some(msg) = ev.content().as_message()
                {
                    latest_body = Some(msg.body().to_owned());
                }
            } else {
                latest_sender = Some(latest_ev.sender.to_string());
                if let Some(msg) = latest_ev.content.as_message() {
                    latest_body = Some(msg.body().to_owned());
                }
            }
        }

        if latest_body.is_none()
            && let Some(event_id) = event.event_id()
            && let Some(&idx) = thread_root_to_last_index.get(event_id)
            && let Some(item) = self.timeline_items.get(idx)
        {
            latest_sender = Some(item.sender_name.clone());
            if let Some(timeline_item) = &item.item
                && let Some(ev) = timeline_item.as_event()
                && let Some(msg) = ev.content().as_message()
            {
                latest_body = Some(msg.body().to_owned());
            }
        }

        (latest_sender, latest_body)
    }

    fn view_thread_summary(
        &'chat self,
        item: &crate::ConstellationItem,
        event: &matrix_sdk_ui::timeline::EventTimelineItem,
        thread_counts: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, u32>,
        event_id_to_index: &std::collections::HashMap<matrix_sdk::ruma::OwnedEventId, usize>,
        thread_root_to_last_index: &std::collections::HashMap<
            matrix_sdk::ruma::OwnedEventId,
            usize,
        >,
    ) -> cosmic::widget::Container<'chat, Message, cosmic::Theme> {
        let has_thread_root = item.thread_root_id.is_some();

        let mut num_replies = event
            .content()
            .thread_summary()
            .map(|s| s.num_replies)
            .unwrap_or_default();

        // Manual count if summary is missing or incomplete (due to view-side filtering)
        if let Some(event_id) = event.event_id() {
            let manual_count = thread_counts.get(event_id).copied().unwrap_or(0);
            if manual_count > num_replies {
                num_replies = manual_count;
            }
        }

        if num_replies > 0 && !has_thread_root && self.active_thread_root.is_none() {
            let (latest_sender, latest_body) = self.resolve_thread_latest_sender_and_body(
                event,
                event_id_to_index,
                thread_root_to_last_index,
            );

            let mut summary_row = Row::new()
                .spacing(5)
                .align_y(Alignment::Center)
                .push(
                    body(format!(
                        "{} {}",
                        num_replies,
                        if num_replies == 1 {
                            REPLY.as_str()
                        } else {
                            REPLIES.as_str()
                        }
                    ))
                    .size(12),
                )
                .push(Named::new("chat-bubble-symbolic").size(14));

            if let Some(final_body) = &latest_body
                && !final_body.is_empty()
            {
                let unknown_sender = fl!("unknown-sender");
                let sender = latest_sender.as_deref().unwrap_or(unknown_sender.as_str());
                let mut text_str = String::with_capacity(64);
                text_str.push_str(sender);
                text_str.push_str(": ");

                if final_body.len() <= 30 {
                    text_str.push_str(final_body);
                } else {
                    let mut char_indices = final_body.char_indices();
                    if let Some((idx_27, _)) = char_indices.nth(27) {
                        if char_indices.nth(2).is_some() {
                            // 30th char
                            text_str.push_str(&final_body[..idx_27]);
                            text_str.push_str("...");
                        } else {
                            text_str.push_str(final_body);
                        }
                    } else {
                        text_str.push_str(final_body);
                    }
                }
                summary_row = summary_row.push(body(text_str).size(12));
            }

            let summary_btn = button::custom(container(summary_row).padding([0, 5])).on_press({
                let fallback_id;
                let id_to_use = if let Some(id) = item.item_id.as_ref() {
                    id
                } else {
                    fallback_id = event.identifier();
                    &fallback_id
                };
                match id_to_use {
                    TimelineEventItemId::EventId(id) => Message::OpenThread(id.to_owned()),
                    _ => Message::NoOp,
                }
            });

            container(summary_btn).padding([5, 0])
        } else {
            container(body(""))
        }
    }

    #[rust_analyzer::skip]
    pub fn view_main_content(&self) -> Element<'_, Message> {
        let mut content = Column::new()
            .spacing(20)
            .padding(20)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill);

        if let Some(active_tab) = self.active_tab()
            && active_tab.is_search()
        {
            content = content.push(self.view_tabbed_header_opt(self.selected_room.as_ref()));
            let mut search_area = Column::new()
                .spacing(10)
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill);
            search_area = search_area.push(self.view_search_results());
            content = content.push(search_area);
            return content.into();
        }

        if let Some(room_id) = &self.selected_room {
            let selected_room_data = self
                .selected_room
                .as_ref()
                .and_then(|id| self.room_by_id(id));

            let is_video_room = selected_room_data
                .map(|r| {
                    r.room_type
                        .as_ref()
                        .is_some_and(|t| t.as_str() == "org.matrix.msc3401.call.room")
                })
                .unwrap_or(false);

            content = content.push(self.view_tabbed_header(room_id));
            if self.inviting_to_room {
                content = content.push(self.view_invite_ui());
            }

            let mut chat_area = Column::new()
                .spacing(10)
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill);
            // When viewing an event-focused (permalink context) timeline,
            // show a persistent banner offering to return to live.
            if self.active_event_focus.is_some() {
                chat_area = chat_area.push(self.view_older_messages_banner());
            }
            if self.active_thread_root.is_some() {
                chat_area = chat_area.push(self.view_threaded_timeline());
            } else {
                chat_area = chat_area.push(self.view_timeline());
            }
            if !is_video_room {
                chat_area = chat_area.push(self.view_composer());
            }
            content = content.push(chat_area);
        } else {
            content = content.push(self.view_empty_state());
        }

        content.into()
    }

    fn view_empty_state(&self) -> Element<'_, Message> {
        let column = Column::new()
            .spacing(10)
            .align_x(Alignment::Center)
            .push(Named::new("chat-bubble-symbolic").size(64))
            .push(text::title1(fl!("no-room-selected")))
            .push(body(fl!("select-room-to-start")));

        container(column)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }

    fn view_invite_ui(&self) -> Element<'_, Message> {
        let mut invite_input = text_input("@user:example.com", &self.invite_to_room_id)
            .on_input(Message::InviteToRoomIdChanged);
        let is_empty = self.invite_to_room_id.trim().is_empty();
        if !is_empty {
            invite_input = invite_input.on_submit(|_| Message::InviteToRoom);
        }
        let invite_btn_widget: Element<'_, Message> = disabled_or_tooltip(
            button::text(fl!("invite")),
            !is_empty,
            Message::InviteToRoom,
            fl!("enter-user-id-to-invite"),
        );
        let invite_ui = Column::new().spacing(5).push(invite_input).push(
            Row::new()
                .spacing(5)
                .push(invite_btn_widget)
                .push(button::text(fl!("cancel")).on_press(Message::ToggleInviteToRoom)),
        );
        container(invite_ui).padding(5).into()
    }

    pub fn view_tabbed_header<'a>(&'a self, room_id: &std::sync::Arc<str>) -> Element<'a, Message> {
        self.view_tabbed_header_opt(Some(room_id))
    }

    pub fn view_tabbed_header_opt<'a>(
        &'a self,
        room_id: Option<&std::sync::Arc<str>>,
    ) -> Element<'a, Message> {
        let context_menus = self.view_tab_context_menus(&self.tab_model);

        let tabs = tab_bar::horizontal(&self.tab_model)
            .show_close_icon_on_hover(true)
            .on_activate(Message::TabActivated)
            .on_close(Message::TabClosed)
            .context_menu(context_menus)
            .width(cosmic::iced::Length::Shrink);

        let scrollable_tabs =
            cosmic::widget::scrollable::horizontal(tabs).width(cosmic::iced::Length::Fill);

        let mut header = Row::new()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(scrollable_tabs);

        if let Some(room_id) = room_id {
            let is_in_call = self.user_id.as_ref().is_some_and(|uid| {
                self.call_participants
                    .get(room_id)
                    .is_some_and(|p| p.iter().any(|participant| participant.as_str() == uid))
            });

            let call_participants = self.call_participants.get(room_id);
            let participant_count = call_participants.map_or(0, |p| p.len());

            if participant_count > 0 {
                header = header.push(
                    container(
                        Row::new()
                            .spacing(5)
                            .align_y(Alignment::Center)
                            .push(Named::new("camera-video-symbolic").size(16))
                            .push(body(format!("{participant_count}")).size(12)),
                    )
                    .padding([2, 5]),
                );
            }

            if is_in_call {
                header = header.push(tooltip_button_at(
                    button::custom(Named::new("call-stop"))
                        .class(cosmic::theme::Button::Destructive)
                        .on_press(Message::LeaveCall),
                    fl!("call-leave"),
                    Position::Bottom,
                ));
            }

            header = header.push(self.view_room_actions_menu(room_id));
        }

        header.into()
    }

    pub fn view_room_header<'a>(&'a self, room_id: &std::sync::Arc<str>) -> Element<'a, Message> {
        self.view_tabbed_header(room_id)
    }

    fn view_tab_context_menus(
        &self,
        model: &cosmic::widget::segmented_button::SingleSelectModel,
    ) -> Option<Vec<menu::Tree<Message>>> {
        let key_binds = std::collections::HashMap::new();
        let mut children = Vec::new();

        for entity in model.iter() {
            let entity_tab = model.data::<Tab>(entity).cloned();

            match entity_tab {
                Some(Tab::Room(rid)) => {
                    children.push(self.room_menu_items(&rid));
                }
                Some(Tab::Thread { .. }) | Some(Tab::Search { .. }) => {
                    let mut items = Vec::new();
                    items.push(menu::Item::Button(
                        fl!("close-tab"),
                        Some(cosmic::widget::icon::Handle::from(Named::new(
                            "window-close-symbolic",
                        ))),
                        MenuAct::CloseRoom,
                    ));
                    children.push(items);
                }
                None => {
                    children.push(Vec::new());
                }
            }
        }

        Some(menu::nav_context(&key_binds, children))
    }

    fn room_menu_items(&self, room_id: &std::sync::Arc<str>) -> Vec<menu::Item<MenuAct, String>> {
        let is_in_call = self.user_id.as_ref().is_some_and(|uid| {
            self.call_participants
                .get(room_id)
                .is_some_and(|p| p.iter().any(|participant| participant.as_str() == uid))
        });

        let mut items = Vec::new();

        if is_in_call {
            items.push(menu::Item::Button(
                fl!("call-leave"),
                Some(cosmic::widget::icon::Handle::from(Named::new("call-stop"))),
                MenuAct::LeaveCall,
            ));
        } else {
            items.push(menu::Item::Button(
                fl!("call-join"),
                Some(cosmic::widget::icon::Handle::from(Named::new("camera-web"))),
                MenuAct::JoinCall,
            ));
        }

        let pinned_count = self.pinned_events.len();
        let pinned_label = if pinned_count > 0 && self.selected_room.as_ref() == Some(room_id) {
            format!("{} ({pinned_count})", fl!("pinned-messages"))
        } else {
            fl!("pinned-messages")
        };

        items.push(menu::Item::Button(
            pinned_label,
            Some(cosmic::widget::icon::Handle::from(Named::new(
                "pin-symbolic",
            ))),
            MenuAct::TogglePinnedPanel,
        ));

        let active_threads_count = self.active_threads.len();
        let active_threads_label =
            if active_threads_count > 0 && self.selected_room.as_ref() == Some(room_id) {
                format!("{} ({active_threads_count})", fl!("active-threads"))
            } else {
                fl!("active-threads")
            };

        items.push(menu::Item::Button(
            active_threads_label,
            Some(cosmic::widget::icon::Handle::from(Named::new(
                "chat-symbolic",
            ))),
            MenuAct::ToggleActiveThreadsPanel,
        ));

        items.push(menu::Item::Button(
            fl!("room-members"),
            Some(cosmic::widget::icon::Handle::from(Named::new(
                "system-users-symbolic",
            ))),
            MenuAct::ToggleMembersPanel,
        ));

        items.push(menu::Item::Button(
            TOOLTIP_COPY_ROOM_LINK.as_str().to_string(),
            Some(cosmic::widget::icon::Handle::from(Named::new(
                "link-symbolic",
            ))),
            MenuAct::CopyRoomLink,
        ));

        items.push(menu::Item::Button(
            fl!("room-settings"),
            Some(cosmic::widget::icon::Handle::from(Named::new(
                "emblem-system",
            ))),
            MenuAct::RoomSettings,
        ));

        items.push(menu::Item::Button(
            fl!("manage-members"),
            Some(cosmic::widget::icon::Handle::from(Named::new(
                "avatar-default-symbolic",
            ))),
            MenuAct::ManageRoomMembers,
        ));

        items.push(menu::Item::Button(
            fl!("invite"),
            Some(cosmic::widget::icon::Handle::from(Named::new(
                "contact-new-symbolic",
            ))),
            MenuAct::RoomInvite,
        ));

        items.push(menu::Item::Button(
            fl!("close-tab"),
            Some(cosmic::widget::icon::Handle::from(Named::new(
                "window-close-symbolic",
            ))),
            MenuAct::CloseRoom,
        ));

        items
    }

    fn view_room_actions_menu(&self, room_id: &std::sync::Arc<str>) -> Element<'_, Message> {
        let key_binds = std::collections::HashMap::new();
        let menu_btn = button::icon(Named::new("view-more-symbolic"));
        let menu_tooltip = tooltip_button_at(menu_btn, fl!("room-actions"), Position::Bottom);
        let items = self.room_menu_items(room_id);
        let menu_tree = menu::Tree::with_children(
            RcElementWrapper::new(menu_tooltip),
            menu::items(&key_binds, items),
        );
        menu::bar(vec![menu_tree])
            .item_height(menu::ItemHeight::Dynamic(40))
            .item_width(menu::ItemWidth::Uniform(200))
            .spacing(4.0)
            .into()
    }

    pub fn view_composer(&self) -> Element<'_, Message> {
        let mut content = Column::new().spacing(10);

        if self.is_composer_emoji_picker_active {
            content = content.push(self.view_emoji_picker(None));
        }

        if let Some(replying_to) = &self.replying_to {
            let body = replying_to.body_text();
            let snippet = truncate_snippet(body);
            let reply_bar = view_reply_bar(snippet, replying_to);
            content = content.push(container(reply_bar).padding(10));
        }

        if let Some(editing_item) = &self.editing_item {
            let body = editing_item.body_text();
            content = content.push(Self::view_editing_bar(body));
        }

        let composer = self.view_composer_editor();
        let attachments_view = self.view_composer_attachments();
        let controls = self.view_composer_controls();

        let composer_card = cosmic::widget::dnd_destination(
            container(Column::new().spacing(5).push(composer).push(controls))
                .style(|theme: &cosmic::Theme| {
                    use cosmic::iced::widget::container::Catalog;
                    theme.style(&cosmic::theme::Container::Card)
                })
                .padding(10),
            vec![std::borrow::Cow::Borrowed("text/uri-list")],
        )
        .on_file_transfer(Message::DndFileTransfer)
        .on_data_received(Message::DndDataReceived);

        content.push(attachments_view).push(composer_card).into()
    }

    fn view_editing_bar(body_text: String) -> Element<'static, Message> {
        let snippet = truncate_snippet(body_text);

        let edit_bar = Row::new()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(body(fl!("editing")).size(12))
            .push(body(snippet).size(12))
            .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
            .push(tooltip_button_at(
                icon(Named::new("window-close-symbolic")).on_press(Message::CancelEdit),
                fl!("cancel"),
                Position::Bottom,
            ));

        container(edit_bar).padding(10).into()
    }

    fn view_composer_editor(&self) -> Element<'_, Message> {
        if self.composer_is_preview {
            self.view_preview()
        } else {
            container(
                text_editor(&self.composer_content)
                    .placeholder(fl!("type-message"))
                    .on_action(Message::ComposerAction)
                    .key_binding(|keypress| match keypress.key.as_ref() {
                        cosmic::iced::keyboard::Key::Named(
                            cosmic::iced::keyboard::key::Named::Enter,
                        ) => {
                            if keypress.modifiers.shift() {
                                Some(cosmic::widget::text_editor::Binding::Enter)
                            } else {
                                Some(cosmic::widget::text_editor::Binding::Custom(
                                    Message::SendMessage,
                                ))
                            }
                        }
                        _ => cosmic::widget::text_editor::Binding::from_key_press(keypress),
                    })
                    .height(80),
            )
            .padding(0)
            .into()
        }
    }

    fn view_composer_attachments(&self) -> Element<'_, Message> {
        let mut attachments_view = Column::new().spacing(5);
        if !self.composer_attachments.is_empty() {
            attachments_view = attachments_view.push(body(fl!("attachments")).size(12));
            for (i, path) in self.composer_attachments.iter().enumerate() {
                let filename = path.file_name().unwrap_or_default().to_string_lossy();
                let attachment_row = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(body(filename).size(12))
                    .push(
                        button::destructive(fl!("remove-attachment"))
                            .on_press(Message::RemoveAttachment(i)),
                    );
                attachments_view = attachments_view.push(attachment_row);
            }
        }
        attachments_view.into()
    }

    fn view_composer_controls(&self) -> Element<'_, Message> {
        let is_empty =
            self.composer_content.text().trim().is_empty() && self.composer_attachments.is_empty();

        let send_btn = icon(if self.editing_item.is_some() {
            Named::new("mail-send-symbolic")
        } else if self.active_thread_root.is_some() {
            Named::new("mail-reply-all-symbolic")
        } else {
            Named::new("mail-send-symbolic")
        });

        let send_btn_widget: Element<'_, Message> = if is_empty {
            tooltip_button(send_btn, fl!("type-message-or-attach"))
        } else {
            tooltip_button_at(
                send_btn.on_press(Message::SendMessage),
                fl!("tooltip-send"),
                Position::Top,
            )
        };

        let attach = icon(Named::new("mail-attachment-symbolic")).on_press(Message::AddAttachment);
        let emoji = icon(Named::new("face-smile-symbolic")).on_press(Message::ToggleEmojiPicker);
        let location = icon(Named::new("mark-location-symbolic")).on_press(Message::ShareLocation);
        Row::new()
            .spacing(10)
            .push(tooltip_button(attach, TOOLTIP_ATTACH.as_str()))
            .push(tooltip_button(emoji, TOOLTIP_EMOJIS.as_str()))
            .push(tooltip_button(location, TOOLTIP_LOCATION.as_str()))
            .push(if self.composer_is_preview {
                let content = icon(Named::new("edit-symbolic")).on_press(Message::TogglePreview);
                tooltip_button(content, TOOLTIP_EDIT.as_str())
            } else {
                let find = icon(Named::new("edit-find-symbolic")).on_press(Message::TogglePreview);
                tooltip_button(find, TOOLTIP_FIND.as_str())
            })
            .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
            .push(send_btn_widget)
            .into()
    }

    pub fn view_search_results<'a>(&'a self) -> Element<'a, Message> {
        let mut results_col = Column::new().spacing(15).width(cosmic::iced::Length::Fill);

        if self.selected_room.is_none() {
            results_col = results_col.push(self.view_search_global_section());
        }

        if self.selected_room.is_some() {
            results_col = results_col.push(self.view_search_messages_section());
        }
        results_col = results_col.push(divider::horizontal::default());
        results_col = results_col.push(self.view_search_public_rooms_section());

        scrollable(results_col)
            .id(crate::SEARCH_RESULTS_ID.clone())
            .height(cosmic::iced::Length::Fill)
            .into()
    }

    fn view_search_global_section<'a>(&'a self) -> Element<'a, Message> {
        use crate::matrix::GlobalSearchScope;

        let mut section = Column::new().spacing(15).width(cosmic::iced::Length::Fill);
        section = section.push(text::title3(fl!("search-messages-global")).size(14));

        // 3-way scope toggle: All / DMs / Groups. The active option uses
        // `suggested` (the codebase's existing toggle idiom — see the join-rule
        // control in space settings); inactive ones switch the scope (which
        // re-fires the query).
        let make_scope_btn = |label: String, this_scope: GlobalSearchScope| {
            if self.global_search_scope == this_scope {
                button::suggested(label)
            } else {
                button::text(label).on_press(Message::SetGlobalSearchScope(this_scope))
            }
        };
        section = section.push(
            Row::new()
                .spacing(6)
                .push(make_scope_btn(
                    fl!("search-scope-all"),
                    GlobalSearchScope::All,
                ))
                .push(make_scope_btn(
                    fl!("search-scope-dms"),
                    GlobalSearchScope::DmsOnly,
                ))
                .push(make_scope_btn(
                    fl!("search-scope-groups"),
                    GlobalSearchScope::GroupsOnly,
                )),
        );

        if self.is_searching_global_messages {
            section = section.push(
                container(cosmic::widget::progress_bar::indeterminate_circular().size(24.0))
                    .width(cosmic::iced::Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(20),
            );
        } else if !self.global_message_search_results.is_empty() {
            let mut message_list = Column::new().spacing(10).width(cosmic::iced::Length::Fill);
            for result in &self.global_message_search_results {
                message_list =
                    message_list.push(self.view_message_search_result_card(result, true));
            }
            section = section.push(message_list);
        } else {
            section = section.push(
                container(
                    Column::new()
                        .spacing(10)
                        .align_x(Alignment::Center)
                        .push(Named::new("edit-find-symbolic").size(32))
                        .push(body(fl!("search-no-global-matches")).size(14)),
                )
                .width(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .padding(20),
            );
        }
        section.into()
    }

    fn view_message_search_result_card<'a>(
        &'a self,
        result: &'a crate::matrix::MessageSearchResult,
        is_global: bool,
    ) -> Element<'a, Message> {
        let room_id_arc: std::sync::Arc<str> = std::sync::Arc::from(result.room_id.as_str());
        let event_id = result.event_id.clone();

        let mut card_content = Column::new().spacing(5);

        if is_global {
            // Room of origin — the headline difference from the in-room
            // card. Fall back to the raw room id if no display name.
            card_content = card_content.push(
                body(
                    result
                        .room_name
                        .as_deref()
                        .unwrap_or(result.room_id.as_str()),
                )
                .font(cosmic::iced::Font {
                    weight: cosmic::iced::font::Weight::Bold,
                    ..Default::default()
                })
                .size(11),
            );
        }

        card_content = card_content.push(
            Row::new()
                .spacing(8)
                .align_y(Alignment::Center)
                .push(
                    body(result.sender_id.as_str())
                        .font(cosmic::iced::Font {
                            weight: cosmic::iced::font::Weight::Bold,
                            ..Default::default()
                        })
                        .size(13),
                )
                .push(body(result.timestamp.as_str()).size(10)),
        );
        card_content = card_content.push(self.view_message_text(&result.plain_text, &result.links));

        let event_id_for_jump = event_id.clone();

        let jump_action = if is_global {
            let room_id_for_jump = room_id_arc.clone();
            Message::OpenRoomEvent {
                room_id: room_id_for_jump,
                event_id: event_id_for_jump,
            }
        } else {
            Message::JumpToMessageOrLoadContext(event_id_for_jump)
        };

        card_content = card_content.push(
            Row::new()
                .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
                .push(button::text(fl!("jump-to-message")).on_press(jump_action)),
        );

        container(card_content)
            .style(|theme: &cosmic::Theme| {
                use cosmic::iced::widget::container::Catalog;
                let cosmic = theme.cosmic();
                let mut style = theme.style(&cosmic::theme::Container::Card);
                style.border.color = cosmic.accent.base.into();
                style.border.width = 1.0;
                style
            })
            .padding(10)
            .width(cosmic::iced::Length::Fill)
            .into()
    }

    fn view_search_messages_section<'a>(&'a self) -> Element<'a, Message> {
        let mut section = Column::new().spacing(15).width(cosmic::iced::Length::Fill);
        section = section.push(text::title3(fl!("search-messages-in-room")).size(14));

        if self.is_searching_messages {
            section = section.push(
                container(cosmic::widget::progress_bar::indeterminate_circular().size(24.0))
                    .width(cosmic::iced::Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(20),
            );
        } else if !self.message_search_results.is_empty() {
            let mut message_list = Column::new().spacing(10).width(cosmic::iced::Length::Fill);
            for result in &self.message_search_results {
                message_list =
                    message_list.push(self.view_message_search_result_card(result, false));
            }
            if self.search_has_more {
                let load_more_widget = if self.is_searching_more_messages {
                    container(cosmic::widget::progress_bar::indeterminate_circular().size(20.0))
                        .width(cosmic::iced::Length::Fill)
                        .align_x(Alignment::Center)
                        .padding(10)
                } else {
                    container(
                        button::text(fl!("load-more")).on_press(Message::LoadMoreMessageSearch),
                    )
                    .width(cosmic::iced::Length::Fill)
                    .align_x(Alignment::Center)
                    .padding(10)
                };
                message_list = message_list.push(load_more_widget);
            }
            section = section.push(message_list);
        } else {
            section = section.push(
                container(
                    Column::new()
                        .spacing(10)
                        .align_x(Alignment::Center)
                        .push(Named::new("edit-find-symbolic").size(32))
                        .push(body(fl!("search-no-room-matches")).size(14)),
                )
                .width(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .padding(20),
            );
        }
        section.into()
    }

    fn view_search_public_rooms_section<'a>(&'a self) -> Element<'a, Message> {
        let mut section = Column::new().spacing(15).width(cosmic::iced::Length::Fill);
        section = section.push(text::title3(fl!("public-rooms-spaces")).size(14));

        section = section.push(if self.is_searching_public {
            container(cosmic::widget::progress_bar::indeterminate_circular().size(24.0))
                .width(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .padding(20)
        } else if self.public_search_results.is_empty() {
            container(body(fl!("no-public-rooms")).size(14))
                .width(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .padding(20)
        } else {
            let mut public_list = Column::new().spacing(10).width(cosmic::iced::Length::Fill);
            for room in &self.public_search_results {
                let name = room
                    .name
                    .as_deref()
                    .or(room.canonical_alias.as_deref())
                    .map(str::to_string)
                    .unwrap_or_else(|| fl!("unnamed-room"));
                let is_joined = self.joined_room_ids.contains(room.id.as_str());

                // Avatar
                let default_avatar = || {
                    let initials: String = name.chars().next().unwrap_or('R').to_string();
                    container(body(initials).size(14))
                        .width(40)
                        .height(40)
                        .align_x(Alignment::Center)
                        .align_y(Alignment::Center)
                        .style(|theme: &cosmic::Theme| {
                            use cosmic::iced::widget::container::Catalog;
                            let cosmic = theme.cosmic();
                            let mut style = theme.style(&cosmic::theme::Container::Card);
                            style.border.radius = cosmic.corner_radii.radius_xs.into();
                            style
                        })
                };
                let avatar_widget: Element<'_, Message> = if let Some(url) = &room.avatar_url {
                    if let Some(handle) = self.media_cache.get(url) {
                        cosmic::widget::image(handle.clone())
                            .width(40)
                            .height(40)
                            .border_radius(4.0)
                            .into()
                    } else {
                        default_avatar().into()
                    }
                } else {
                    default_avatar().into()
                };

                let mut details_col = Column::new().spacing(4).push(
                    Row::new()
                        .spacing(8)
                        .align_y(Alignment::Center)
                        .push(body(name.to_string()).font(cosmic::iced::Font {
                            weight: cosmic::iced::font::Weight::Bold,
                            ..Default::default()
                        }))
                        .push(
                            body(fl!("room-member-count", count = room.num_joined_members))
                                .size(10),
                        ),
                );
                if let Some(topic) = &room.topic {
                    details_col = details_col.push(body(topic.to_string()).size(11));
                }
                details_col = details_col.push(body(room.id.to_string()).size(9));

                let action_btn = if is_joined {
                    button::text(fl!("joined-button"))
                } else {
                    let id_arc = std::sync::Arc::from(room.id.as_str());
                    button::suggested(fl!("join")).on_press(Message::JoinRoom(id_arc))
                };

                let card_row = Row::new()
                    .spacing(15)
                    .align_y(Alignment::Center)
                    .push(avatar_widget)
                    .push(details_col.width(cosmic::iced::Length::Fill))
                    .push(action_btn);

                public_list = public_list.push(
                    container(card_row)
                        .style(|theme: &cosmic::Theme| {
                            use cosmic::iced::widget::container::Catalog;
                            theme.style(&cosmic::theme::Container::Card)
                        })
                        .padding(10)
                        .width(cosmic::iced::Length::Fill),
                );
            }
            container(public_list)
        });
        section.into()
    }

    pub fn view_video_room<'a>(
        &'a self,
        room_data: &'a crate::matrix::RoomData,
    ) -> Element<'a, Message> {
        let room_id = room_data.id.as_ref();
        let is_in_call = self.user_id.as_ref().is_some_and(|uid| {
            self.call_participants
                .get(room_id)
                .is_some_and(|p| p.iter().any(|participant| participant.as_str() == uid))
        });

        let mut content = Column::new()
            .spacing(20)
            .align_x(Alignment::Center)
            .width(cosmic::iced::Length::Fill);

        // Icon
        content = content.push(Named::new("camera-video-symbolic").size(96));

        // Room Name
        let room_name = room_data
            .name
            .as_deref()
            .map(str::to_string)
            .unwrap_or_else(|| fl!("unnamed-video-room"));
        content = content.push(text::title1(room_name).size(24));

        // Call status
        if is_in_call {
            content = content.push(body(fl!("call-status-connected")).size(16));
            let leave_btn = button::destructive(fl!("call-leave")).on_press(Message::LeaveCall);
            content = content.push(leave_btn);
        } else {
            content = content.push(body(fl!("call-status-not-connected")).size(16));
            let join_btn = button::suggested(fl!("call-join")).on_press(Message::JoinCall);
            content = content.push(join_btn);
        }

        // Participants list
        let call_participants = self.call_participants.get(room_id);
        let participant_count = call_participants.map_or(0, |p| p.len());

        let mut participants_col = Column::new().spacing(10).align_x(Alignment::Center);
        participants_col = participants_col
            .push(text::title3(fl!("participants", count = participant_count)).size(16));

        if let Some(participants) = call_participants
            && !participants.is_empty()
        {
            for participant in participants {
                participants_col = participants_col.push(body(participant.as_str()).size(14));
            }
        } else {
            participants_col = participants_col.push(body(fl!("no-participants")).size(14));
        }

        content = content.push(
            container(participants_col)
                .style(|theme: &cosmic::Theme| {
                    use cosmic::iced::widget::container::Catalog;
                    theme.style(&cosmic::theme::Container::Card)
                })
                .padding(15)
                .width(300.0),
        );

        container(content)
            .width(cosmic::iced::Length::Fill)
            .height(cosmic::iced::Length::Fill)
            .align_x(Alignment::Center)
            .align_y(Alignment::Center)
            .into()
    }

    pub fn view_members_panel(&self) -> Element<'_, Message> {
        if self.is_loading_members {
            container(cosmic::widget::progress_bar::indeterminate_circular().size(24.0))
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        } else {
            let mut member_list = Column::new().spacing(5);
            for member in &self.room_members {
                let avatar = if let Some(avatar_url) = &member.avatar_url
                    && let Some(handle) = self.media_cache.get(avatar_url)
                {
                    Element::from(cosmic::widget::image(handle.clone()).width(24).height(24))
                } else {
                    container(Named::new("avatar-default-symbolic").size(16))
                        .padding(4)
                        .into()
                };

                let name_text = member.display_name.as_deref().unwrap_or(&member.user_id);
                let mut member_info_col = Column::new().spacing(2);
                member_info_col = member_info_col.push(body(name_text).size(14));
                if member.display_name.is_some() {
                    member_info_col = member_info_col.push(body(&member.user_id).size(10));
                }

                let member_row = Row::new()
                    .spacing(10)
                    .align_y(Alignment::Center)
                    .push(avatar)
                    .push(member_info_col);

                let member_container = container(member_row)
                    .padding(8)
                    .width(cosmic::iced::Length::Fill);

                member_list = member_list.push(member_container);
            }

            scrollable(member_list)
                .height(cosmic::iced::Length::Fill)
                .into()
        }
    }

    pub fn view_pinned_panel(&self) -> Element<'_, Message> {
        if self.is_loading_pinned {
            container(cosmic::widget::progress_bar::indeterminate_circular().size(24.0))
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        } else {
            let mut pinned_list = Column::new().spacing(10);
            let mut found_any = false;

            for item in &self.pinned_events_details {
                found_any = true;
                pinned_list = pinned_list.push(self.view_pinned_item(item));
            }

            if !found_any {
                container(
                    Column::new()
                        .spacing(10)
                        .align_x(Alignment::Center)
                        .push(Named::new("pin-symbolic").size(48))
                        .push(body(fl!("no-pinned-messages")).size(14)),
                )
                .width(cosmic::iced::Length::Fill)
                .padding(20)
                .align_x(Alignment::Center)
                .into()
            } else {
                scrollable(pinned_list)
                    .height(cosmic::iced::Length::Fill)
                    .into()
            }
        }
    }

    fn view_pinned_item<'a>(&'a self, item: &'a matrix::PinnedEventInfo) -> Element<'a, Message> {
        let avatar = if let Some(avatar_url) = &item.avatar_url
            && let Some(handle) = self.media_cache.get(avatar_url)
        {
            Element::from(cosmic::widget::image(handle.clone()).width(20).height(20))
        } else {
            container(Named::new("avatar-default-symbolic").size(12))
                .padding(2)
                .into()
        };

        let message_row = Row::new()
            .spacing(10)
            .align_y(Alignment::Center)
            .push(avatar)
            .push(
                Column::new()
                    .spacing(2)
                    .push(
                        Row::new()
                            .spacing(5)
                            .align_y(Alignment::Center)
                            .push(text::title3(&item.sender_name).size(12))
                            .push(body(&item.timestamp).size(10)),
                    )
                    .push(body(&item.body).size(12)),
            );

        // Card is a jump-to-message link, with a sibling unpin button
        // (siblings, not nested, to keep iced happy).
        let card_row = match matrix_sdk::ruma::EventId::parse(&item.event_id) {
            Ok(event_id) => {
                let jump_btn = button::custom(
                    container(message_row)
                        .padding(5)
                        .width(cosmic::iced::Length::Fill),
                )
                .width(cosmic::iced::Length::Fill)
                .class(cosmic::theme::Button::ListItem(
                    self.core.system_theme().cosmic().corner_radii.radius_m,
                ))
                .on_press(Message::JumpToMessage(event_id.clone()));

                let unpin_btn = tooltip_button_at(
                    icon(Named::new("pin-symbolic")).on_press(Message::UnpinMessage(event_id)),
                    fl!("unpin-message"),
                    Position::Bottom,
                );

                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(jump_btn)
                    .push(unpin_btn)
            }
            Err(_) => Row::new().push(message_row),
        };

        container(card_row)
            .style(move |theme: &cosmic::Theme| {
                use cosmic::iced::widget::container::Catalog;
                theme.style(&cosmic::theme::Container::Card)
            })
            .width(cosmic::iced::Length::Fill)
            .into()
    }

    pub fn view_active_threads_panel(&self) -> Element<'_, Message> {
        if self.is_loading_active_threads {
            container(cosmic::widget::progress_bar::indeterminate_circular().size(24.0))
                .width(cosmic::iced::Length::Fill)
                .height(cosmic::iced::Length::Fill)
                .align_x(Alignment::Center)
                .align_y(Alignment::Center)
                .into()
        } else {
            let mut threads_list = Column::new().spacing(10);
            let mut found_any = false;

            for item in &self.active_threads {
                found_any = true;
                threads_list = threads_list.push(self.view_active_thread_item(item));
            }

            if !found_any {
                container(
                    Column::new()
                        .spacing(10)
                        .align_x(Alignment::Center)
                        .push(Named::new("chat-symbolic").size(48))
                        .push(body(fl!("no-active-threads")).size(14)),
                )
                .width(cosmic::iced::Length::Fill)
                .padding(20)
                .align_x(Alignment::Center)
                .into()
            } else {
                scrollable(threads_list)
                    .height(cosmic::iced::Length::Fill)
                    .into()
            }
        }
    }

    fn view_active_thread_item<'a>(
        &'a self,
        item: &'a matrix::ActiveThreadInfo,
    ) -> Element<'a, Message> {
        let avatar = if let Some(avatar_url) = &item.avatar_url
            && let Some(handle) = self.media_cache.get(avatar_url)
        {
            Element::from(cosmic::widget::image(handle.clone()).width(20).height(20))
        } else {
            container(Named::new("avatar-default-symbolic").size(12))
                .padding(2)
                .into()
        };

        let num_replies = if let Ok(event_id) = matrix_sdk::ruma::EventId::parse(&item.event_id) {
            item.num_replies
                .max(*self.thread_counts.get(&event_id).unwrap_or(&0))
        } else {
            item.num_replies
        };

        let replies_text = format!(
            "{} {}",
            num_replies,
            if num_replies == 1 {
                REPLY.as_str()
            } else {
                REPLIES.as_str()
            }
        );

        let mut meta_row = Row::new()
            .spacing(5)
            .align_y(Alignment::Center)
            .push(Named::new("chat-symbolic").size(12))
            .push(body(replies_text).size(10));

        if let Some(latest) = &item.latest_activity {
            meta_row = meta_row
                .push(body("•").size(10))
                .push(body(latest).size(10));
        }
        let unread = if let Ok(event_id) = matrix_sdk::ruma::EventId::parse(&item.event_id) {
            self.thread_unreads
                .get(&event_id)
                .copied()
                .unwrap_or_else(|| item.unread_summary())
        } else {
            item.unread_summary()
        };

        if unread.is_unread() {
            let count = unread.display_count();
            let badge_text = if unread.num_unread_mentions > 0 {
                format!("@ {count}")
            } else {
                format!("{count}")
            };
            meta_row = meta_row.push(body("•").size(10)).push(
                container(body(badge_text).size(10)).padding([1, 6]).style(
                    move |theme: &cosmic::Theme| {
                        use cosmic::iced::widget::container::Catalog;
                        let cosmic = theme.cosmic();
                        let mut style = theme.style(&cosmic::theme::Container::Card);
                        style.background =
                            Some(cosmic::iced::Background::Color(cosmic.accent.base.into()));
                        style.text_color = Some(cosmic.accent.on.into());
                        style.border.radius = cosmic.corner_radii.radius_xs.into();
                        style
                    },
                ),
            );
        }

        let message_col = Column::new()
            .spacing(2)
            .push(
                Row::new()
                    .spacing(5)
                    .align_y(Alignment::Center)
                    .push(text::title3(&item.sender_name).size(12))
                    .push(body(&item.timestamp).size(10)),
            )
            .push(body(&item.body).size(12))
            .push(meta_row);

        let message_row = Row::new()
            .spacing(10)
            .align_y(Alignment::Start)
            .push(avatar)
            .push(message_col);

        let card_row = match matrix_sdk::ruma::EventId::parse(&item.event_id) {
            Ok(event_id) => {
                let open_thread_btn = button::custom(
                    container(message_row)
                        .padding(5)
                        .width(cosmic::iced::Length::Fill),
                )
                .width(cosmic::iced::Length::Fill)
                .class(cosmic::theme::Button::ListItem(
                    self.core.system_theme().cosmic().corner_radii.radius_m,
                ))
                .on_press(Message::OpenThread(event_id.clone()));

                let jump_btn = tooltip_button_at(
                    icon(Named::new("go-jump-symbolic")).on_press(Message::JumpToMessage(event_id)),
                    fl!("jump-to-message"),
                    Position::Bottom,
                );

                Row::new()
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .push(open_thread_btn)
                    .push(jump_btn)
            }
            Err(_) => Row::new().push(message_row),
        };

        container(card_row)
            .style(move |theme: &cosmic::Theme| {
                use cosmic::iced::widget::container::Catalog;
                theme.style(&cosmic::theme::Container::Card)
            })
            .width(cosmic::iced::Length::Fill)
            .into()
    }
}

#[rust_analyzer::skip]
fn view_reply_bar<'a>(
    snippet: impl Into<std::borrow::Cow<'a, str>> + 'a,
    replying_to: &'a crate::ConstellationItem,
) -> Row<'a, Message, Theme> {
    Row::new()
        .spacing(10)
        .align_y(Alignment::Center)
        .push(body(fl!("replying-to", user = replying_to.sender_name.as_str())).size(12))
        .push(body(snippet).size(12))
        .push(cosmic::widget::space().width(cosmic::iced::Length::Fill))
        .push(tooltip_button(
            icon(Named::new("window-close-symbolic")).on_press(Message::CancelReply),
            fl!("cancel"),
        ))
}

fn truncate_snippet(body: String) -> std::borrow::Cow<'static, str> {
    let mut char_indices = body.char_indices();
    if let Some((idx_97, _)) = char_indices.nth(97) {
        if char_indices.nth(2).is_some() {
            let mut s = String::with_capacity(100);
            s.push_str(&body[..idx_97]);
            s.push_str("...");
            std::borrow::Cow::Owned(s)
        } else {
            std::borrow::Cow::Owned(body)
        }
    } else {
        std::borrow::Cow::Owned(body)
    }
}

/// Extract the mxc URL from a media source, used for cache lookups.
#[cfg(feature = "video-player")]
fn source_to_mxc(source: &MediaSource) -> Option<String> {
    match source {
        MediaSource::Plain(uri) => Some(uri.to_string()),
        MediaSource::Encrypted(file) => Some(file.url.to_string()),
    }
}
