//! Request payload (JSON) matching with object intersection and JMESPath expressions.

use jmespath::Variable;
use serde_json::Value;
use std::collections::HashMap;
use std::rc::Rc;

/// Check if subset JSON object is contained in target JSON object.
/// Supports deep comparison of nested objects, arrays, and primitive types.
/// Returns true if subset is None, Null, or empty object (matches any target).
pub fn object_intersects(target: Option<&Value>, subset: Option<&Value>) -> bool {
    let subset = match subset {
        None | Some(Value::Null) => return true,
        Some(Value::Object(o)) if o.is_empty() => return true,
        Some(s) => s,
    };

    let target = match target {
        None | Some(Value::Null) => return false,
        Some(t) => t,
    };

    value_intersects(target, subset)
}

fn value_intersects(target: &Value, subset: &Value) -> bool {
    match (target, subset) {
        (Value::Object(t), Value::Object(s)) => s
            .iter()
            .all(|(k, sv)| t.get(k).is_some_and(|tv| value_intersects(tv, sv))),
        (Value::Array(t), Value::Array(s)) => s
            .iter()
            .all(|sv| t.iter().any(|tv| value_intersects(tv, sv))),
        _ => target == subset,
    }
}

/// Convert serde_json::Value to jmespath::Variable
fn value_to_variable(value: &Value) -> Rc<Variable> {
    match value {
        Value::Null => Rc::new(Variable::Null),
        Value::Bool(b) => Rc::new(Variable::Bool(*b)),
        Value::Number(n) => Rc::new(Variable::Number(n.clone())),
        Value::String(s) => Rc::new(Variable::String(s.clone())),
        Value::Array(arr) => {
            let vars: Vec<Rc<Variable>> = arr.iter().map(value_to_variable).collect();
            Rc::new(Variable::Array(vars))
        }
        Value::Object(obj) => {
            let map: std::collections::BTreeMap<String, Rc<Variable>> = obj
                .iter()
                .map(|(k, v)| (k.clone(), value_to_variable(v)))
                .collect();
            Rc::new(Variable::Object(map))
        }
    }
}

/// Convert jmespath::Variable to serde_json::Value
fn variable_to_value(var: &Rc<Variable>) -> Result<Value, String> {
    match var.as_ref() {
        Variable::Null => Ok(Value::Null),
        Variable::Bool(b) => Ok(Value::Bool(*b)),
        Variable::Number(n) => Ok(Value::Number(n.clone())),
        Variable::String(s) => Ok(Value::String(s.clone())),
        Variable::Array(arr) => {
            let values: Result<Vec<Value>, String> = arr.iter().map(variable_to_value).collect();
            Ok(Value::Array(values?))
        }
        Variable::Object(obj) => {
            let map: Result<serde_json::Map<String, Value>, String> = obj
                .iter()
                .map(|(k, v)| variable_to_value(v).map(|val| (k.clone(), val)))
                .collect();
            Ok(Value::Object(map?))
        }
        Variable::Expref(_) => Err("Expression references not supported".to_string()),
    }
}

/// Match request payload using JMESPath expression.
/// Returns true if expression evaluates to a truthy value.
fn match_payload_with_expression(expression: &str, body: &Value) -> bool {
    // Parse JMESPath expression
    let expr = match jmespath::compile(expression) {
        Ok(expr) => expr,
        Err(_) => return false, // Invalid expression = no match
    };

    // Convert body to jmespath Variable
    let body_var = value_to_variable(body);

    // Execute expression on body
    let result = match expr.search(&body_var) {
        Ok(result) => result,
        Err(_) => return false, // Execution error = no match
    };

    // Convert jmespath Variable to serde_json::Value
    let value = match variable_to_value(&result) {
        Ok(v) => v,
        Err(_) => return false, // Conversion error = no match
    };

    // Convert result to boolean
    match value {
        Value::Bool(b) => b,
        Value::Null => false,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Match request payload using either object intersection or JMESPath expression.
/// If payload_expr is provided, use JMESPath. Otherwise, use object_intersects.
pub fn payload_matches(
    payload: Option<&HashMap<String, Value>>,
    payload_expr: Option<&str>,
    request_payload: &Value,
) -> bool {
    // If expression is provided, use JMESPath
    if let Some(expr) = payload_expr {
        return match_payload_with_expression(expr, request_payload);
    }

    // Otherwise, use object intersection
    if let Some(expected) = payload {
        let expected_value = serde_json::to_value(expected).unwrap_or(Value::Null);
        return object_intersects(Some(request_payload), Some(&expected_value));
    }

    // No payload specified = match any request_payload
    true
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

    #[rstest]
    #[case(Some(&json!({"a": 1})), None, true)]
    #[case(Some(&json!({"a": 1})), Some(&Value::Null), true)]
    #[case(Some(&json!({"a": 1})), Some(&json!({})), true)]
    #[case(None, Some(&json!({"a": 1})), false)]
    #[case(Some(&Value::Null), Some(&json!({"a": 1})), false)]
    #[case(Some(&json!({"a": 1, "b": 2})), Some(&json!({"a": 1})), true)]
    #[case(Some(&json!({"a": 1})), Some(&json!({"a": 2})), false)]
    #[case(Some(&json!({"a": 1})), Some(&json!({"b": 1})), false)]
    #[case(Some(&json!({"user": {"name": "John", "age": 30}})), Some(&json!({"user": {"name": "John"}})), true)]
    #[case(Some(&json!({"user": {"name": "John"}})), Some(&json!({"user": {"name": "Jane"}})), false)]
    #[case(Some(&json!({"items": [{"id": 1}, {"id": 2}]})), Some(&json!({"items": [{"id": 1}]})), true)]
    #[case(Some(&json!({"items": [{"id": 1}]})), Some(&json!({"items": [{"id": 4}]})), false)]
    #[case(Some(&json!(1)), Some(&json!(1)), true)]
    #[case(Some(&json!("test")), Some(&json!("test")), true)]
    #[case(Some(&json!(1)), Some(&json!(2)), false)]
    fn test_object_intersects(
        #[case] target: Option<&Value>,
        #[case] subset: Option<&Value>,
        #[case] expected: bool,
    ) {
        assert_eq!(object_intersects(target, subset), expected);
    }

    #[rstest]
    #[case("items[2].id == `5`", true)]
    #[case("items[2].id == `10`", false)]
    fn test_match_payload_with_expression_array_position(
        #[case] expression: &str,
        #[case] expected: bool,
    ) {
        let body = json!({"items": [{"id": 1}, {"id": 2}, {"id": 5}]});
        assert_eq!(match_payload_with_expression(expression, &body), expected);
    }

    #[rstest]
    #[case("contains(items[*].id, `5`)", true)]
    #[case("contains(items[*].id, `10`)", false)]
    fn test_match_payload_with_expression_array_contains(
        #[case] expression: &str,
        #[case] expected: bool,
    ) {
        let body = json!({"items": [{"id": 1}, {"id": 5}, {"id": 3}]});
        assert_eq!(match_payload_with_expression(expression, &body), expected);
    }

    #[rstest]
    fn test_match_payload_with_expression_complex_query() {
        let body = json!({
            "users": [
                {"name": "John", "age": 20},
                {"name": "Jane", "age": 15}
            ]
        });
        assert!(match_payload_with_expression(
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
        assert_eq!(match_payload_with_expression(expression, &body), expected);
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
        assert!(!match_payload_with_expression("[invalid", &body));
    }

    #[rstest]
    fn test_match_payload_with_expression_null_result() {
        let body = json!({"value": null});
        // Expression that returns null
        assert!(!match_payload_with_expression("value", &body));
    }

    #[rstest]
    fn test_match_payload_with_expression_number_result() {
        let body = json!({"value": 5});
        // Expression that returns number
        assert!(match_payload_with_expression("value", &body));
        assert!(!match_payload_with_expression("value - value", &body)); // Returns 0
    }

    #[rstest]
    fn test_match_payload_with_expression_string_result() {
        let body = json!({"value": "test"});
        // Expression that returns string
        assert!(match_payload_with_expression("value", &body));

        let body_empty = json!({"value": ""});
        assert!(!match_payload_with_expression("value", &body_empty));
    }

    #[rstest]
    fn test_match_payload_with_expression_array_result() {
        let body = json!({"items": [1, 2, 3]});
        // Expression that returns array
        assert!(match_payload_with_expression("items", &body));

        let body_empty = json!({"items": []});
        assert!(!match_payload_with_expression("items", &body_empty));
    }

    #[rstest]
    fn test_match_payload_with_expression_object_result() {
        let body = json!({"user": {"name": "John"}});
        // Expression that returns object
        assert!(match_payload_with_expression("user", &body));

        let body_empty = json!({"user": {}});
        assert!(!match_payload_with_expression("user", &body_empty));
    }

    #[rstest]
    fn test_value_to_variable_null() {
        let value = json!(null);
        let var = value_to_variable(&value);
        assert!(matches!(*var, Variable::Null));
    }

    #[rstest]
    fn test_value_to_variable_bool() {
        let value = json!(true);
        let var = value_to_variable(&value);
        assert!(matches!(*var, Variable::Bool(true)));
    }

    #[rstest]
    fn test_variable_to_value_all_types() {
        use jmespath::Variable;

        // Test Null
        let var_null = Rc::new(Variable::Null);
        assert_eq!(variable_to_value(&var_null).unwrap(), json!(null));

        // Test Bool
        let var_bool = Rc::new(Variable::Bool(true));
        assert_eq!(variable_to_value(&var_bool).unwrap(), json!(true));

        // Test Number
        let var_num = Rc::new(Variable::Number(serde_json::Number::from(123)));
        assert_eq!(variable_to_value(&var_num).unwrap(), json!(123));

        // Test String
        let var_str = Rc::new(Variable::String("test".to_string()));
        assert_eq!(variable_to_value(&var_str).unwrap(), json!("test"));

        // Test Array
        let var_arr = Rc::new(Variable::Array(vec![
            Rc::new(Variable::Number(serde_json::Number::from(1))),
            Rc::new(Variable::Number(serde_json::Number::from(2))),
        ]));
        assert_eq!(variable_to_value(&var_arr).unwrap(), json!([1, 2]));

        // Test Object
        let mut map = std::collections::BTreeMap::new();
        map.insert(
            "key".to_string(),
            Rc::new(Variable::String("value".to_string())),
        );
        let var_obj = Rc::new(Variable::Object(map));
        assert_eq!(
            variable_to_value(&var_obj).unwrap(),
            json!({"key": "value"})
        );
    }
}
