// ⚡ Bolt Optimization:
// We cache the parsed Markdown structure in `PreviewEvent`s to avoid running
// `pulldown_cmark::Parser` on every single render frame inside `view_preview()`.
#[derive(Clone, Debug, PartialEq)]
pub enum PreviewEvent {
    StartHeading,
    EndBlock,
    Text(String),
    Code(String),
    Break,
    StartLink(String),
    EndLink,
    CustomEmoji { url: String, alt: String },
}

/// Parses an HTML `<img>` tag containing the MSC2545 `data-mx-emoticon` attribute into `PreviewEvent::CustomEmoji`.
pub fn parse_custom_emoji_tag(tag: &str) -> Option<PreviewEvent> {
    let lower = tag.to_lowercase();
    if !lower.starts_with("<img") || !lower.contains("data-mx-emoticon") {
        return None;
    }

    let src = extract_html_attr(tag, "src")?;
    let alt = extract_html_attr(tag, "alt")
        .or_else(|| extract_html_attr(tag, "title"))
        .unwrap_or_default();

    Some(PreviewEvent::CustomEmoji { url: src, alt })
}

fn extract_html_attr(tag: &str, attr: &str) -> Option<String> {
    let patterns = [format!("{attr}=\""), format!("{attr}='")];
    for pattern in patterns {
        if let Some(pos) = tag.find(&pattern) {
            let quote = if pattern.ends_with('"') { '"' } else { '\'' };
            let val_start = pos + pattern.len();
            let val_end = tag[val_start..].find(quote)?;
            return Some(tag[val_start..val_start + val_end].to_string());
        }
    }
    None
}

fn split_text_by_urls(text: &str, events: &mut Vec<PreviewEvent>) {
    let mut current_idx = 0;

    while current_idx < text.len() {
        let remaining = &text[current_idx..];
        if let Some(pos) = remaining
            .find("http://")
            .or_else(|| remaining.find("https://"))
        {
            let start_of_url = current_idx + pos;

            if start_of_url > current_idx {
                events.push(PreviewEvent::Text(
                    text[current_idx..start_of_url].to_string(),
                ));
            }

            let mut end_of_url = start_of_url;
            while end_of_url < text.len() {
                let c = text.as_bytes()[end_of_url];
                if c.is_ascii_whitespace() {
                    break;
                }
                end_of_url += 1;
            }

            while end_of_url > start_of_url {
                let last_char = text.as_bytes()[end_of_url - 1];
                if matches!(
                    last_char,
                    b'.' | b',' | b'?' | b'!' | b':' | b';' | b')' | b']' | b'>'
                ) {
                    end_of_url -= 1;
                } else {
                    break;
                }
            }

            let url = &text[start_of_url..end_of_url];
            if !url.is_empty() {
                events.push(PreviewEvent::StartLink(url.to_string()));
                events.push(PreviewEvent::Text(url.to_string()));
                events.push(PreviewEvent::EndLink);
            }

            current_idx = end_of_url;
        } else {
            events.push(PreviewEvent::Text(remaining.to_string()));
            break;
        }
    }
}

fn parse_line_inlines(mut text: &str, events: &mut Vec<PreviewEvent>) {
    while !text.is_empty() {
        let next_special = text.find(['<', '`', '[', '*', '_', '~']);
        let (prefix, rest) = match next_special {
            Some(idx) => (&text[..idx], &text[idx..]),
            None => (text, ""),
        };

        if !prefix.is_empty() {
            split_text_by_urls(prefix, events);
        }

        if rest.is_empty() {
            break;
        }

        // Try custom emoji: <img data-mx-emoticon ... />
        if (rest.starts_with("<img") || rest.starts_with("<IMG"))
            && let Some(close_idx) = rest.find('>')
        {
            let tag = &rest[..=close_idx];
            if let Some(emoji) = parse_custom_emoji_tag(tag) {
                events.push(emoji);
                text = &rest[close_idx + 1..];
                continue;
            }
        }

        // Try inline code: `code`
        if let Some(stripped) = rest.strip_prefix('`')
            && let Some(end_code) = stripped.find('`')
        {
            let code = &stripped[..end_code];
            events.push(PreviewEvent::Code(code.to_string()));
            text = &stripped[end_code + 1..];
            continue;
        }

        // Try markdown link: [label](url)
        if let Some(stripped) = rest.strip_prefix('[')
            && let Some(close_bracket) = stripped.find(']')
        {
            let after_bracket = &stripped[close_bracket + 1..];
            if let Some(stripped_paren) = after_bracket.strip_prefix('(')
                && let Some(close_paren) = stripped_paren.find(')')
            {
                let label = &stripped[..close_bracket];
                let url = &stripped_paren[..close_paren];
                events.push(PreviewEvent::StartLink(url.to_string()));
                if !label.is_empty() {
                    events.push(PreviewEvent::Text(label.to_string()));
                }
                events.push(PreviewEvent::EndLink);
                text = &stripped_paren[close_paren + 1..];
                continue;
            }
        }

        // Try bold: **bold** or __bold__
        if rest.starts_with("**") || rest.starts_with("__") {
            let delim = &rest[..2];
            if let Some(end_idx) = rest[2..].find(delim) {
                let inner = &rest[2..2 + end_idx];
                parse_line_inlines(inner, events);
                text = &rest[2 + end_idx + 2..];
                continue;
            }
        }

        // Try italic: *italic* or _italic_
        if rest.starts_with('*') || rest.starts_with('_') {
            let delim = rest.chars().next().unwrap();
            if let Some(end_idx) = rest[delim.len_utf8()..].find(delim) {
                let inner = &rest[delim.len_utf8()..delim.len_utf8() + end_idx];
                parse_line_inlines(inner, events);
                text = &rest[delim.len_utf8() + end_idx + delim.len_utf8()..];
                continue;
            }
        }

        // Try strikethrough: ~~strike~~
        if let Some(stripped) = rest.strip_prefix("~~")
            && let Some(end_idx) = stripped.find("~~")
        {
            let inner = &stripped[..end_idx];
            parse_line_inlines(inner, events);
            text = &stripped[end_idx + 2..];
            continue;
        }

        // If none matched, consume one char and continue
        let first_char_len = rest.chars().next().map(|c| c.len_utf8()).unwrap_or(1);
        split_text_by_urls(&rest[..first_char_len], events);
        text = &rest[first_char_len..];
    }
}

fn parse_blocks(text: &str, is_markdown: bool, skip_first_blockquote: bool) -> Vec<PreviewEvent> {
    let mut events = Vec::new();
    if text.trim().is_empty() {
        return events;
    }

    // Handle skip_first_blockquote
    let mut skipped_first = false;
    if skip_first_blockquote && text.trim_start().starts_with('>') {
        // Find end of blockquote
        let mut lines = Vec::new();
        let mut past_blockquote = false;
        for line in text.lines() {
            if !past_blockquote {
                if line.trim().is_empty() {
                    past_blockquote = true;
                    skipped_first = true;
                } else if !line.trim_start().starts_with('>') {
                    past_blockquote = true;
                    skipped_first = true;
                    lines.push(line);
                }
            } else {
                lines.push(line);
            }
        }
        if skipped_first {
            let remaining = lines.join("\n");
            return parse_blocks(&remaining, is_markdown, false);
        }
    }

    // Split text into paragraphs separated by double newlines
    let normalized = text.replace("\r\n", "\n").replace('\r', "\n");
    let mut current_block_lines: Vec<&str> = Vec::new();

    let flush_block = |lines: &mut Vec<&str>, events: &mut Vec<PreviewEvent>| {
        if lines.is_empty() {
            return;
        }
        let block_lines = std::mem::take(lines);

        // Check if the lines are part of a heading
        let mut i = 0;
        while i < block_lines.len() {
            let line = block_lines[i];
            let trimmed = line.trim_start();
            let is_heading = trimmed.starts_with('#');
            if is_heading {
                let heading_content = trimmed.trim_start_matches('#').trim();
                if is_markdown {
                    events.push(PreviewEvent::StartHeading);
                }
                parse_line_inlines(heading_content, events);
                events.push(PreviewEvent::EndBlock);
                i += 1;
            } else {
                // Collect lines until next heading or end
                let mut para_lines = Vec::new();
                while i < block_lines.len() {
                    let next_line = block_lines[i];
                    if next_line.trim_start().starts_with('#') {
                        break;
                    }
                    let cleaned = if next_line.trim_start().starts_with('>') {
                        next_line.trim_start().trim_start_matches('>').trim_start()
                    } else {
                        next_line
                    };
                    para_lines.push(cleaned.trim_end_matches([' ', '\t']));
                    i += 1;
                }
                for (idx, pline) in para_lines.iter().enumerate() {
                    parse_line_inlines(pline, events);
                    if idx + 1 < para_lines.len() {
                        events.push(PreviewEvent::Break);
                    }
                }
                events.push(PreviewEvent::EndBlock);
            }
        }
    };

    for line in normalized.lines() {
        if line.trim().is_empty() {
            flush_block(&mut current_block_lines, &mut events);
        } else {
            current_block_lines.push(line);
        }
    }
    flush_block(&mut current_block_lines, &mut events);

    events
}

pub fn parse_markdown(text: &str, skip_first_blockquote: bool) -> Vec<PreviewEvent> {
    parse_blocks(text, true, skip_first_blockquote)
}

pub fn parse_plain_text(text: &str) -> Vec<PreviewEvent> {
    parse_blocks(text, false, false)
}

pub fn extract_links(events: &[PreviewEvent]) -> Vec<(String, String)> {
    let mut links = Vec::new();
    let mut current_link = None;
    let mut current_text = String::new();

    for event in events {
        match event {
            PreviewEvent::StartLink(url) => {
                current_link = Some(url.clone());
                current_text.clear();
            }
            PreviewEvent::Text(text) => {
                if current_link.is_some() {
                    current_text.push_str(text);
                }
            }
            PreviewEvent::EndLink => {
                if let Some(url) = current_link.take() {
                    let label = if current_text.trim().is_empty() {
                        url.clone()
                    } else {
                        current_text.clone()
                    };
                    links.push((label, url));
                }
            }
            _ => {}
        }
    }

    links
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_text_by_urls_no_url() {
        let text = "Just some normal text without any links.";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![PreviewEvent::Text(
                "Just some normal text without any links.".to_string()
            )]
        );
    }

    #[test]
    fn test_split_text_by_urls_only_url() {
        let text = "https://example.com";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::StartLink("https://example.com".to_string()),
                PreviewEvent::Text("https://example.com".to_string()),
                PreviewEvent::EndLink,
            ]
        );
    }

    #[test]
    fn test_split_text_by_urls_empty_string() {
        let text = "";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert!(events.is_empty());
    }

    #[test]
    fn test_split_text_by_urls_whitespace_only() {
        let text = "   \n\t ";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(events, vec![PreviewEvent::Text("   \n\t ".to_string())]);
    }

    #[test]
    fn test_split_text_by_urls_unicode() {
        let text = "🦀 Check out https://rust-lang.org 🚀 for awesome Rust stuff!";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("🦀 Check out ".to_string()),
                PreviewEvent::StartLink("https://rust-lang.org".to_string()),
                PreviewEvent::Text("https://rust-lang.org".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(" 🚀 for awesome Rust stuff!".to_string()),
            ]
        );
    }

    #[test]
    fn test_split_text_by_urls_complex_url() {
        let text =
            "Link: https://user:pass@example.com:8080/path/to/resource?query=1&foo=bar#section-1.";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Link: ".to_string()),
                PreviewEvent::StartLink(
                    "https://user:pass@example.com:8080/path/to/resource?query=1&foo=bar#section-1"
                        .to_string()
                ),
                PreviewEvent::Text(
                    "https://user:pass@example.com:8080/path/to/resource?query=1&foo=bar#section-1"
                        .to_string()
                ),
                PreviewEvent::EndLink,
                PreviewEvent::Text(".".to_string()),
            ]
        );
    }

    #[test]
    fn test_split_text_by_urls_all_trailing_punctuation() {
        let text = "Url: https://example.com.,?!:;)]>";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Url: ".to_string()),
                PreviewEvent::StartLink("https://example.com".to_string()),
                PreviewEvent::Text("https://example.com".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(".,?!:;)]>".to_string()),
            ]
        );
    }

    #[test]
    fn test_split_text_by_urls_bare_schemes() {
        let text = "Bare schemes: http:// and https:// here";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Bare schemes: ".to_string()),
                PreviewEvent::StartLink("http://".to_string()),
                PreviewEvent::Text("http://".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(" and ".to_string()),
                PreviewEvent::StartLink("https://".to_string()),
                PreviewEvent::Text("https://".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(" here".to_string()),
            ]
        );
    }

    #[test]
    fn test_split_text_by_urls_adjacent_urls() {
        let text = "https://a.comhttps://b.com";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::StartLink("https://a.comhttps://b.com".to_string()),
                PreviewEvent::Text("https://a.comhttps://b.com".to_string()),
                PreviewEvent::EndLink,
            ]
        );
    }

    #[test]
    fn test_split_text_by_urls_url_in_middle() {
        let text = "Check out https://google.com for more info.";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Check out ".to_string()),
                PreviewEvent::StartLink("https://google.com".to_string()),
                PreviewEvent::Text("https://google.com".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(" for more info.".to_string()),
            ]
        );
    }

    #[test]
    fn test_split_text_by_urls_multiple_urls() {
        let text = "Visit http://a.com and https://b.com today.";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Visit ".to_string()),
                PreviewEvent::StartLink("http://a.com".to_string()),
                PreviewEvent::Text("http://a.com".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(" and ".to_string()),
                PreviewEvent::StartLink("https://b.com".to_string()),
                PreviewEvent::Text("https://b.com".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(" today.".to_string()),
            ]
        );
    }

    #[test]
    fn test_split_text_by_urls_trailing_punctuation() {
        let text = "Look at this (https://example.com/test)!";
        let mut events = Vec::new();
        split_text_by_urls(text, &mut events);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Look at this (".to_string()),
                PreviewEvent::StartLink("https://example.com/test".to_string()),
                PreviewEvent::Text("https://example.com/test".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(")!".to_string()),
            ]
        );
    }

    #[test]
    fn test_parse_markdown_paragraph() {
        let text = "This is a simple paragraph.";
        let events = parse_markdown(text, false);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("This is a simple paragraph.".to_string()),
                PreviewEvent::EndBlock
            ]
        );
    }

    #[test]
    fn test_parse_markdown_heading() {
        let text = "# Heading 1\nSome text.";
        let events = parse_markdown(text, false);
        assert_eq!(
            events,
            vec![
                PreviewEvent::StartHeading,
                PreviewEvent::Text("Heading 1".to_string()),
                PreviewEvent::EndBlock,
                PreviewEvent::Text("Some text.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_markdown_code() {
        let text = "Here is `some code` inline.";
        let events = parse_markdown(text, false);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Here is ".to_string()),
                PreviewEvent::Code("some code".to_string()),
                PreviewEvent::Text(" inline.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_markdown_breaks() {
        let text = "Line 1\nLine 2  \nLine 3";
        let events = parse_markdown(text, false);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Line 1".to_string()),
                PreviewEvent::Break,
                PreviewEvent::Text("Line 2".to_string()),
                PreviewEvent::Break,
                PreviewEvent::Text("Line 3".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_markdown_ignored_formatting() {
        // Italics and bold should just emit Text events without wrapping them in special formatting events.
        let text = "Some **bold** and *italic* text.";
        let events = parse_markdown(text, false);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Some ".to_string()),
                PreviewEvent::Text("bold".to_string()),
                PreviewEvent::Text(" and ".to_string()),
                PreviewEvent::Text("italic".to_string()),
                PreviewEvent::Text(" text.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_markdown_skip_fallback() {
        let text = "> <@alice:example.com> Hello\n\nHi!";
        let events = parse_markdown(text, true);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Hi!".to_string()),
                PreviewEvent::EndBlock
            ]
        );
    }

    #[test]
    fn test_parse_markdown_skip_fallback_no_blockquote() {
        let text = "Just some plain text.";
        let events = parse_markdown(text, true);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Just some plain text.".to_string()),
                PreviewEvent::EndBlock
            ]
        );
    }

    #[test]
    fn test_parse_markdown_no_skip_normal_blockquote() {
        let text = "> This is a quote\n\nAnd this is text.";
        let events = parse_markdown(text, false);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("This is a quote".to_string()),
                PreviewEvent::EndBlock,
                PreviewEvent::Text("And this is text.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_plain_text_urls() {
        let text = "Check out https://google.com/search?q=test, it is awesome!";
        let events = parse_markdown(text, false);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Check out ".to_string()),
                PreviewEvent::StartLink("https://google.com/search?q=test".to_string()),
                PreviewEvent::Text("https://google.com/search?q=test".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(", it is awesome!".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_plain_text_only_urls() {
        let text = "Check out https://google.com/search?q=test, it is awesome!";
        let events = parse_plain_text(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Check out ".to_string()),
                PreviewEvent::StartLink("https://google.com/search?q=test".to_string()),
                PreviewEvent::Text("https://google.com/search?q=test".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(", it is awesome!".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_plain_text_markdown_link() {
        let text = "See [this is text](https://example.org) for details";
        let events = parse_plain_text(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("See ".to_string()),
                PreviewEvent::StartLink("https://example.org".to_string()),
                PreviewEvent::Text("this is text".to_string()),
                PreviewEvent::EndLink,
                PreviewEvent::Text(" for details".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_extract_links() {
        let text = "Check out [Google](https://google.com) and [](https://github.com) or just https://rust-lang.org!";
        let events = parse_markdown(text, false);
        let links = extract_links(&events);
        assert_eq!(
            links,
            vec![
                ("Google".to_string(), "https://google.com".to_string()),
                (
                    "https://github.com".to_string(),
                    "https://github.com".to_string()
                ),
                (
                    "https://rust-lang.org".to_string(),
                    "https://rust-lang.org".to_string()
                ),
            ]
        );
    }

    #[test]
    fn test_extract_links_empty() {
        let events = vec![];
        let links = extract_links(&events);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_no_links() {
        let events = vec![
            PreviewEvent::Text("Just some text".to_string()),
            PreviewEvent::Break,
        ];
        let links = extract_links(&events);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_empty_text() {
        let events = vec![
            PreviewEvent::StartLink("https://example.com".to_string()),
            PreviewEvent::EndLink,
        ];
        let links = extract_links(&events);
        assert_eq!(
            links,
            vec![(
                "https://example.com".to_string(),
                "https://example.com".to_string()
            )]
        );
    }

    #[test]
    fn test_extract_links_whitespace_text() {
        let events = vec![
            PreviewEvent::StartLink("https://example.com".to_string()),
            PreviewEvent::Text("   ".to_string()),
            PreviewEvent::EndLink,
        ];
        let links = extract_links(&events);
        assert_eq!(
            links,
            vec![(
                "https://example.com".to_string(),
                "https://example.com".to_string()
            )]
        );
    }

    #[test]
    fn test_extract_links_multiple_text_events() {
        let events = vec![
            PreviewEvent::StartLink("https://example.com".to_string()),
            PreviewEvent::Text("Example ".to_string()),
            PreviewEvent::Text("Site".to_string()),
            PreviewEvent::EndLink,
        ];
        let links = extract_links(&events);
        assert_eq!(
            links,
            vec![(
                "Example Site".to_string(),
                "https://example.com".to_string()
            )]
        );
    }

    #[test]
    fn test_extract_links_unclosed_link() {
        let events = vec![
            PreviewEvent::StartLink("https://example.com".to_string()),
            PreviewEvent::Text("Example".to_string()),
        ];
        let links = extract_links(&events);
        assert!(links.is_empty());
    }

    #[test]
    fn test_extract_links_ignored_events() {
        let events = vec![
            PreviewEvent::StartLink("https://example.com".to_string()),
            PreviewEvent::Text("Example ".to_string()),
            PreviewEvent::Code("code".to_string()), // Should be ignored by extract_links, just like other non-text formatting inside a link would be.
            PreviewEvent::Text(" Site".to_string()),
            PreviewEvent::EndLink,
        ];
        let links = extract_links(&events);
        // Note: the current implementation only accumulates Text events.
        // So Code events are ignored and won't contribute to the label text.
        assert_eq!(
            links,
            vec![(
                "Example  Site".to_string(),
                "https://example.com".to_string()
            )]
        );
    }

    #[test]
    fn test_parse_plain_text_simple() {
        let text = "Just a simple text";
        let events = parse_plain_text(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Just a simple text".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_plain_text_heading_ignored() {
        let text = "# Heading 1
Some text.";
        let events = parse_plain_text(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Heading 1".to_string()),
                PreviewEvent::EndBlock,
                PreviewEvent::Text("Some text.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_plain_text_code() {
        let text = "Here is `some code` inline.";
        let events = parse_plain_text(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Here is ".to_string()),
                PreviewEvent::Code("some code".to_string()),
                PreviewEvent::Text(" inline.".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_plain_text_breaks() {
        let text = "Line 1
Line 2
Line 3";
        let events = parse_plain_text(text);
        assert_eq!(
            events,
            vec![
                PreviewEvent::Text("Line 1".to_string()),
                PreviewEvent::Break,
                PreviewEvent::Text("Line 2".to_string()),
                PreviewEvent::Break,
                PreviewEvent::Text("Line 3".to_string()),
                PreviewEvent::EndBlock,
            ]
        );
    }

    #[test]
    fn test_parse_custom_emoji_tag() {
        let tag1 =
            r#"<img data-mx-emoticon src="mxc://example.org/cat" alt=":cat:" title=":cat:" />"#;
        assert_eq!(
            parse_custom_emoji_tag(tag1),
            Some(PreviewEvent::CustomEmoji {
                url: "mxc://example.org/cat".to_string(),
                alt: ":cat:".to_string(),
            })
        );

        let tag2 = r#"<img src='mxc://example.org/dog' alt='doge' data-mx-emoticon>"#;
        assert_eq!(
            parse_custom_emoji_tag(tag2),
            Some(PreviewEvent::CustomEmoji {
                url: "mxc://example.org/dog".to_string(),
                alt: "doge".to_string(),
            })
        );

        // Regular image tag without data-mx-emoticon should not be parsed as custom emoji
        let tag3 = r#"<img src="https://example.com/pic.png" alt="pic" />"#;
        assert_eq!(parse_custom_emoji_tag(tag3), None);

        // Non-img tag should return None
        let tag4 = r#"<div data-mx-emoticon>text</div>"#;
        assert_eq!(parse_custom_emoji_tag(tag4), None);
    }

    #[test]
    fn test_parse_markdown_with_custom_emoji() {
        let text =
            "Hello <img data-mx-emoticon src=\"mxc://example.org/cat\" alt=\":cat:\" /> world!";
        let events = parse_markdown(text, false);
        assert!(events.contains(&PreviewEvent::CustomEmoji {
            url: "mxc://example.org/cat".to_string(),
            alt: ":cat:".to_string(),
        }));
    }

    #[hegel::test]
    fn prop_parse_custom_emoji_tag_no_panic(tc: hegel::TestCase) {
        let input: String = tc.draw(hegel::generators::text());
        let _ = parse_custom_emoji_tag(&input);
    }

    #[hegel::test]
    fn prop_parse_custom_emoji_tag_valid_inputs(tc: hegel::TestCase) {
        let url: String = tc.draw(
            hegel::generators::text()
                .min_size(1)
                .max_size(30)
                .alphabet("abcdefghijklmnopqrstuvwxyz0123456789_:/.-"),
        );
        let alt: String = tc.draw(
            hegel::generators::text()
                .min_size(1)
                .max_size(20)
                .alphabet("abcdefghijklmnopqrstuvwxyz0123456789_:"),
        );

        let tag = format!(r#"<img data-mx-emoticon src="{url}" alt="{alt}" />"#);
        let parsed = parse_custom_emoji_tag(&tag);
        assert_eq!(parsed, Some(PreviewEvent::CustomEmoji { url, alt }));
    }

    #[hegel::test]
    fn prop_split_text_by_urls_conservation(tc: hegel::TestCase) {
        let text: String = tc.draw(hegel::generators::text());
        let mut events = Vec::new();
        split_text_by_urls(&text, &mut events);

        let mut reconstructed = String::new();
        for event in &events {
            match event {
                PreviewEvent::Text(s) => reconstructed.push_str(s),
                PreviewEvent::StartLink(_) | PreviewEvent::EndLink => {}
                _ => panic!("split_text_by_urls produced unexpected event: {event:?}"),
            }
        }
        assert_eq!(reconstructed, text);
    }

    #[hegel::test]
    fn prop_parse_markdown_never_panics(tc: hegel::TestCase) {
        let text: String = tc.draw(hegel::generators::text());
        let skip_first: bool = tc.draw(hegel::generators::booleans());
        let _ = parse_markdown(&text, skip_first);
    }

    #[hegel::test]
    fn prop_parse_plain_text_never_panics(tc: hegel::TestCase) {
        let text: String = tc.draw(hegel::generators::text());
        let _ = parse_plain_text(&text);
    }
}
