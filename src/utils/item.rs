use crate::matrix;
use crate::preview::{PreviewEvent, extract_links, parse_markdown, parse_plain_text};
use std::sync::Arc;

#[derive(Clone, Debug)]
pub struct StickerItem {
    pub body: String,
    pub source: matrix_sdk::ruma::events::room::MediaSource,
    pub url: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(Clone, Debug)]
pub struct ConstellationItem {
    pub item: Option<Arc<matrix::TimelineItem>>,
    pub sender_id: matrix_sdk::ruma::OwnedUserId,
    pub sender_name: String,
    pub avatar_url: Option<String>,
    pub timestamp: String,
    pub is_me: bool,
    pub markdown: Vec<PreviewEvent>,
    pub plain_text: Vec<PreviewEvent>,
    pub markdown_links: Vec<(String, String)>,
    pub plain_links: Vec<(String, String)>,
    pub thread_root_id: Option<matrix_sdk::ruma::OwnedEventId>,
    pub item_id: Option<matrix::TimelineEventItemId>,
    pub sticker: Option<StickerItem>,
}
impl ConstellationItem {
    pub fn new(item: Arc<matrix::TimelineItem>, user_id: Option<&str>) -> Self {
        let mut sender_id = matrix_sdk::ruma::user_id!("@unknown:example.com").to_owned();
        let mut sender_name = String::new();
        let mut avatar_url = None;
        let mut timestamp = String::new();
        let mut is_me = false;
        let mut markdown = Vec::new();
        let mut plain_text = Vec::new();
        let mut markdown_links = Vec::new();
        let mut plain_links = Vec::new();
        let mut thread_root_id = None;
        let mut item_id = None;
        let mut sticker = None;
        // ⚡ Bolt Optimization: Pre-compute plain_text representation here
        // to avoid allocating new Strings and Vecs inside the UI render loop (`view_message_text`).

        if let Some(event) = item.as_event() {
            item_id = Some(event.identifier());
            sender_id = event.sender().to_owned();
            if let Some(msg) = event.content().as_message() {
                let is_reply = event.content().in_reply_to().is_some();
                let formatted = match msg.msgtype() {
                    matrix_sdk::ruma::events::room::message::MessageType::Text(t) => {
                        t.formatted.as_ref()
                    }
                    matrix_sdk::ruma::events::room::message::MessageType::Notice(n) => {
                        n.formatted.as_ref()
                    }
                    matrix_sdk::ruma::events::room::message::MessageType::Emote(e) => {
                        e.formatted.as_ref()
                    }
                    _ => None,
                };
                let input_text = if let Some(f) = formatted
                    && f.body.contains("data-mx-emoticon")
                {
                    &f.body
                } else {
                    msg.body()
                };
                markdown = parse_markdown(input_text, is_reply);
                plain_text = parse_plain_text(input_text);
                markdown_links = extract_links(&markdown);
                plain_links = extract_links(&plain_text);
            } else if let Some(stk) = event.content().as_sticker() {
                let content = stk.content();
                let source: matrix_sdk::ruma::events::room::MediaSource =
                    content.source.clone().into();
                let url = match &source {
                    matrix_sdk::ruma::events::room::MediaSource::Plain(uri) => uri.to_string(),
                    matrix_sdk::ruma::events::room::MediaSource::Encrypted(file) => {
                        file.url.to_string()
                    }
                };
                let width = content
                    .info
                    .width
                    .and_then(|w| u32::try_from(u64::from(w)).ok());
                let height = content
                    .info
                    .height
                    .and_then(|h| u32::try_from(u64::from(h)).ok());
                sticker = Some(StickerItem {
                    body: content.body.clone(),
                    source,
                    url,
                    width,
                    height,
                });
            }
            let (name, url) = match event.sender_profile() {
                matrix_sdk_ui::timeline::TimelineDetails::Ready(profile) => (
                    profile
                        .display_name
                        .as_deref()
                        .unwrap_or(event.sender().as_ref())
                        .to_string(),
                    profile.avatar_url.as_ref().map(|uri| uri.to_string()),
                ),
                _ => (event.sender().to_string(), None),
            };
            sender_name = name;
            avatar_url = url;

            let ts_millis = u64::from(event.timestamp().0);
            let datetime =
                chrono::DateTime::from_timestamp_millis(ts_millis as i64).unwrap_or_default();
            timestamp = datetime
                .with_timezone(&chrono::Local)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            is_me = user_id == Some(event.sender().as_str());
            thread_root_id = event.content().thread_root();
        }

        Self {
            item: Some(item),
            sender_id,
            sender_name,
            avatar_url,
            timestamp,
            is_me,
            markdown,
            plain_text,
            markdown_links,
            plain_links,
            thread_root_id,
            item_id,
            sticker,
        }
    }

    pub fn body_text(&self) -> String {
        if let Some(stk) = &self.sticker {
            return stk.body.clone();
        }
        self.item
            .as_ref()
            .and_then(|i| i.as_event())
            .and_then(|ev| ev.content().as_message())
            .map(|msg| msg.body().to_string())
            .unwrap_or_else(|| {
                self.plain_text
                    .iter()
                    .filter_map(|p| {
                        if let PreviewEvent::Text(txt) = p {
                            Some(txt.as_str())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("")
            })
    }

    /// Stable identity used for anchored scrolling (`constellation::scroll`):
    /// remote events by event id, local echoes by transaction id.
    pub fn scroll_key(&self) -> Option<String> {
        self.item_id.as_ref().map(|id| match id {
            matrix::TimelineEventItemId::EventId(event_id) => format!("e:{event_id}"),
            matrix::TimelineEventItemId::TransactionId(txn_id) => format!("t:{txn_id}"),
        })
    }
}

#[cfg(test)]
impl ConstellationItem {
    pub fn mock(sender_name: &str, text: &str, timestamp: &str, is_me: bool) -> Self {
        let sender_id = matrix_sdk::ruma::user_id!("@unknown:example.com").to_owned();
        let markdown = parse_markdown(text, false);
        let plain_text = parse_plain_text(text);
        let markdown_links = extract_links(&markdown);
        let plain_links = extract_links(&plain_text);
        Self {
            item: None,
            sender_id,
            sender_name: sender_name.to_string(),
            avatar_url: None,
            timestamp: timestamp.to_string(),
            is_me,
            markdown,
            plain_text,
            markdown_links,
            plain_links,
            thread_root_id: None,
            item_id: None,
            sticker: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constellation_item_links_precomputation() {
        let item = ConstellationItem::mock(
            "Alice",
            "Check out [Google](https://google.com) and https://rust-lang.org!",
            "2026-07-10 12:00:00",
            false,
        );

        assert_eq!(
            item.markdown_links,
            vec![
                ("Google".to_string(), "https://google.com".to_string()),
                (
                    "https://rust-lang.org".to_string(),
                    "https://rust-lang.org".to_string()
                ),
            ]
        );

        assert_eq!(
            item.plain_links,
            vec![
                ("Google".to_string(), "https://google.com".to_string()),
                (
                    "https://rust-lang.org".to_string(),
                    "https://rust-lang.org".to_string()
                ),
            ]
        );
    }

    #[test]
    fn test_constellation_item_sticker() {
        let mut item = ConstellationItem::mock("Bob", "", "2026-09-17 10:00:00", false);
        let sticker = StickerItem {
            body: "Cat wave".to_string(),
            source: matrix_sdk::ruma::events::room::MediaSource::Plain(
                matrix_sdk::ruma::mxc_uri!("mxc://example.org/sticker").to_owned(),
            ),
            url: "mxc://example.org/sticker".to_string(),
            width: Some(256),
            height: Some(256),
        };
        item.sticker = Some(sticker);
        assert_eq!(item.body_text(), "Cat wave");
    }
}
