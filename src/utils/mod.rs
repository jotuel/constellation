use url::Url;

// ⚡ Bolt Optimization: Fast path for ASCII string filtering
// Avoids costly heap allocations from `.to_lowercase()`
pub fn contains_ignore_ascii_case(
    haystack: &str,
    query: &str,
    query_lower_fallback: Option<&str>,
) -> bool {
    if query.is_empty() {
        return true;
    }

    if query.is_ascii() {
        let query_bytes = query.as_bytes();
        let query_len = query_bytes.len();
        let h_bytes = haystack.as_bytes();

        if h_bytes.len() < query_len {
            return false;
        }

        h_bytes
            .windows(query_len)
            .any(|window| window.eq_ignore_ascii_case(query_bytes))
    } else if let Some(query_lower) = query_lower_fallback {
        haystack.to_lowercase().contains(query_lower)
    } else {
        haystack.to_lowercase().contains(&query.to_lowercase())
    }
}

pub fn fuzzy_match_ignore_case(haystack: &str, query: &str) -> bool {
    if query.is_empty() {
        return true;
    }
    let mut query_chars = query.chars().peekable();
    for h_char in haystack.chars() {
        if let Some(&q_char) = query_chars.peek() {
            // ⚡ Bolt Optimization: Compare iterators directly with `.eq()`
            // to avoid O(N) `.to_string()` heap allocations per character match.
            if h_char.to_lowercase().eq(q_char.to_lowercase()) {
                query_chars.next();
            }
        } else {
            return true;
        }
    }
    query_chars.peek().is_none()
}

/// Redacts the values of sensitive query params and the entire URL fragment,
/// intended for safely logging URLs.
pub fn redact_url(url: &Url) -> String {
    let mut redacted = url.clone();
    if redacted.password().is_some() {
        let _ = redacted.set_password(Some("***"));
    }
    if redacted.fragment().is_some() {
        redacted.set_fragment(Some("REDACTED"));
    }
    let pairs: Vec<(String, String)> = redacted
        .query_pairs()
        .map(|(k, v)| (k.into_owned(), v.into_owned()))
        .collect();
    redacted.set_query(None);
    for (k, mut v) in pairs {
        if k == "code" || k == "state" || k == "access_token" || k == "login_token" {
            v = "[REDACTED]".to_string();
        }
        redacted.query_pairs_mut().append_pair(&k, &v);
    }
    redacted.to_string()
}

pub trait ApplyVectorDiffExt<T> {
    fn apply_diff(&mut self, diff: eyeball_im::VectorDiff<T>);
}

pub trait VectorOperations<T> {
    fn v_len(&self) -> usize;
    fn v_insert(&mut self, index: usize, value: T);
    fn v_remove(&mut self, index: usize);
    fn v_set(&mut self, index: usize, value: T);
    fn v_push_back(&mut self, value: T);
    fn v_push_front(&mut self, value: T);
    fn v_pop_back(&mut self);
    fn v_pop_front(&mut self);
    fn v_clear(&mut self);
    fn v_reset(&mut self, values: eyeball_im::Vector<T>);
    fn v_extend(&mut self, values: eyeball_im::Vector<T>);
    fn v_truncate(&mut self, length: usize);
}

impl<T: Clone> VectorOperations<T> for Vec<T> {
    fn v_len(&self) -> usize {
        self.len()
    }
    fn v_insert(&mut self, index: usize, value: T) {
        self.insert(index, value);
    }
    fn v_remove(&mut self, index: usize) {
        self.remove(index);
    }
    fn v_set(&mut self, index: usize, value: T) {
        self[index] = value;
    }
    fn v_push_back(&mut self, value: T) {
        self.push(value);
    }
    fn v_push_front(&mut self, value: T) {
        self.insert(0, value);
    }
    fn v_pop_back(&mut self) {
        self.pop();
    }
    fn v_pop_front(&mut self) {
        if !self.is_empty() {
            self.remove(0);
        }
    }
    fn v_clear(&mut self) {
        self.clear();
    }
    fn v_reset(&mut self, values: eyeball_im::Vector<T>) {
        *self = values.into_iter().collect();
    }
    fn v_extend(&mut self, values: eyeball_im::Vector<T>) {
        self.extend(values);
    }
    fn v_truncate(&mut self, length: usize) {
        self.truncate(length);
    }
}

impl<T: Clone> VectorOperations<T> for eyeball_im::Vector<T> {
    fn v_len(&self) -> usize {
        self.len()
    }
    fn v_insert(&mut self, index: usize, value: T) {
        self.insert(index, value);
    }
    fn v_remove(&mut self, index: usize) {
        self.remove(index);
    }
    fn v_set(&mut self, index: usize, value: T) {
        self.set(index, value);
    }
    fn v_push_back(&mut self, value: T) {
        self.push_back(value);
    }
    fn v_push_front(&mut self, value: T) {
        self.push_front(value);
    }
    fn v_pop_back(&mut self) {
        self.pop_back();
    }
    fn v_pop_front(&mut self) {
        self.pop_front();
    }
    fn v_clear(&mut self) {
        self.clear();
    }
    fn v_reset(&mut self, values: eyeball_im::Vector<T>) {
        *self = values;
    }
    fn v_extend(&mut self, values: eyeball_im::Vector<T>) {
        self.extend(values);
    }
    fn v_truncate(&mut self, length: usize) {
        self.truncate(length);
    }
}

impl<C: VectorOperations<T>, T: Clone> ApplyVectorDiffExt<T> for C {
    fn apply_diff(&mut self, diff: eyeball_im::VectorDiff<T>) {
        match diff {
            eyeball_im::VectorDiff::Insert { index, value } => {
                if index <= self.v_len() {
                    self.v_insert(index, value);
                } else {
                    self.v_push_back(value);
                }
            }
            eyeball_im::VectorDiff::Remove { index } => {
                if index < self.v_len() {
                    self.v_remove(index);
                }
            }
            eyeball_im::VectorDiff::Set { index, value } => {
                if index < self.v_len() {
                    self.v_set(index, value);
                }
            }
            eyeball_im::VectorDiff::Reset { values } => {
                self.v_reset(values);
            }
            eyeball_im::VectorDiff::PushBack { value } => {
                self.v_push_back(value);
            }
            eyeball_im::VectorDiff::PushFront { value } => {
                self.v_push_front(value);
            }
            eyeball_im::VectorDiff::PopBack => {
                self.v_pop_back();
            }
            eyeball_im::VectorDiff::PopFront => {
                self.v_pop_front();
            }
            eyeball_im::VectorDiff::Clear => {
                self.v_clear();
            }
            eyeball_im::VectorDiff::Append { values } => {
                self.v_extend(values);
            }
            eyeball_im::VectorDiff::Truncate { length } => {
                self.v_truncate(length);
            }
        }
    }
}

pub mod i18n;
pub(crate) mod ipc;
pub mod item;
pub mod og;
pub mod permalink;
pub mod preview;
pub mod rich_text;
pub mod search_query;
pub mod unified_push;
pub mod widget;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contains_ignore_ascii_case_empty() {
        assert!(contains_ignore_ascii_case("anything", "", None));
        assert!(contains_ignore_ascii_case("", "", None));
        assert!(!contains_ignore_ascii_case("", "a", None));
    }

    #[test]
    fn test_contains_ignore_ascii_case_fast_path_length() {
        // Haystack shorter than query
        assert!(!contains_ignore_ascii_case("a", "ab", None));
        assert!(!contains_ignore_ascii_case("foo", "foobar", None));
    }

    #[test]
    fn test_contains_ignore_ascii_case_ascii_bounds() {
        assert!(contains_ignore_ascii_case("Hello World", "WORLD", None));
        assert!(contains_ignore_ascii_case("Hello World", "he", None));
        assert!(contains_ignore_ascii_case("Hello World", "Lo W", None));
        assert!(!contains_ignore_ascii_case("Hello World", "foo", None));
    }

    #[test]
    fn test_contains_ignore_ascii_case_utf8_boundaries() {
        // Multi-byte haystack
        assert!(contains_ignore_ascii_case("héllo wörld", "lo w", None));
        assert!(contains_ignore_ascii_case("Emoji 🚀 Test", "test", None));
        assert!(!contains_ignore_ascii_case("Emoji 🚀 Test", "foo", None));

        // Ensure multi-byte haystack boundaries don't accidentally match ASCII query
        // "İ" (U+0130) is [196, 176] in UTF-8
        // "i" is [105] in ASCII
        assert!(!contains_ignore_ascii_case("İ", "i", None));
    }

    #[test]
    fn test_contains_ignore_ascii_case_unicode_fallback() {
        // Non-ASCII query with fallback
        assert!(contains_ignore_ascii_case(
            "héllo wörld",
            "WÖRLD",
            Some("wörld")
        ));
        assert!(!contains_ignore_ascii_case("héllo wörld", "Ü", Some("ü")));

        // Non-ASCII query without fallback
        assert!(contains_ignore_ascii_case("héllo wörld", "WÖRLD", None));
        assert!(!contains_ignore_ascii_case("héllo wörld", "Ü", None));
    }

    #[test]
    fn test_contains_ignore_ascii_case_unicode_expansion() {
        // "K" (Kelvin sign U+212A) lowercases to "k"
        assert!(!contains_ignore_ascii_case("K", "k", None));
        // "ß" (U+00DF) lowercases to "ss" (expansion)
        assert!(!contains_ignore_ascii_case("ß", "ss", None));
    }

    #[test]
    fn test_fuzzy_match_ignore_case() {
        // Empty query
        assert!(fuzzy_match_ignore_case("anything", ""));
        assert!(fuzzy_match_ignore_case("", ""));

        // Exact match
        assert!(fuzzy_match_ignore_case("HelloWorld", "HelloWorld"));
        assert!(fuzzy_match_ignore_case("HelloWorld", "helloworld"));
        assert!(fuzzy_match_ignore_case("helloworld", "HelloWorld"));

        // Fuzzy match
        assert!(fuzzy_match_ignore_case("Hello World", "hwd"));
        assert!(fuzzy_match_ignore_case("matrix-rust-sdk", "mrs"));
        assert!(fuzzy_match_ignore_case("matrix-rust-sdk", "matrsdk"));

        // Non-match
        assert!(!fuzzy_match_ignore_case("Hello World", "foo"));
        assert!(!fuzzy_match_ignore_case("Hello World", "hdw")); // out of order
        assert!(!fuzzy_match_ignore_case("matrix", "xmatrix")); // missing x at start

        // Non-ASCII and emojis
        assert!(fuzzy_match_ignore_case("héllo wörld", "hlwr"));
        assert!(fuzzy_match_ignore_case("Emoji 🚀 Test", "mojitst"));
        assert!(fuzzy_match_ignore_case("Emoji 🚀 Test", "🚀t"));
        assert!(!fuzzy_match_ignore_case("Emoji 🚀 Test", "🚀x"));

        // Missing edge cases
        assert!(fuzzy_match_ignore_case("hello", "hlo"));
        assert!(fuzzy_match_ignore_case("Test", "t"));
        assert!(fuzzy_match_ignore_case("Emoji 🚀 Test", "🚀"));
        assert!(fuzzy_match_ignore_case("abcd", "bc"));

        assert!(!fuzzy_match_ignore_case("", "a"));
        assert!(!fuzzy_match_ignore_case("a", "b"));
    }

    #[test]
    fn test_redact_url() {
        // query params `code` and `state` redacted to "[REDACTED]", while a normal param (e.g. `room`) is preserved
        let url =
            Url::parse("https://example.com/oauth?code=secret_code&state=secret_state&room=123")
                .unwrap();
        let redacted_str = redact_url(&url);
        let redacted_url =
            Url::parse(&redacted_str).expect("redact_url produces a valid URL string");
        let query_map: std::collections::HashMap<_, _> =
            redacted_url.query_pairs().into_owned().collect();
        assert_eq!(
            query_map.get("code").map(|s| s.as_str()),
            Some("[REDACTED]")
        );
        assert_eq!(
            query_map.get("state").map(|s| s.as_str()),
            Some("[REDACTED]")
        );
        assert_eq!(query_map.get("room").map(|s| s.as_str()), Some("123"));

        // access_token and login_token redacted
        let url = Url::parse(
            "https://example.com/login?access_token=secret_access&login_token=secret_login",
        )
        .unwrap();
        let redacted_str = redact_url(&url);
        let redacted_url =
            Url::parse(&redacted_str).expect("redact_url produces a valid URL string");
        let query_map: std::collections::HashMap<_, _> =
            redacted_url.query_pairs().into_owned().collect();
        assert_eq!(
            query_map.get("access_token").map(|s| s.as_str()),
            Some("[REDACTED]")
        );
        assert_eq!(
            query_map.get("login_token").map(|s| s.as_str()),
            Some("[REDACTED]")
        );

        // fragment redacted: build a URL with a fragment secret and assert the
        // returned string does NOT contain the secret text and DOES contain
        // "#REDACTED". (This fixture is a generic URL-with-fragment; the QR
        // login no longer produces a matrix.to URL — it uses binary MSC4108
        // bytes — but `redact_url` must still redact any URL fragment.)
        let secret_text = "SECRET_RENDEZVOUS_123456";
        let url = Url::parse(&format!(
            "https://matrix.to/#/login?rendezvous={}",
            secret_text
        ))
        .unwrap();
        let redacted_str = redact_url(&url);
        assert!(!redacted_str.contains(secret_text));
        assert!(redacted_str.contains("#REDACTED"));

        // a URL with no query and no fragment round-trips unchanged
        let url = Url::parse("https://example.com/plain/path").unwrap();
        let redacted_str = redact_url(&url);
        assert_eq!(redacted_str, "https://example.com/plain/path");

        // a URL with query params but no sensitive ones round-trips unchanged
        let url = Url::parse("https://example.com/search?q=rust&sort=desc").unwrap();
        let redacted_str = redact_url(&url);
        assert_eq!(redacted_str, "https://example.com/search?q=rust&sort=desc");

        // URL with username and password
        let url = Url::parse("https://user:password123@example.com/path").unwrap();
        let redacted_str = redact_url(&url);
        assert_eq!(redacted_str, "https://user:***@example.com/path");

        // URL with just username
        let url = Url::parse("https://user@example.com/path").unwrap();
        let redacted_str = redact_url(&url);
        assert_eq!(redacted_str, "https://user@example.com/path");
    }

    #[test]
    fn test_apply_diff() {
        let mut vec = vec![1, 2, 3];

        // PushBack
        vec.apply_diff(eyeball_im::VectorDiff::PushBack { value: 4 });
        assert_eq!(vec, vec![1, 2, 3, 4]);

        // PushFront
        vec.apply_diff(eyeball_im::VectorDiff::PushFront { value: 0 });
        assert_eq!(vec, vec![0, 1, 2, 3, 4]);

        // PopBack
        vec.apply_diff(eyeball_im::VectorDiff::PopBack);
        assert_eq!(vec, vec![0, 1, 2, 3]);

        // PopFront
        vec.apply_diff(eyeball_im::VectorDiff::PopFront);
        assert_eq!(vec, vec![1, 2, 3]);

        // Insert
        vec.apply_diff(eyeball_im::VectorDiff::Insert { index: 1, value: 5 });
        assert_eq!(vec, vec![1, 5, 2, 3]);

        // Insert out of bounds (should push back according to apply_diff logic)
        vec.apply_diff(eyeball_im::VectorDiff::Insert {
            index: 10,
            value: 6,
        });
        assert_eq!(vec, vec![1, 5, 2, 3, 6]);

        // Remove
        vec.apply_diff(eyeball_im::VectorDiff::Remove { index: 1 });
        assert_eq!(vec, vec![1, 2, 3, 6]);

        // Remove out of bounds (should do nothing)
        vec.apply_diff(eyeball_im::VectorDiff::Remove { index: 10 });
        assert_eq!(vec, vec![1, 2, 3, 6]);

        // Set
        vec.apply_diff(eyeball_im::VectorDiff::Set { index: 1, value: 7 });
        assert_eq!(vec, vec![1, 7, 3, 6]);

        // Set out of bounds (should do nothing)
        vec.apply_diff(eyeball_im::VectorDiff::Set {
            index: 10,
            value: 8,
        });
        assert_eq!(vec, vec![1, 7, 3, 6]);

        // Clear
        vec.apply_diff(eyeball_im::VectorDiff::Clear);
        assert!(vec.is_empty());

        // Append
        let append_vec: eyeball_im::Vector<i32> = vec![1, 2].into_iter().collect();
        vec.apply_diff(eyeball_im::VectorDiff::Append { values: append_vec });
        assert_eq!(vec, vec![1, 2]);

        // Truncate
        vec.apply_diff(eyeball_im::VectorDiff::Truncate { length: 1 });
        assert_eq!(vec, vec![1]);

        // Reset
        let reset_vec: eyeball_im::Vector<i32> = vec![9, 8, 7].into_iter().collect();
        vec.apply_diff(eyeball_im::VectorDiff::Reset { values: reset_vec });
        assert_eq!(vec, vec![9, 8, 7]);
    }

    // --- Property-based tests ---

    #[hegel::test]
    fn prop_apply_vector_diff_model(tc: hegel::TestCase) {
        let mut vec = Vec::<i64>::new();
        let mut eye = eyeball_im::Vector::<i64>::new();

        let num_ops = tc.draw(
            hegel::generators::integers::<usize>()
                .min_value(1)
                .max_value(50),
        );
        for _ in 0..num_ops {
            let op_type = tc.draw(
                hegel::generators::integers::<u8>()
                    .min_value(0)
                    .max_value(10),
            );
            let val = tc.draw(hegel::generators::integers::<i64>());
            let idx = tc.draw(hegel::generators::integers::<usize>().max_value(vec.len() + 5));

            let diff = match op_type {
                0 => eyeball_im::VectorDiff::Insert {
                    index: idx,
                    value: val,
                },
                1 => eyeball_im::VectorDiff::Remove { index: idx },
                2 => eyeball_im::VectorDiff::Set {
                    index: idx,
                    value: val,
                },
                3 => eyeball_im::VectorDiff::PushBack { value: val },
                4 => eyeball_im::VectorDiff::PushFront { value: val },
                5 => eyeball_im::VectorDiff::PopBack,
                6 => eyeball_im::VectorDiff::PopFront,
                7 => eyeball_im::VectorDiff::Clear,
                8 => {
                    let count = tc.draw(hegel::generators::integers::<usize>().max_value(5));
                    let items: eyeball_im::Vector<i64> = (0..count).map(|_| val).collect();
                    eyeball_im::VectorDiff::Append { values: items }
                }
                9 => {
                    let len =
                        tc.draw(hegel::generators::integers::<usize>().max_value(vec.len() + 2));
                    eyeball_im::VectorDiff::Truncate { length: len }
                }
                _ => {
                    let count = tc.draw(hegel::generators::integers::<usize>().max_value(5));
                    let items: eyeball_im::Vector<i64> =
                        (0..count).map(|i| val.wrapping_add(i as i64)).collect();
                    eyeball_im::VectorDiff::Reset { values: items }
                }
            };

            vec.apply_diff(diff.clone());
            eye.apply_diff(diff);

            let eye_as_vec: Vec<i64> = eye.iter().copied().collect();
            assert_eq!(vec, eye_as_vec, "Vec and eyeball_im::Vector diverged");
            assert_eq!(vec.len(), eye.len());
        }
    }

    #[hegel::test]
    fn prop_redact_url_invariants(tc: hegel::TestCase) {
        let domain = tc.draw(hegel::generators::from_regex(r"[a-z0-9-]{1,10}\.(com|org)"));
        let has_pass = tc.draw(hegel::generators::booleans());
        let has_frag = tc.draw(hegel::generators::booleans());

        let user_info = if has_pass { "user:secret@" } else { "" };
        let mut url_str = format!("https://{user_info}{domain}/oauth/callback");

        let sensitive_keys = ["code", "state", "access_token", "login_token"];
        let num_params = tc.draw(
            hegel::generators::integers::<usize>()
                .min_value(1)
                .max_value(6),
        );
        let mut expected_normal = Vec::new();

        let mut query_pairs = Vec::new();
        for i in 0..num_params {
            let is_sensitive = tc.draw(hegel::generators::booleans());
            if is_sensitive {
                let key = tc.draw(hegel::generators::sampled_from(sensitive_keys.to_vec()));
                let val = tc.draw(hegel::generators::from_regex(r"[a-zA-Z0-9_-]{5,15}"));
                query_pairs.push((key.to_string(), val));
            } else {
                let key = format!("param{i}");
                let val = tc.draw(hegel::generators::from_regex(r"[a-zA-Z0-9_-]{5,15}"));
                expected_normal.push((key.clone(), val.clone()));
                query_pairs.push((key, val));
            }
        }

        let query_str = query_pairs
            .iter()
            .map(|(k, v)| format!("{k}={v}"))
            .collect::<Vec<_>>()
            .join("&");
        url_str.push('?');
        url_str.push_str(&query_str);

        if has_frag {
            url_str.push_str("#secret-fragment");
        }

        let url = Url::parse(&url_str).unwrap();
        let redacted_str = redact_url(&url);
        let parsed_redacted = Url::parse(&redacted_str).expect("redacted URL must be valid URL");

        if has_pass {
            assert_eq!(parsed_redacted.password(), Some("***"));
        }
        if has_frag {
            assert_eq!(parsed_redacted.fragment(), Some("REDACTED"));
        }

        for (k, v) in parsed_redacted.query_pairs() {
            if sensitive_keys.contains(&k.as_ref()) {
                assert_eq!(v, "[REDACTED]");
            }
        }
        for (k, v) in expected_normal {
            let found = parsed_redacted
                .query_pairs()
                .any(|(pk, pv)| pk == k && pv == v);
            assert!(found, "non-sensitive param {k}={v} must be preserved");
        }
    }

    #[hegel::test]
    fn prop_contains_ignore_ascii_case_invariants(tc: hegel::TestCase) {
        let haystack: String = tc.draw(hegel::generators::text());
        let query: String = tc.draw(hegel::generators::text());

        // Never panics with or without fallback
        let _ = contains_ignore_ascii_case(&haystack, &query, None);
        let _ = contains_ignore_ascii_case(&haystack, &query, Some(&query.to_lowercase()));

        // Invariant: any string contains the empty query
        assert!(contains_ignore_ascii_case(&haystack, "", None));

        // Invariant: any string contains itself when lowercased fallback is provided
        let lower = haystack.to_lowercase();
        assert!(contains_ignore_ascii_case(
            &haystack,
            &haystack,
            Some(&lower)
        ));
    }
}
