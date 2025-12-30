//! Query parameters matching with HashMap intersection and JMESPath expressions.

use crate::expression::match_with_jmespath;
use crate::matching::intersection::{hashmap_intersects, hashmap_to_value};
use std::collections::HashMap;

/// Parse query string into HashMap with support for multiple values per key.
/// URL-decodes both keys and values.
/// For multiple values, stores them as comma-separated string or array.
pub fn parse_query_string(query_str: &str) -> HashMap<String, String> {
    let mut result = HashMap::new();

    if query_str.is_empty() {
        return result;
    }

    for pair in query_str.split('&') {
        if pair.is_empty() {
            continue;
        }

        let parts: Vec<&str> = pair.splitn(2, '=').collect();
        let key = urlencoding::decode(parts[0])
            .unwrap_or_else(|_| parts[0].into())
            .to_string();
        let value = if parts.len() > 1 {
            urlencoding::decode(parts[1])
                .unwrap_or_else(|_| parts[1].into())
                .to_string()
        } else {
            String::new()
        };

        // Handle multiple values for the same key
        if let Some(existing) = result.get_mut(&key) {
            existing.push(',');
            existing.push_str(&value);
        } else {
            result.insert(key, value);
        }
    }

    result
}

/// Match query parameters using JMESPath expression.
/// Converts query HashMap to JSON and evaluates expression.
fn match_query_with_expression(expression: &str, query_params: &HashMap<String, String>) -> bool {
    let query_json = hashmap_to_value(query_params);
    match_with_jmespath(expression, &query_json)
}

/// Match query parameters using either HashMap intersection or JMESPath expression.
/// If query_expr is provided, use JMESPath. Otherwise, use hashmap_intersects.
pub fn query_matches(
    expected: Option<&HashMap<String, String>>,
    query_expr: Option<&str>,
    actual: &HashMap<String, String>,
) -> bool {
    // If expression is provided, use JMESPath
    if let Some(expr) = query_expr {
        return match_query_with_expression(expr, actual);
    }

    // Otherwise, use HashMap intersection
    hashmap_intersects(expected, Some(actual))
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    fn h(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[rstest]
    #[case("", &[])]
    #[case("page=1", &[("page", "1")])]
    #[case("page=1&limit=10", &[("page", "1"), ("limit", "10")])]
    #[case("page=1&limit=10&sort=name", &[("page", "1"), ("limit", "10"), ("sort", "name")])]
    #[case("key=value%20with%20spaces", &[("key", "value with spaces")])]
    #[case("key%20name=value", &[("key name", "value")])]
    #[case("page=1&page=2", &[("page", "1,2")])]
    // Test empty pair (should be skipped)
    #[case("page=1&&limit=10", &[("page", "1"), ("limit", "10")])]
    #[case("&page=1&limit=10", &[("page", "1"), ("limit", "10")])]
    #[case("page=1&limit=10&", &[("page", "1"), ("limit", "10")])]
    // Test key without value
    #[case("page=&limit=10", &[("page", ""), ("limit", "10")])]
    #[case("page&limit=10", &[("page", ""), ("limit", "10")])]
    fn test_parse_query_string(#[case] query_str: &str, #[case] expected: &[(&str, &str)]) {
        let result = parse_query_string(query_str);
        let expected_map = h(expected);
        assert_eq!(result, expected_map);
    }

    #[rstest]
    #[case("page == '1'", true)]
    #[case("page == '2'", false)]
    #[case("page == '1' && limit == '10'", true)]
    #[case("page == '1' && limit == '20'", false)]
    #[case("page != null && limit != null", true)]
    #[case("page != null && limit != null && sort != null", false)]
    fn test_match_query_with_expression_simple(#[case] expression: &str, #[case] expected: bool) {
        let query = h(&[("page", "1"), ("limit", "10")]);
        assert_eq!(match_query_with_expression(expression, &query), expected);
    }

    #[rstest]
    #[case("to_number(page) > `0`", true)]
    #[case("to_number(page) > `5`", false)]
    #[case("to_number(page) > `0` && to_number(limit) <= `100`", true)]
    #[case("to_number(page) > `0` && to_number(limit) <= `5`", false)]
    fn test_match_query_with_expression_numeric(#[case] expression: &str, #[case] expected: bool) {
        let query = h(&[("page", "1"), ("limit", "10")]);
        assert_eq!(match_query_with_expression(expression, &query), expected);
    }

    #[rstest]
    #[case("contains(tags, 'important')", true)]
    #[case("contains(tags, 'unimportant')", false)]
    #[case("tags[0] == 'important'", true)]
    fn test_match_query_with_expression_array(#[case] expression: &str, #[case] expected: bool) {
        let mut query = HashMap::new();
        query.insert("tags".to_string(), "important,urgent,normal".to_string());
        assert_eq!(match_query_with_expression(expression, &query), expected);
    }

    #[rstest]
    fn test_query_matches_hashmap() {
        let expected = h(&[("page", "1")]);
        let actual = h(&[("page", "1"), ("limit", "10")]);
        assert!(query_matches(Some(&expected), None, &actual));
    }

    #[rstest]
    fn test_query_matches_expression() {
        let actual = h(&[("page", "1"), ("limit", "10")]);
        assert!(query_matches(
            None,
            Some("page == '1' && limit == '10'"),
            &actual
        ));
    }

    #[rstest]
    fn test_query_matches_no_expected() {
        let actual = h(&[("page", "1")]);
        assert!(query_matches(None, None, &actual));
    }
}
