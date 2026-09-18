use crate::matrix::GlobalSearchScope;
use chrono::NaiveDate;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ParsedSearchQuery {
    pub room_filter: Option<String>,
    pub sender_filter: Option<String>,
    pub scope: Option<GlobalSearchScope>,
    pub date_after: Option<NaiveDate>,
    pub date_before: Option<NaiveDate>,
    pub sanitized_query: String,
}

impl ParsedSearchQuery {
    /// Returns `true` if all filters are empty and the sanitized query is empty.
    pub fn is_empty(&self) -> bool {
        self.room_filter.is_none()
            && self.sender_filter.is_none()
            && self.scope.is_none()
            && self.date_after.is_none()
            && self.date_before.is_none()
            && self.sanitized_query.is_empty()
    }

    /// Constructs a safe Tantivy query string for local indexing.
    ///
    /// If `resolved_sender_id` is provided, it takes precedence over `sender_filter`.
    pub fn to_tantivy_query(&self, resolved_sender_id: Option<&str>) -> String {
        let mut clauses = Vec::new();

        if let Some(sender) = resolved_sender_id.or(self.sender_filter.as_deref()) {
            let clean = sender.trim();
            if !clean.is_empty() {
                let escaped = clean.replace('\\', "\\\\").replace('"', "\\\"");
                clauses.push(format!("sender:\"{escaped}\""));
            }
        }

        match (self.date_after, self.date_before) {
            (Some(after), Some(before)) => {
                clauses.push(format!(
                    "date:[{}T00:00:00Z TO {}T23:59:59Z]",
                    after.format("%Y-%m-%d"),
                    before.format("%Y-%m-%d")
                ));
            }
            (Some(after), None) => {
                clauses.push(format!(
                    "date:[{}T00:00:00Z TO *]",
                    after.format("%Y-%m-%d")
                ));
            }
            (None, Some(before)) => {
                clauses.push(format!(
                    "date:[* TO {}T23:59:59Z]",
                    before.format("%Y-%m-%d")
                ));
            }
            (None, None) => {}
        }

        if !self.sanitized_query.is_empty() {
            if clauses.is_empty() {
                clauses.push(self.sanitized_query.clone());
            } else {
                clauses.push(format!("({})", self.sanitized_query));
            }
        }

        clauses.join(" AND ")
    }
}

/// Parses raw user search input into structured tokens and a Tantivy-sanitized query.
pub fn parse_search_query(input: &str) -> ParsedSearchQuery {
    let mut room_filter = None;
    let mut sender_filter = None;
    let mut scope = None;
    let mut date_after = None;
    let mut date_before = None;
    let mut plain_terms = Vec::new();

    let tokens = tokenize_search_input(input);

    for token in tokens {
        if let Some(room) = parse_room_token(&token) {
            room_filter = Some(room);
            continue;
        }

        if let Some(sender) = parse_sender_token(&token) {
            sender_filter = Some(sender);
            continue;
        }

        if let Some(sc) = parse_scope_token(&token) {
            scope = Some(sc);
            continue;
        }

        if let Some(date) = parse_date_after_token(&token) {
            date_after = Some(date);
            continue;
        }
        if let Some(date) = parse_date_before_token(&token) {
            date_before = Some(date);
            continue;
        }

        let sanitized = sanitize_tantivy_token(&token);
        if !sanitized.is_empty() {
            plain_terms.push(sanitized);
        }
    }

    let sanitized_query = plain_terms.join(" ");

    ParsedSearchQuery {
        room_filter,
        sender_filter,
        scope,
        date_after,
        date_before,
        sanitized_query,
    }
}

fn strip_quotes(s: &str) -> &str {
    s.trim_matches('"')
}

fn parse_room_token(token: &str) -> Option<String> {
    if let Some(rest) = token.strip_prefix("in:") {
        let trimmed = strip_quotes(rest);
        let val = trimmed.strip_prefix('#').unwrap_or(trimmed);
        if !val.is_empty() {
            return Some(val.to_string());
        }
    } else if let Some(rest) = token.strip_prefix('#') {
        let val = strip_quotes(rest);
        if !val.is_empty() {
            return Some(val.to_string());
        }
    } else if token.starts_with('!') && token.contains(':') {
        return Some(token.to_string());
    }
    None
}

fn parse_sender_token(token: &str) -> Option<String> {
    if let Some(rest) = token.strip_prefix("from:") {
        let val = strip_quotes(rest);
        if !val.is_empty() {
            return Some(val.to_string());
        }
    } else if let Some(rest) = token.strip_prefix('@') {
        let val = strip_quotes(rest);
        if !val.is_empty() {
            return Some(format!("@{val}"));
        }
    }
    None
}

fn parse_scope_token(token: &str) -> Option<GlobalSearchScope> {
    let lower = token.to_lowercase();
    let val = lower
        .strip_prefix("is:")
        .or_else(|| lower.strip_prefix("scope:"))?;
    match val {
        "dm" | "dms" => Some(GlobalSearchScope::DmsOnly),
        "group" | "groups" => Some(GlobalSearchScope::GroupsOnly),
        "all" => Some(GlobalSearchScope::All),
        _ => None,
    }
}

fn parse_date_after_token(token: &str) -> Option<NaiveDate> {
    let val = token
        .strip_prefix("after:")
        .or_else(|| token.strip_prefix("since:"))?;
    let val = strip_quotes(val);
    NaiveDate::parse_from_str(val, "%Y-%m-%d").ok()
}

fn parse_date_before_token(token: &str) -> Option<NaiveDate> {
    let val = token
        .strip_prefix("before:")
        .or_else(|| token.strip_prefix("until:"))?;
    let val = strip_quotes(val);
    NaiveDate::parse_from_str(val, "%Y-%m-%d").ok()
}

/// Tokenizes search input into whitespace-delimited tokens while respecting quotes.
fn tokenize_search_input(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        while i < len && chars[i].is_whitespace() {
            i += 1;
        }
        if i >= len {
            break;
        }

        let mut token = String::new();
        if chars[i..].starts_with(&['i', 'n', ':', '"']) {
            token.push_str("in:\"");
            i += 4;
            while i < len && chars[i] != '"' {
                token.push(chars[i]);
                i += 1;
            }
            if i < len && chars[i] == '"' {
                token.push('"');
                i += 1;
            } else {
                token.push('"');
            }
            tokens.push(token);
            continue;
        } else if chars[i..].starts_with(&['f', 'r', 'o', 'm', ':', '"']) {
            token.push_str("from:\"");
            i += 6;
            while i < len && chars[i] != '"' {
                token.push(chars[i]);
                i += 1;
            }
            if i < len && chars[i] == '"' {
                token.push('"');
                i += 1;
            } else {
                token.push('"');
            }
            tokens.push(token);
            continue;
        } else if chars[i..].starts_with(&['#', '"']) {
            token.push_str("#\"");
            i += 2;
            while i < len && chars[i] != '"' {
                token.push(chars[i]);
                i += 1;
            }
            if i < len && chars[i] == '"' {
                token.push('"');
                i += 1;
            } else {
                token.push('"');
            }
            tokens.push(token);
            continue;
        } else if chars[i..].starts_with(&['@', '"']) {
            token.push_str("@\"");
            i += 2;
            while i < len && chars[i] != '"' {
                token.push(chars[i]);
                i += 1;
            }
            if i < len && chars[i] == '"' {
                token.push('"');
                i += 1;
            } else {
                token.push('"');
            }
            tokens.push(token);
            continue;
        } else if chars[i] == '"' {
            token.push('"');
            i += 1;
            while i < len && chars[i] != '"' {
                token.push(chars[i]);
                i += 1;
            }
            if i < len && chars[i] == '"' {
                token.push('"');
                i += 1;
            } else {
                token.push('"');
            }
            tokens.push(token);
            continue;
        }

        while i < len && !chars[i].is_whitespace() {
            token.push(chars[i]);
            i += 1;
        }
        if !token.is_empty() {
            tokens.push(token);
        }
    }

    tokens
}

/// Sanitizes a single plain text search token for safe consumption by Tantivy QueryParser.
fn sanitize_tantivy_token(token: &str) -> String {
    if token.starts_with('"') && token.ends_with('"') && token.len() >= 2 {
        let inner = &token[1..token.len() - 1];
        let mut escaped_inner = String::with_capacity(inner.len());
        for c in inner.chars() {
            if c == '\\' || c == '"' {
                escaped_inner.push('\\');
            }
            escaped_inner.push(c);
        }
        return format!("\"{escaped_inner}\"");
    }

    if token == "AND" || token == "OR" || token == "NOT" {
        return token.to_string();
    }

    let mut sanitized = String::with_capacity(token.len() + 4);
    for c in token.chars() {
        match c {
            ':' | '(' | ')' | '[' | ']' | '{' | '}' | '^' | '~' | '!' | '\\' | '"' | '\'' => {
                sanitized.push('\\');
                sanitized.push(c);
            }
            _ => sanitized.push(c),
        }
    }
    sanitized
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutocompleteTrigger<'a> {
    Room { needle: &'a str },
    Member { needle: &'a str },
}

/// Detects if the current user input is actively typing a room or member trigger.
pub fn extract_autocomplete_trigger(input: &str) -> Option<AutocompleteTrigger<'_>> {
    if input.is_empty() || input.chars().last().is_some_and(char::is_whitespace) {
        return None;
    }

    // Check for open quote at the end:
    if let Some(idx) = input.rfind("in:\"") {
        let after = &input[idx + 4..];
        if !after.contains('"') {
            return Some(AutocompleteTrigger::Room { needle: after });
        }
    }
    if let Some(idx) = input.rfind("#\"") {
        let after = &input[idx + 2..];
        if !after.contains('"') {
            return Some(AutocompleteTrigger::Room { needle: after });
        }
    }
    if let Some(idx) = input.rfind("from:\"") {
        let after = &input[idx + 6..];
        if !after.contains('"') {
            return Some(AutocompleteTrigger::Member { needle: after });
        }
    }
    if let Some(idx) = input.rfind("@\"") {
        let after = &input[idx + 2..];
        if !after.contains('"') {
            return Some(AutocompleteTrigger::Member { needle: after });
        }
    }

    let last_token = match input.char_indices().rev().find(|(_, c)| c.is_whitespace()) {
        Some((idx, c)) => &input[idx + c.len_utf8()..],
        None => input,
    };

    if let Some(needle) = last_token.strip_prefix("in:#") {
        Some(AutocompleteTrigger::Room { needle })
    } else if let Some(needle) = last_token.strip_prefix("in:") {
        Some(AutocompleteTrigger::Room { needle })
    } else if let Some(needle) = last_token.strip_prefix('#') {
        Some(AutocompleteTrigger::Room { needle })
    } else if let Some(needle) = last_token.strip_prefix("from:@") {
        Some(AutocompleteTrigger::Member { needle })
    } else if let Some(needle) = last_token.strip_prefix("from:") {
        Some(AutocompleteTrigger::Member { needle })
    } else {
        last_token
            .strip_prefix('@')
            .map(|needle| AutocompleteTrigger::Member { needle })
    }
}

/// Replaces the active trigger token being typed with the selected suggestion.
pub fn apply_autocomplete_replacement(current_input: &str, replacement: &str) -> String {
    for prefix in &["in:\"", "#\"", "from:\"", "@\""] {
        if let Some(idx) = current_input.rfind(prefix) {
            let after = &current_input[idx + prefix.len()..];
            if !after.contains('"') {
                let mut res = current_input[..idx].to_string();
                res.push_str(replacement);
                return res;
            }
        }
    }

    match current_input
        .char_indices()
        .rev()
        .find(|(_, c)| c.is_whitespace())
    {
        Some((idx, c)) => {
            let end_idx = idx + c.len_utf8();
            let mut res = current_input[..end_idx].to_string();
            res.push_str(replacement);
            res
        }
        None => replacement.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_parse_plain_query() {
        let parsed = parse_search_query("hello world");
        assert_eq!(parsed.sanitized_query, "hello world");
        assert_eq!(parsed.room_filter, None);
        assert_eq!(parsed.sender_filter, None);
        assert_eq!(parsed.scope, None);
        assert_eq!(parsed.date_after, None);
        assert_eq!(parsed.date_before, None);
    }

    #[test]
    fn test_parse_room_filter() {
        let p1 = parse_search_query("#general hello");
        assert_eq!(p1.room_filter.as_deref(), Some("general"));
        assert_eq!(p1.sanitized_query, "hello");

        let p2 = parse_search_query("in:general hello");
        assert_eq!(p2.room_filter.as_deref(), Some("general"));
        assert_eq!(p2.sanitized_query, "hello");

        let p3 = parse_search_query("in:#general hello");
        assert_eq!(p3.room_filter.as_deref(), Some("general"));
        assert_eq!(p3.sanitized_query, "hello");
    }

    #[test]
    fn test_parse_quoted_room_filter() {
        let p1 = parse_search_query(r#"#"General Chat" hello"#);
        assert_eq!(p1.room_filter.as_deref(), Some("General Chat"));
        assert_eq!(p1.sanitized_query, "hello");

        let p2 = parse_search_query(r#"in:"General Chat" hello"#);
        assert_eq!(p2.room_filter.as_deref(), Some("General Chat"));
        assert_eq!(p2.sanitized_query, "hello");
    }

    #[test]
    fn test_parse_room_id() {
        let p = parse_search_query("!abc123xyz:matrix.org urgent");
        assert_eq!(p.room_filter.as_deref(), Some("!abc123xyz:matrix.org"));
        assert_eq!(p.sanitized_query, "urgent");

        let p2 = parse_search_query("in:!abc123xyz:matrix.org urgent");
        assert_eq!(p2.room_filter.as_deref(), Some("!abc123xyz:matrix.org"));
        assert_eq!(p2.sanitized_query, "urgent");
    }

    #[test]
    fn test_parse_sender_filter() {
        let p1 = parse_search_query("@alice hello");
        assert_eq!(p1.sender_filter.as_deref(), Some("@alice"));
        assert_eq!(p1.sanitized_query, "hello");

        let p2 = parse_search_query("@alice:matrix.org hello");
        assert_eq!(p2.sender_filter.as_deref(), Some("@alice:matrix.org"));
        assert_eq!(p2.sanitized_query, "hello");

        let p3 = parse_search_query("from:alice hello");
        assert_eq!(p3.sender_filter.as_deref(), Some("alice"));
        assert_eq!(p3.sanitized_query, "hello");

        let p4 = parse_search_query(r#"from:"Alice Smith" hello"#);
        assert_eq!(p4.sender_filter.as_deref(), Some("Alice Smith"));
        assert_eq!(p4.sanitized_query, "hello");

        let p5 = parse_search_query(r#"@"Alice Smith" hello"#);
        assert_eq!(p5.sender_filter.as_deref(), Some("@Alice Smith"));
        assert_eq!(p5.sanitized_query, "hello");
    }

    #[test]
    fn test_parse_scope_filter() {
        let p1 = parse_search_query("is:dm hello");
        assert_eq!(p1.scope, Some(GlobalSearchScope::DmsOnly));
        assert_eq!(p1.sanitized_query, "hello");

        let p2 = parse_search_query("scope:group hello");
        assert_eq!(p2.scope, Some(GlobalSearchScope::GroupsOnly));
        assert_eq!(p2.sanitized_query, "hello");

        let p3 = parse_search_query("is:all hello");
        assert_eq!(p3.scope, Some(GlobalSearchScope::All));
        assert_eq!(p3.sanitized_query, "hello");
    }

    #[test]
    fn test_parse_date_filters() {
        let p1 = parse_search_query("after:2026-01-01 before:2026-06-30 meeting");
        assert_eq!(
            p1.date_after,
            Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        );
        assert_eq!(
            p1.date_before,
            Some(NaiveDate::from_ymd_opt(2026, 6, 30).unwrap())
        );
        assert_eq!(p1.sanitized_query, "meeting");

        let p2 = parse_search_query("since:2026-02-15 until:2026-03-01 report");
        assert_eq!(
            p2.date_after,
            Some(NaiveDate::from_ymd_opt(2026, 2, 15).unwrap())
        );
        assert_eq!(
            p2.date_before,
            Some(NaiveDate::from_ymd_opt(2026, 3, 1).unwrap())
        );
        assert_eq!(p2.sanitized_query, "report");
    }

    #[test]
    fn test_parse_combined_query() {
        let q =
            r#"#general @alice:matrix.org is:group after:2026-01-01 "quarterly report" foo:bar"#;
        let parsed = parse_search_query(q);
        assert_eq!(parsed.room_filter.as_deref(), Some("general"));
        assert_eq!(parsed.sender_filter.as_deref(), Some("@alice:matrix.org"));
        assert_eq!(parsed.scope, Some(GlobalSearchScope::GroupsOnly));
        assert_eq!(
            parsed.date_after,
            Some(NaiveDate::from_ymd_opt(2026, 1, 1).unwrap())
        );
        assert_eq!(parsed.sanitized_query, r#""quarterly report" foo\:bar"#);
    }

    #[test]
    fn test_to_tantivy_query() {
        let parsed = parse_search_query("after:2026-01-01 @alice:matrix.org hello world");
        let tantivy = parsed.to_tantivy_query(None);
        assert_eq!(
            tantivy,
            r#"sender:"@alice:matrix.org" AND date:[2026-01-01T00:00:00Z TO *] AND (hello world)"#
        );

        let parsed_plain = parse_search_query("hello world");
        assert_eq!(parsed_plain.to_tantivy_query(None), "hello world");

        let parsed_sender_only = parse_search_query("@alice:matrix.org");
        assert_eq!(
            parsed_sender_only.to_tantivy_query(None),
            r#"sender:"@alice:matrix.org""#
        );

        let parsed_with_resolved = parse_search_query("@alice hello");
        assert_eq!(
            parsed_with_resolved.to_tantivy_query(Some("@alice:server.org")),
            r#"sender:"@alice:server.org" AND (hello)"#
        );
    }

    #[test]
    fn test_tantivy_sanitization() {
        let parsed =
            parse_search_query("https://matrix.org (bracket) [square] {curly} ^5 ~2 !urgent");
        assert_eq!(
            parsed.sanitized_query,
            r#"https\://matrix.org \(bracket\) \[square\] \{curly\} \^5 \~2 \!urgent"#
        );

        let parsed_ops = parse_search_query("cat AND dog OR bird NOT fish");
        assert_eq!(parsed_ops.sanitized_query, "cat AND dog OR bird NOT fish");
    }

    #[test]
    fn test_autocomplete_triggers() {
        assert_eq!(
            extract_autocomplete_trigger("#"),
            Some(AutocompleteTrigger::Room { needle: "" })
        );
        assert_eq!(
            extract_autocomplete_trigger("#gen"),
            Some(AutocompleteTrigger::Room { needle: "gen" })
        );
        assert_eq!(
            extract_autocomplete_trigger("hello #"),
            Some(AutocompleteTrigger::Room { needle: "" })
        );
        assert_eq!(
            extract_autocomplete_trigger("hello #gen"),
            Some(AutocompleteTrigger::Room { needle: "gen" })
        );
        assert_eq!(
            extract_autocomplete_trigger("in:gen"),
            Some(AutocompleteTrigger::Room { needle: "gen" })
        );
        assert_eq!(
            extract_autocomplete_trigger(r#"#"General"#),
            Some(AutocompleteTrigger::Room { needle: "General" })
        );
        assert_eq!(
            extract_autocomplete_trigger(r#"in:"General"#),
            Some(AutocompleteTrigger::Room { needle: "General" })
        );

        assert_eq!(
            extract_autocomplete_trigger("@"),
            Some(AutocompleteTrigger::Member { needle: "" })
        );
        assert_eq!(
            extract_autocomplete_trigger("@ali"),
            Some(AutocompleteTrigger::Member { needle: "ali" })
        );
        assert_eq!(
            extract_autocomplete_trigger("from:ali"),
            Some(AutocompleteTrigger::Member { needle: "ali" })
        );
        assert_eq!(
            extract_autocomplete_trigger(r#"@"Alice"#),
            Some(AutocompleteTrigger::Member { needle: "Alice" })
        );
        assert_eq!(
            extract_autocomplete_trigger(r#"from:"Alice"#),
            Some(AutocompleteTrigger::Member { needle: "Alice" })
        );

        assert_eq!(extract_autocomplete_trigger("#general "), None);
        assert_eq!(extract_autocomplete_trigger("hello "), None);
        assert_eq!(extract_autocomplete_trigger(""), None);
        assert_eq!(extract_autocomplete_trigger("C#"), None);
        assert_eq!(extract_autocomplete_trigger("alice@matrix.org"), None);
    }

    #[test]
    fn test_autocomplete_replacement() {
        assert_eq!(
            apply_autocomplete_replacement("#", "#General "),
            "#General "
        );
        assert_eq!(
            apply_autocomplete_replacement("hello #gen", "#General "),
            "hello #General "
        );
        assert_eq!(
            apply_autocomplete_replacement(r#"#"Gen"#, r#"#"General Chat" "#),
            r#"#"General Chat" "#
        );
        assert_eq!(
            apply_autocomplete_replacement("hello @al", "@alice:matrix.org "),
            "hello @alice:matrix.org "
        );
        assert_eq!(
            apply_autocomplete_replacement("from:al", "from:@alice:matrix.org "),
            "from:@alice:matrix.org "
        );
    }

    #[hegel::test]
    fn prop_parse_search_query_never_panics(tc: hegel::TestCase) {
        let input: String = tc.draw(hegel::generators::text());
        let parsed = parse_search_query(&input);
        let _ = parsed.is_empty();
        let tantivy = parsed.to_tantivy_query(None);
        let _ = tantivy.is_empty();
    }

    #[hegel::test]
    fn prop_extract_autocomplete_trigger_never_panics(tc: hegel::TestCase) {
        let input: String = tc.draw(hegel::generators::text());
        let _ = extract_autocomplete_trigger(&input);
    }

    #[hegel::test]
    fn prop_apply_autocomplete_replacement_never_panics(tc: hegel::TestCase) {
        let current: String = tc.draw(hegel::generators::text());
        let replacement: String = tc.draw(hegel::generators::text());
        let _ = apply_autocomplete_replacement(&current, &replacement);
    }
}
