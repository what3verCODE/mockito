//! URL pattern matching with path parameters.

use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UrlMatchResult {
    pub matched: bool,
    pub params: HashMap<String, String>,
}

pub fn url_matches(pattern: &str, url: &str) -> UrlMatchResult {
    let pattern = normalize_url(pattern);
    let url = normalize_url(url);

    let (regex, param_names) = pattern_to_regex(&pattern);

    let Some(caps) = regex.captures(&url) else {
        return UrlMatchResult::default();
    };

    let params = param_names
        .into_iter()
        .enumerate()
        .filter_map(|(i, name)| caps.get(i + 1).map(|m| (name, m.as_str().to_owned())))
        .collect();

    UrlMatchResult {
        matched: true,
        params,
    }
}

fn normalize_url(url: &str) -> String {
    let without_query = url.split('?').next().unwrap_or("");
    let trimmed = without_query.trim_end_matches('/');
    if trimmed.is_empty() {
        "/".into()
    } else {
        trimmed.into()
    }
}

fn pattern_to_regex(pattern: &str) -> (Regex, Vec<String>) {
    let mut param_names = Vec::new();
    let mut regex_str = String::new();
    let mut chars = pattern.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '{' {
            let name: String = chars.by_ref().take_while(|&c| c != '}').collect();
            param_names.push(name);
            regex_str.push_str("([^/]+)");
        } else if matches!(
            c,
            '.' | '*' | '+' | '?' | '^' | '$' | '(' | ')' | '[' | ']' | '|' | '\\'
        ) {
            regex_str.push('\\');
            regex_str.push(c);
        } else {
            regex_str.push(c);
        }
    }

    let regex = Regex::new(&format!("^{regex_str}/?$")).expect("valid regex");
    (regex, param_names)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case("/api/users", "/api/users", true, &[])]
    #[case("/api/users", "/api/users/", true, &[])]
    #[case("/api/users/{id}", "/api/users/123", true, &[("id", "123")])]
    #[case("/api/users/{id}", "/api/users/abc-123", true, &[("id", "abc-123")])]
    #[case("/api/users/{a}/posts/{b}", "/api/users/1/posts/2", true, &[("a", "1"), ("b", "2")])]
    #[case("/api/users", "/api/posts", false, &[])]
    #[case("/api/users/{id}", "/api/users", false, &[])]
    #[case("/api/users/{id}", "/api/users/123/extra", false, &[])]
    #[case("/", "/", true, &[])]
    #[case("/api/users", "/api/users?page=1", true, &[])]
    #[case("/api/users.json", "/api/users.json", true, &[])]
    #[case("/api/users.json", "/api/usersXjson", false, &[])]
    fn test_url_matches(
        #[case] pattern: &str,
        #[case] url: &str,
        #[case] expected: bool,
        #[case] params: &[(&str, &str)],
    ) {
        let result = url_matches(pattern, url);
        assert_eq!(result.matched, expected);
        for (k, v) in params {
            assert_eq!(result.params.get(*k), Some(&(*v).to_owned()));
        }
    }
}
