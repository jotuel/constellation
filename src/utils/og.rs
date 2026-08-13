use cosmic::iced::widget::image;
use reqwest::Client;
use std::sync::Arc;
use url::Url;

#[derive(Clone, Debug, PartialEq)]
pub struct OgPreview {
    pub url: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub site_name: Option<String>,
    pub domain: String,
    pub image_url: Option<String>,
    pub image: Option<image::Handle>,
}

#[derive(Clone, Debug)]
pub enum OgState {
    Pending,
    Loaded(Arc<OgPreview>),
    Failed,
}

pub async fn fetch_og_preview(url_str: String) -> Option<OgPreview> {
    let parsed_url = Url::parse(&url_str).ok()?;
    if !matches!(parsed_url.scheme(), "http" | "https") {
        return None;
    }
    let domain = parsed_url.host_str()?.to_string();

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(5))
        .user_agent(
            "Mozilla/5.0 (compatible; Constellation/0.1; +https://github.com/pop-os/libcosmic)",
        )
        .build()
        .ok()?;

    let response = client.get(&url_str).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let bytes = response.bytes().await.ok()?;
    let max_len = bytes.len().min(512 * 1024);
    let html_slice = &bytes[..max_len];
    let html_str = String::from_utf8_lossy(html_slice);

    let (mut title, description, site_name, raw_image_url) = parse_og_meta(&html_str);

    if title.is_none() {
        title = parse_title_tag(&html_str);
    }

    let description = description.or_else(|| parse_meta_description(&html_str));

    if title.is_none() && description.is_none() && raw_image_url.is_none() {
        return None;
    }

    let image_url =
        raw_image_url.and_then(|img_rel| parsed_url.join(&img_rel).ok().map(|u| u.to_string()));

    let image = if let Some(img_url) = &image_url {
        fetch_image_handle(&client, img_url).await
    } else {
        None
    };

    Some(OgPreview {
        url: url_str,
        title,
        description,
        site_name,
        domain,
        image_url,
        image,
    })
}

async fn fetch_image_handle(client: &Client, img_url: &str) -> Option<image::Handle> {
    let resp = client.get(img_url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let bytes = resp.bytes().await.ok()?;
    if bytes.len() > 2 * 1024 * 1024 {
        return None;
    }
    Some(image::Handle::from_bytes(bytes.to_vec()))
}

pub fn parse_og_meta(
    html: &str,
) -> (
    Option<String>,
    Option<String>,
    Option<String>,
    Option<String>,
) {
    let mut title = None;
    let mut description = None;
    let mut site_name = None;
    let mut image = None;

    let mut cursor = html;
    while let Some(start_idx) = find_insensitive(cursor, "<meta") {
        let tag_slice = &cursor[start_idx..];
        let end_idx = match tag_slice.find('>') {
            Some(i) => i,
            None => break,
        };
        let meta_tag = &tag_slice[..end_idx];

        let prop = get_attribute_value(meta_tag, "property")
            .or_else(|| get_attribute_value(meta_tag, "name"));
        let content = get_attribute_value(meta_tag, "content");

        if let (Some(p), Some(c)) = (prop, content) {
            let p_lower = p.to_lowercase();
            let decoded_content = decode_html_entities(&c);
            if !decoded_content.trim().is_empty() {
                match p_lower.as_str() {
                    "og:title" if title.is_none() => title = Some(decoded_content),
                    "og:description" if description.is_none() => {
                        description = Some(decoded_content)
                    }
                    "og:site_name" if site_name.is_none() => site_name = Some(decoded_content),
                    "og:image" | "og:image:src" if image.is_none() => {
                        image = Some(decoded_content);
                    }
                    _ => {}
                }
            }
        }

        cursor = &tag_slice[end_idx + 1..];
    }

    (title, description, site_name, image)
}

pub fn parse_meta_description(html: &str) -> Option<String> {
    let mut cursor = html;
    while let Some(start_idx) = find_insensitive(cursor, "<meta") {
        let tag_slice = &cursor[start_idx..];
        let end_idx = match tag_slice.find('>') {
            Some(i) => i,
            None => break,
        };
        let meta_tag = &tag_slice[..end_idx];

        let name = get_attribute_value(meta_tag, "name")
            .or_else(|| get_attribute_value(meta_tag, "property"));
        let content = get_attribute_value(meta_tag, "content");

        if let (Some(n), Some(c)) = (name, content)
            && n.eq_ignore_ascii_case("description")
        {
            let decoded = decode_html_entities(&c);
            if !decoded.trim().is_empty() {
                return Some(decoded);
            }
        }
        cursor = &tag_slice[end_idx + 1..];
    }
    None
}

pub fn parse_title_tag(html: &str) -> Option<String> {
    let start_idx = find_insensitive(html, "<title")?;
    let tag_slice = &html[start_idx..];
    let open_end = tag_slice.find('>')?;
    let content_start = open_end + 1;
    let close_idx = find_insensitive(&tag_slice[content_start..], "</title")?;
    let raw_title = &tag_slice[content_start..content_start + close_idx];
    let decoded = decode_html_entities(raw_title.trim());
    if decoded.is_empty() {
        None
    } else {
        Some(decoded)
    }
}

fn find_insensitive(haystack: &str, needle: &str) -> Option<usize> {
    if needle.is_ascii() {
        let needle_lower = needle.to_ascii_lowercase();
        haystack.bytes().enumerate().find_map(|(i, _)| {
            if haystack[i..].len() >= needle.len()
                && haystack[i..i + needle.len()].eq_ignore_ascii_case(&needle_lower)
            {
                Some(i)
            } else {
                None
            }
        })
    } else {
        haystack.to_lowercase().find(&needle.to_lowercase())
    }
}

fn get_attribute_value(tag: &str, attr_name: &str) -> Option<String> {
    let mut cursor = tag;
    while let Some(pos) = find_insensitive(cursor, attr_name) {
        let attr_slice = &cursor[pos..];
        let before_char = if pos == 0 {
            ' '
        } else {
            cursor[..pos].chars().last().unwrap_or(' ')
        };
        let after_name = &attr_slice[attr_name.len()..];
        let after_char = after_name.chars().next().unwrap_or(' ');

        if (before_char.is_whitespace() || before_char == '<' || before_char == '/')
            && (after_char.is_whitespace() || after_char == '=')
            && let Some(eq_pos) = after_name.find('=')
        {
            let rest = after_name[eq_pos + 1..].trim_start();
            if let Some(inner) = rest.strip_prefix('"') {
                if let Some(end_q) = inner.find('"') {
                    return Some(inner[..end_q].to_string());
                }
            } else if let Some(inner) = rest.strip_prefix('\'') {
                if let Some(end_q) = inner.find('\'') {
                    return Some(inner[..end_q].to_string());
                }
            } else {
                let end = rest
                    .find(|c: char| c.is_whitespace() || c == '>' || c == '/')
                    .unwrap_or(rest.len());
                return Some(rest[..end].to_string());
            }
        }
        cursor = &attr_slice[attr_name.len()..];
    }
    None
}

pub fn decode_html_entities(input: &str) -> String {
    input
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_og_meta() {
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta property="og:title" content="Test Page Title" />
                <meta property="og:description" content="Test description for &amp; testing" />
                <meta property="og:site_name" content="Test Site" />
                <meta property="og:image" content="https://example.com/image.png" />
            </head>
            <body></body>
            </html>
        "#;

        let (title, description, site_name, image) = parse_og_meta(html);
        assert_eq!(title.as_deref(), Some("Test Page Title"));
        assert_eq!(
            description.as_deref(),
            Some("Test description for & testing")
        );
        assert_eq!(site_name.as_deref(), Some("Test Site"));
        assert_eq!(image.as_deref(), Some("https://example.com/image.png"));
    }

    #[test]
    fn test_parse_title_tag_and_meta_desc_fallback() {
        let html = r#"
            <html>
            <head>
                <title>Fallback Title</title>
                <meta name="description" content="Fallback Description" />
            </head>
            </html>
        "#;

        let title = parse_title_tag(html);
        let desc = parse_meta_description(html);

        assert_eq!(title.as_deref(), Some("Fallback Title"));
        assert_eq!(desc.as_deref(), Some("Fallback Description"));
    }

    #[test]
    fn test_decode_html_entities() {
        assert_eq!(
            decode_html_entities("Hello &amp; World &#39;test&#39;"),
            "Hello & World 'test'"
        );
    }
}
