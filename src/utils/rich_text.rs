use crate::PreviewEvent;
use cosmic::Element;
use cosmic::widget::selectable_text::{self, SelectableText};

/// Converts a slice of [`PreviewEvent`]s into a clean string representation.
pub fn events_to_string(events: &[PreviewEvent]) -> String {
    let mut buf = String::new();

    for event in events {
        match event {
            PreviewEvent::StartHeading | PreviewEvent::StartLink(_) | PreviewEvent::EndLink => {}
            PreviewEvent::EndBlock | PreviewEvent::Break => {
                buf.push('\n');
            }
            PreviewEvent::Text(s) | PreviewEvent::Code(s) => {
                buf.push_str(s);
            }
        }
    }

    let trimmed_len = buf.trim_end_matches('\n').len();
    buf.truncate(trimmed_len);
    buf
}

/// Creates a selectable text widget from a slice of [`PreviewEvent`]s using
/// libcosmic's native [`SelectableText`].
pub fn rich_selectable_text<'a>(events: &'a [PreviewEvent]) -> SelectableText<'a> {
    let text = events_to_string(events);
    selectable_text::body(text)
}

pub struct RichSelectableText<'a> {
    content: &'a [PreviewEvent],
}

impl<'a> RichSelectableText<'a> {
    pub fn new<Message>(
        content: &'a [PreviewEvent],
        _on_link_click: impl Fn(String) -> Message + 'a,
    ) -> Self {
        Self { content }
    }

    pub fn into_element<Message: Clone + 'static>(self) -> Element<'a, Message> {
        let text = events_to_string(self.content);
        selectable_text::body(text).into()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_events_to_string_simple() {
        let events = vec![
            PreviewEvent::Text("Hello ".to_string()),
            PreviewEvent::Code("world".to_string()),
        ];
        assert_eq!(events_to_string(&events), "Hello world");
    }

    #[test]
    fn test_events_to_string_with_links_and_breaks() {
        let events = vec![
            PreviewEvent::StartHeading,
            PreviewEvent::Text("Title".to_string()),
            PreviewEvent::EndBlock,
            PreviewEvent::StartLink("https://example.com".to_string()),
            PreviewEvent::Text("https://example.com".to_string()),
            PreviewEvent::EndLink,
            PreviewEvent::Break,
            PreviewEvent::Text("Footer".to_string()),
        ];
        assert_eq!(
            events_to_string(&events),
            "Title\nhttps://example.com\nFooter"
        );
    }
}
