//! Configuration file parsing (YAML/JSON/JSONC).

use crate::config::error::ConfigError;
use serde::de::DeserializeOwned;
use std::path::Path;

/// Config file type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileType {
    Yaml,
    Json,
    Jsonc,
    Unknown,
}

/// Get config file type from path extension
pub fn get_file_type(path: &str) -> ConfigFileType {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    match ext.as_str() {
        "yaml" | "yml" => ConfigFileType::Yaml,
        "json" => ConfigFileType::Json,
        "jsonc" => ConfigFileType::Jsonc,
        _ => ConfigFileType::Unknown,
    }
}

/// Strip comments from JSONC content
pub fn strip_json_comments(content: &str) -> String {
    let mut result = String::with_capacity(content.len());
    let chars: Vec<char> = content.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_string = false;
    let mut in_line_comment = false;
    let mut in_block_comment = false;

    while i < len {
        let c = chars[i];
        let next = chars.get(i + 1).copied();

        // Handle string boundaries
        if c == '"' && !in_line_comment && !in_block_comment {
            let mut escape_count = 0;
            let mut j = result.len();
            while j > 0 && result.chars().nth(j - 1) == Some('\\') {
                escape_count += 1;
                j -= 1;
            }
            if escape_count % 2 == 0 {
                in_string = !in_string;
            }
        }

        if !in_string {
            // Start line comment
            if c == '/' && next == Some('/') && !in_block_comment {
                in_line_comment = true;
                i += 2;
                continue;
            }
            // Start block comment
            if c == '/' && next == Some('*') && !in_line_comment {
                in_block_comment = true;
                i += 2;
                continue;
            }
            // End line comment
            if in_line_comment && (c == '\n' || c == '\r') {
                in_line_comment = false;
                result.push(c);
                i += 1;
                continue;
            }
            // End block comment
            if in_block_comment && c == '*' && next == Some('/') {
                in_block_comment = false;
                i += 2;
                continue;
            }
        }

        if !in_line_comment && !in_block_comment {
            result.push(c);
        }
        i += 1;
    }

    result
}

/// Parse JSON content
pub fn parse_json<T: DeserializeOwned>(content: &str) -> Result<T, ConfigError> {
    serde_json::from_str(content).map_err(ConfigError::from)
}

/// Parse JSONC content (JSON with comments)
pub fn parse_jsonc<T: DeserializeOwned>(content: &str) -> Result<T, ConfigError> {
    let stripped = strip_json_comments(content);
    serde_json::from_str(&stripped).map_err(ConfigError::from)
}

/// Parse YAML content
pub fn parse_yaml<T: DeserializeOwned>(content: &str) -> Result<T, ConfigError> {
    serde_yaml::from_str(content).map_err(ConfigError::from)
}

/// Parse config content based on file type
pub fn parse_config<T: DeserializeOwned>(content: &str, path: &str) -> Result<T, ConfigError> {
    match get_file_type(path) {
        ConfigFileType::Yaml => parse_yaml(content),
        ConfigFileType::Json => parse_json(content),
        ConfigFileType::Jsonc => parse_jsonc(content),
        ConfigFileType::Unknown => Err(ConfigError::UnknownFileType(path.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::route::Route;
    use rstest::rstest;

    #[rstest]
    #[case("test.yaml", ConfigFileType::Yaml)]
    #[case("test.YAML", ConfigFileType::Yaml)]
    #[case("test.yml", ConfigFileType::Yaml)]
    #[case("test.YML", ConfigFileType::Yaml)]
    #[case("test.json", ConfigFileType::Json)]
    #[case("test.JSON", ConfigFileType::Json)]
    #[case("test.jsonc", ConfigFileType::Jsonc)]
    #[case("test.JSONC", ConfigFileType::Jsonc)]
    #[case("test.txt", ConfigFileType::Unknown)]
    #[case("test", ConfigFileType::Unknown)]
    #[case("", ConfigFileType::Unknown)]
    fn test_get_file_type(#[case] path: &str, #[case] expected: ConfigFileType) {
        assert_eq!(get_file_type(path), expected);
    }

    #[rstest]
    #[case("{\"key\": \"value\"}", "{\"key\":\"value\"}")]
    #[case("{\"key\": \"value\"} // comment", "{\"key\":\"value\"} ")]
    #[case("{\"key\": \"value\"} /* block */", "{\"key\":\"value\"} ")]
    #[case("{\"key\": \"value\"} // comment\n", "{\"key\":\"value\"} \n")]
    #[case("{\"key\": \"value\"} /* block */\n", "{\"key\":\"value\"} \n")]
    #[case(
        "{\"key\": \"value\"} // comment\n{\"key2\": \"value2\"}",
        "{\"key\":\"value\"} \n{\"key2\":\"value2\"}"
    )]
    #[case(
        "{\"key\": \"value\"} /* block */\n{\"key2\": \"value2\"}",
        "{\"key\":\"value\"} \n{\"key2\":\"value2\"}"
    )]
    #[case(
        "{\"key\": \"value\"} // comment\n{\"key2\": \"value2\"} // comment2",
        "{\"key\":\"value\"} \n{\"key2\":\"value2\"} "
    )]
    #[case(
        "{\"key\": \"value\"} /* block */\n{\"key2\": \"value2\"} /* block2 */",
        "{\"key\":\"value\"} \n{\"key2\":\"value2\"} "
    )]
    #[case(
        "{\"key\": \"value\"} // comment\n{\"key2\": \"value2\"} /* block */",
        "{\"key\":\"value\"} \n{\"key2\":\"value2\"} "
    )]
    fn test_strip_json_comments(#[case] input: &str, #[case] expected: &str) {
        let result = strip_json_comments(input);
        // Normalize whitespace for comparison
        let result_normalized = result.replace(" ", "").replace("\n", "");
        let expected_normalized = expected.replace(" ", "").replace("\n", "");
        assert_eq!(result_normalized, expected_normalized);
    }

    #[rstest]
    #[case(r#"{"key": "value"}"#)]
    #[case(r#"{"key": "value"} // comment"#)]
    #[case(r#"{"key": "value"} /* block */"#)]
    fn test_strip_json_comments_preserves_valid_json(#[case] input: &str) {
        let stripped = strip_json_comments(input);
        // Should be valid JSON after stripping
        let result: Result<serde_json::Value, _> = serde_json::from_str(&stripped);
        assert!(
            result.is_ok(),
            "Failed to parse JSON after stripping comments: {}",
            stripped
        );
    }

    #[rstest]
    fn test_strip_json_comments_preserves_strings() {
        let input = r#"{"key": "value // not a comment"}"#;
        let result = strip_json_comments(input);
        assert!(result.contains("value // not a comment"));
    }

    #[rstest]
    fn test_strip_json_comments_preserves_escaped_quotes() {
        let input = r#"{"key": "value \"quote\" here"}"#;
        let result = strip_json_comments(input);
        assert!(result.contains("value \\\"quote\\\" here"));
    }

    #[rstest]
    fn test_parse_json_valid() {
        let content = r#"{"id": "test", "name": "value"}"#;
        let result: Result<serde_json::Value, _> = parse_json(content);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["id"], "test");
        assert_eq!(value["name"], "value");
    }

    #[rstest]
    fn test_parse_json_invalid() {
        let content = "invalid json";
        let result: Result<serde_json::Value, _> = parse_json(content);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Json(_)));
    }

    #[rstest]
    fn test_parse_jsonc_valid() {
        let content = r#"{"id": "test"} // comment"#;
        let result: Result<serde_json::Value, _> = parse_jsonc(content);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["id"], "test");
    }

    #[rstest]
    fn test_parse_jsonc_with_block_comment() {
        let content = r#"{"id": "test"} /* block comment */"#;
        let result: Result<serde_json::Value, _> = parse_jsonc(content);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["id"], "test");
    }

    #[rstest]
    fn test_parse_yaml_valid() {
        let content = "id: test\nname: value";
        let result: Result<serde_json::Value, _> = parse_yaml(content);
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["id"], "test");
        assert_eq!(value["name"], "value");
    }

    #[rstest]
    fn test_parse_yaml_invalid() {
        let content = "invalid: yaml: [";
        let result: Result<serde_json::Value, _> = parse_yaml(content);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Yaml(_)));
    }

    #[rstest]
    fn test_parse_config_json() {
        let content = r#"{"id": "test", "url": "/api", "transport": "HTTP", "presets": []}"#;
        let result: Result<Route, _> = parse_config(content, "test.json");
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_parse_config_jsonc() {
        let content =
            r#"{"id": "test", "url": "/api", "transport": "HTTP", "presets": []} // comment"#;
        let result: Result<Route, _> = parse_config(content, "test.jsonc");
        assert!(result.is_ok());
    }

    #[rstest]
    fn test_parse_config_yaml() {
        let content = "id: test\nurl: /api\ntransport: HTTP\npresets: []";
        let result: Result<Route, _> = parse_config(content, "test.yaml");
        assert!(result.is_ok());
    }

    #[rstest]
    #[case("test.txt")]
    #[case("test.unknown")]
    #[case("")]
    fn test_parse_config_unknown_file_type(#[case] path: &str) {
        let content = r#"{"id": "test"}"#;
        let result: Result<serde_json::Value, _> = parse_config(content, path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::UnknownFileType(_)
        ));
    }
}
