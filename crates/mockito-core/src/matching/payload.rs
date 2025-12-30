//! Request payload (JSON) matching with object intersection and JMESPath expressions.

use crate::expression::match_with_jmespath;
use crate::matching::intersection::object_intersects;
use serde_json::Value;
use std::collections::HashMap;

/// Match request payload using either object intersection or JMESPath expression.
/// If payload_expr is provided, use JMESPath. Otherwise, use object_intersects.
pub fn payload_matches(
    payload: Option<&HashMap<String, Value>>,
    payload_expr: Option<&str>,
    actual: &Value,
) -> bool {
    // If expression is provided, use JMESPath
    if let Some(expr) = payload_expr {
        return match_with_jmespath(expr, actual);
    }

    // Otherwise, use object intersection
    if let Some(expected) = payload {
        let expected_value = serde_json::to_value(expected).unwrap_or(Value::Null);
        return object_intersects(Some(actual), Some(&expected_value));
    }

    // No payload specified = match any actual
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case("items[2].id == `5`", true)]
    #[case("items[2].id == `10`", false)]
    fn test_match_payload_with_expression_array_position(
        #[case] expression: &str,
        #[case] expected: bool,
    ) {
        let body = json!({"items": [{"id": 1}, {"id": 2}, {"id": 5}]});
        assert_eq!(match_with_jmespath(expression, &body), expected);
    }

    #[rstest]
    #[case("contains(items[*].id, `5`)", true)]
    #[case("contains(items[*].id, `10`)", false)]
    fn test_match_payload_with_expression_array_contains(
        #[case] expression: &str,
        #[case] expected: bool,
    ) {
        let body = json!({"items": [{"id": 1}, {"id": 5}, {"id": 3}]});
        assert_eq!(match_with_jmespath(expression, &body), expected);
    }

    #[rstest]
    fn test_match_payload_with_expression_complex_query() {
        let body = json!({
            "users": [
                {"name": "John", "age": 20},
                {"name": "Jane", "age": 15}
            ]
        });
        assert!(match_with_jmespath(
            "length(users[?age > `18`].name) > `0`",
            &body
        ));
    }

    #[rstest]
    #[case("value > `3`", true)]
    #[case("value > `10`", false)]
    fn test_match_payload_with_expression_boolean_result(
        #[case] expression: &str,
        #[case] expected: bool,
    ) {
        let body = json!({"value": 5});
        assert_eq!(match_with_jmespath(expression, &body), expected);
    }

    #[rstest]
    fn test_payload_matches_object_notation() {
        let body = json!({"userId": 123, "name": "John"});
        let mut payload = HashMap::new();
        payload.insert("userId".to_string(), json!(123));

        assert!(payload_matches(Some(&payload), None, &body));
    }

    #[rstest]
    fn test_payload_matches_expression_notation() {
        let body = json!({"items": [{"id": 5}]});
        assert!(payload_matches(
            None,
            Some("contains(items[*].id, `5`)"),
            &body
        ));
    }

    #[rstest]
    fn test_payload_matches_no_payload() {
        let body = json!({"any": "value"});
        assert!(payload_matches(None, None, &body));
    }

    #[rstest]
    fn test_match_payload_with_expression_invalid_syntax() {
        let body = json!({"value": 5});
        // Invalid JMESPath syntax
        assert!(!match_with_jmespath("[invalid", &body));
    }

    #[rstest]
    fn test_match_payload_with_expression_null_result() {
        let body = json!({"value": null});
        // Expression that returns null
        assert!(!match_with_jmespath("value", &body));
    }

    #[rstest]
    fn test_match_payload_with_expression_number_result() {
        let body = json!({"value": 5});
        // Expression that returns number
        assert!(match_with_jmespath("value", &body));
        assert!(!match_with_jmespath("value - value", &body)); // Returns 0
    }

    #[rstest]
    fn test_match_payload_with_expression_string_result() {
        let body = json!({"value": "test"});
        // Expression that returns string
        assert!(match_with_jmespath("value", &body));

        let body_empty = json!({"value": ""});
        assert!(!match_with_jmespath("value", &body_empty));
    }

    #[rstest]
    fn test_match_payload_with_expression_array_result() {
        let body = json!({"items": [1, 2, 3]});
        // Expression that returns array
        assert!(match_with_jmespath("items", &body));

        let body_empty = json!({"items": []});
        assert!(!match_with_jmespath("items", &body_empty));
    }

    #[rstest]
    fn test_match_payload_with_expression_object_result() {
        let body = json!({"user": {"name": "John"}});
        // Expression that returns object
        assert!(match_with_jmespath("user", &body));

        let body_empty = json!({"user": {}});
        assert!(!match_with_jmespath("user", &body_empty));
    }
}
