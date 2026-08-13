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

    let mut default_headers = reqwest::header::HeaderMap::new();
    default_headers.insert(
        reqwest::header::ACCEPT,
        "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8"
            .parse()
            .unwrap(),
    );
    default_headers.insert(
        reqwest::header::ACCEPT_LANGUAGE,
        "en-US,en;q=0.9".parse().unwrap(),
    );

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(8))
        .user_agent("Mozilla/5.0 (X11; Linux x86_64; rv:128.0) Gecko/20100101 Firefox/128.0")
        .default_headers(default_headers)
        .build()
        .ok()?;

    let response = client.get(&url_str).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    let path_lower = parsed_url.path().to_lowercase();
    let is_direct_image = content_type.starts_with("image/")
        || path_lower.ends_with(".png")
        || path_lower.ends_with(".jpg")
        || path_lower.ends_with(".jpeg")
        || path_lower.ends_with(".gif")
        || path_lower.ends_with(".webp")
        || path_lower.ends_with(".svg")
        || path_lower.ends_with(".avif");

    if is_direct_image {
        let bytes = response.bytes().await.ok()?;
        if bytes.len() > 5 * 1024 * 1024 {
            return None;
        }
        let filename = parsed_url
            .path_segments()
            .and_then(|mut s| s.next_back())
            .filter(|s| !s.is_empty())
            .unwrap_or(&domain)
            .to_string();

        let image = Some(image::Handle::from_bytes(bytes.to_vec()));
        return Some(OgPreview {
            url: url_str.clone(),
            title: Some(filename),
            description: None,
            site_name: Some(domain.clone()),
            domain,
            image_url: Some(url_str),
            image,
        });
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
    if bytes.len() > 5 * 1024 * 1024 {
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
    if needle.is_empty() {
        return Some(0);
    }
    if needle.is_ascii() {
        let needle_len = needle.len();
        let needle_lower = needle.to_ascii_lowercase();
        haystack.char_indices().find_map(|(i, _)| {
            let end = i + needle_len;
            if end <= haystack.len()
                && haystack.is_char_boundary(end)
                && haystack[i..end].eq_ignore_ascii_case(&needle_lower)
            {
                Some(i)
            } else {
                None
            }
        })
    } else {
        let needle_lower = needle.to_lowercase();
        haystack.to_lowercase().find(&needle_lower)
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

    #[test]
    fn test_find_insensitive_multibyte_utf8() {
        let html = "Hello … world <meta property=\"og:title\" content=\"Test\">";
        let pos = find_insensitive(html, "content");
        assert!(pos.is_some());
        let (title, _, _, _) = parse_og_meta(html);
        assert_eq!(title.as_deref(), Some("Test"));
    }
    #[tokio::test]
    async fn test_direct_image_url_detection() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let webp_bytes: Vec<u8> = vec![
            0x52, 0x49, 0x46, 0x46, 0x1a, 0x00, 0x00, 0x00, 0x57, 0x45, 0x42, 0x50, 0x56, 0x50,
            0x38, 0x20, 0x0e, 0x00, 0x00, 0x00, 0xb0, 0x01, 0x00, 0x9d, 0x01, 0x2a, 0x01, 0x00,
            0x01, 0x00, 0x02, 0x00, 0x34, 0x25,
        ];

        Mock::given(method("GET"))
            .and(path("/image.webp"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "image/webp")
                    .set_body_bytes(webp_bytes),
            )
            .mount(&server)
            .await;

        let url = format!("{}/image.webp", server.uri());
        let preview = fetch_og_preview(url.clone()).await;
        assert!(preview.is_some());
        let og = preview.unwrap();
        assert_eq!(og.title.as_deref(), Some("image.webp"));
        assert!(og.image.is_some());
    }

    #[tokio::test]
    async fn test_og_preview_with_mock_server() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        let html = r#"
            <!DOCTYPE html>
            <html>
            <head>
                <meta property="og:title" content="Codeberg Release" />
                <meta property="og:site_name" content="Codeberg.org" />
            </head>
            <body></body>
            </html>
        "#;

        Mock::given(method("GET"))
            .and(path("/releases"))
            .respond_with(
                ResponseTemplate::new(200)
                    .insert_header("content-type", "text/html; charset=utf-8")
                    .set_body_string(html),
            )
            .mount(&server)
            .await;

        let url = format!("{}/releases", server.uri());
        let preview = fetch_og_preview(url.clone()).await;
        assert!(preview.is_some());
        let og = preview.unwrap();
        assert_eq!(og.title.as_deref(), Some("Codeberg Release"));
        assert_eq!(og.site_name.as_deref(), Some("Codeberg.org"));
    }
}
