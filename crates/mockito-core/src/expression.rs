//! JMESPath expression utilities for matching and response processing.

use jmespath::Variable;
use serde_json::Value;
use std::rc::Rc;

/// Convert serde_json::Value to jmespath::Variable.
pub fn value_to_variable(value: &Value) -> Rc<Variable> {
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

/// Convert jmespath::Variable to serde_json::Value.
pub fn variable_to_value(var: &Rc<Variable>) -> Result<Value, String> {
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

/// Convert JMESPath expression result to boolean.
pub fn jmespath_result_to_bool(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => n.as_f64().map(|f| f != 0.0).unwrap_or(false),
        Value::String(s) => !s.is_empty(),
        Value::Array(a) => !a.is_empty(),
        Value::Object(o) => !o.is_empty(),
    }
}

/// Match data using JMESPath expression.
pub fn match_with_jmespath(expression: &str, data: &Value) -> bool {
    // Parse JMESPath expression
    let expr = match jmespath::compile(expression) {
        Ok(expr) => expr,
        Err(_) => return false, // Invalid expression = no match
    };

    // Convert data to jmespath Variable
    let data_var = value_to_variable(data);

    // Execute expression on data
    let result = match expr.search(&data_var) {
        Ok(result) => result,
        Err(_) => return false, // Execution error = no match
    };

    // Convert jmespath Variable to serde_json::Value
    let value = match variable_to_value(&result) {
        Ok(v) => v,
        Err(_) => return false, // Conversion error = no match
    };

    // Convert result to boolean
    jmespath_result_to_bool(&value)
}

/// Evaluate JMESPath expression on data and return the result as Value.
pub fn evaluate_jmespath(expression: &str, data: &Value) -> Option<Value> {
    // Parse JMESPath expression
    let expr = match jmespath::compile(expression) {
        Ok(expr) => expr,
        Err(_) => return None, // Invalid expression
    };

    // Convert data to jmespath Variable
    let data_var = value_to_variable(data);

    // Execute expression on data
    let result = match expr.search(&data_var) {
        Ok(result) => result,
        Err(_) => return None, // Execution error
    };

    // Convert jmespath Variable to serde_json::Value
    variable_to_value(&result).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use serde_json::json;

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

    #[rstest]
    #[case(Value::Bool(true), true)]
    #[case(Value::Bool(false), false)]
    #[case(Value::Null, false)]
    #[case(json!(0), false)]
    #[case(json!(1), true)]
    #[case(json!(-1), true)]
    #[case(json!(""), false)]
    #[case(json!("test"), true)]
    #[case(json!([]), false)]
    #[case(json!([1, 2]), true)]
    #[case(json!({}), false)]
    #[case(json!({"key": "value"}), true)]
    fn test_jmespath_result_to_bool(#[case] value: Value, #[case] expected: bool) {
        assert_eq!(jmespath_result_to_bool(&value), expected);
    }

    #[rstest]
    #[case("value > `3`", true)]
    #[case("value > `10`", false)]
    fn test_match_with_jmespath(#[case] expression: &str, #[case] expected: bool) {
        let data = json!({"value": 5});
        assert_eq!(match_with_jmespath(expression, &data), expected);
    }

    #[rstest]
    #[case("value", Some(json!(5)))]
    #[case("items[0].id", Some(json!(1)))]
    #[case("items[*].id", Some(json!([1, 2, 3])))]
    #[case("nonexistent", Some(json!(null)))]
    #[case("[invalid", None)]
    fn test_evaluate_jmespath(#[case] expression: &str, #[case] expected: Option<Value>) {
        let data = json!({
            "value": 5,
            "items": [
                {"id": 1},
                {"id": 2},
                {"id": 3}
            ]
        });
        assert_eq!(evaluate_jmespath(expression, &data), expected);
    }
}
