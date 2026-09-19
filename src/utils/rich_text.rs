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
            PreviewEvent::CustomEmoji { alt, .. } => {
                buf.push_str(alt);
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

    #[test]
    fn test_events_to_string_with_custom_emoji() {
        let events = vec![
            PreviewEvent::Text("Hello ".to_string()),
            PreviewEvent::CustomEmoji {
                url: "mxc://example.org/cat".to_string(),
                alt: ":cat:".to_string(),
            },
            PreviewEvent::Text(" world!".to_string()),
        ];
        assert_eq!(events_to_string(&events), "Hello :cat: world!");
    }

    #[hegel::test]
    fn prop_events_to_string_invariants(tc: hegel::TestCase) {
        let num_events = tc.draw(
            hegel::generators::integers::<usize>()
                .min_value(0)
                .max_value(20),
        );
        let mut events = Vec::with_capacity(num_events);
        for _ in 0..num_events {
            let variant = tc.draw(
                hegel::generators::integers::<u8>()
                    .min_value(0)
                    .max_value(7),
            );
            let event = match variant {
                0 => PreviewEvent::StartHeading,
                1 => PreviewEvent::EndBlock,
                2 => PreviewEvent::Text(tc.draw(hegel::generators::text().max_size(50))),
                3 => PreviewEvent::Code(tc.draw(hegel::generators::text().max_size(50))),
                4 => PreviewEvent::Break,
                5 => PreviewEvent::StartLink(tc.draw(hegel::generators::text().max_size(50))),
                6 => PreviewEvent::EndLink,
                _ => PreviewEvent::CustomEmoji {
                    url: tc.draw(hegel::generators::text().max_size(30)),
                    alt: tc.draw(hegel::generators::text().max_size(20)),
                },
            };
            events.push(event);
        }

        let result = events_to_string(&events);
        assert!(
            !result.ends_with('\n'),
            "result should not end with newline: {result:?}"
        );
    }
}
