//! Object intersection utilities for matching.

use serde_json::Value;
use std::collections::HashMap;

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

/// Check if expected HashMap is contained in actual HashMap.
/// Supports simple key-value matching with support for multiple comma-separated values.
/// Returns true if expected is None or empty (matches any actual).
pub fn hashmap_intersects(
    expected: Option<&HashMap<String, String>>,
    actual: Option<&HashMap<String, String>>,
) -> bool {
    // If expected is None, it means "not specified in config" = don't check = match any
    let expected = match expected {
        None => return true, // Not specified in config = match any actual
        Some(e) if e.is_empty() => return true, // Empty expected = match any
        Some(e) => e,
    };

    let actual = match actual {
        None => return false, // Expected some params but actual is None - no match
        Some(a) => a,
    };

    // Check that all expected keys exist in actual with matching values
    // If any expected key is missing in actual, return false
    expected.iter().all(|(k, v)| {
        match actual.get(k) {
            None => false, // Key missing in actual
            Some(actual_value) => {
                // If expected value contains comma, check if any of the comma-separated values match
                if v.contains(',') {
                    v.split(',')
                        .any(|ev| actual_value.split(',').any(|av| av.trim() == ev.trim()))
                } else if actual_value.contains(',') {
                    // If actual has multiple values, check if expected value is in the list
                    actual_value.split(',').any(|av| av.trim() == v.trim())
                } else {
                    actual_value.trim() == v.trim()
                }
            }
        }
    })
}

/// Convert HashMap<String, String> to JSON Value for intersection matching.
pub fn hashmap_to_value(map: &HashMap<String, String>) -> Value {
    let mut json_map = serde_json::Map::new();
    for (key, value) in map {
        // Check if value contains comma (multiple values)
        if value.contains(',') {
            let array: Vec<Value> = value
                .split(',')
                .map(|v| Value::String(v.trim().to_string()))
                .collect();
            json_map.insert(key.clone(), Value::Array(array));
        } else {
            json_map.insert(key.clone(), Value::String(value.clone()));
        }
    }
    Value::Object(json_map)
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
    fn test_object_intersects(
        #[case] target: Option<&Value>,
        #[case] subset: Option<&Value>,
        #[case] expected: bool,
    ) {
        assert_eq!(object_intersects(target, subset), expected);
    }

    fn h(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[rstest]
    #[case(None, None, true)]
    #[case(Some(&h(&[])), Some(&h(&[])), true)]
    #[case(Some(&h(&[("page", "1")])), None, false)]
    #[case(Some(&h(&[("page", "1")])), Some(&h(&[])), false)]
    #[case(Some(&h(&[("page", "1")])), Some(&h(&[("page", "1"), ("limit", "10")])), true)]
    #[case(Some(&h(&[("page", "1")])), Some(&h(&[("page", "2")])), false)]
    #[case(Some(&h(&[("page", "1")])), Some(&h(&[("limit", "10")])), false)]
    #[case(None, Some(&h(&[("page", "1")])), true)]
    #[case(Some(&h(&[("page", "1"), ("limit", "10")])), Some(&h(&[("page", "1")])), false)]
    // Test comma-separated values: expected contains comma
    #[case(Some(&h(&[("tags", "important,urgent")])), Some(&h(&[("tags", "important")])), true)]
    #[case(Some(&h(&[("tags", "important,urgent")])), Some(&h(&[("tags", "urgent")])), true)]
    #[case(Some(&h(&[("tags", "important,urgent")])), Some(&h(&[("tags", "normal")])), false)]
    // Test comma-separated values: actual contains comma, expected doesn't
    #[case(Some(&h(&[("tags", "important")])), Some(&h(&[("tags", "important,urgent")])), true)]
    #[case(Some(&h(&[("tags", "urgent")])), Some(&h(&[("tags", "important,urgent")])), true)]
    #[case(Some(&h(&[("tags", "normal")])), Some(&h(&[("tags", "important,urgent")])), false)]
    fn test_hashmap_intersects(
        #[case] expected: Option<&HashMap<String, String>>,
        #[case] actual: Option<&HashMap<String, String>>,
        #[case] result: bool,
    ) {
        assert_eq!(hashmap_intersects(expected, actual), result);
    }

    #[rstest]
    fn test_hashmap_to_value_single_values() {
        let map = h(&[("page", "1"), ("limit", "10")]);
        let value = hashmap_to_value(&map);
        assert_eq!(value["page"], "1");
        assert_eq!(value["limit"], "10");
    }

    #[rstest]
    fn test_hashmap_to_value_multiple_values() {
        let mut map = HashMap::new();
        map.insert("tags".to_string(), "important,urgent".to_string());
        let value = hashmap_to_value(&map);
        assert_eq!(value["tags"], json!(["important", "urgent"]));
    }
}
