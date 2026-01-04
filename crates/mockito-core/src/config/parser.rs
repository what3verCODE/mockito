//! Configuration file parsing (YAML/JSON/JSONC).

use crate::config::error::ConfigError;
use crate::types::{collection::Collection, route::Route};
use glob::glob;
use serde::de::DeserializeOwned;
use std::{fs, path::Path};

/// Config file type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfigFileType {
    Yaml,
    Json,
    Jsonc,
    Unknown,
}

/// Get config file type from path extension.
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

/// Check if file is a supported config file
fn is_supported_config_file(path: &str) -> bool {
    !matches!(get_file_type(path), ConfigFileType::Unknown)
}

/// Strip comments from JSONC content.
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

fn expand_glob(pattern: &str) -> Result<Vec<String>, ConfigError> {
    let entries = glob(pattern)
        .map_err(|e| ConfigError::GlobPattern(format!("Invalid glob pattern: {}", e)))?;

    let mut paths = Vec::new();
    for entry in entries {
        let path =
            entry.map_err(|e| ConfigError::GlobPattern(format!("Glob pattern error: {}", e)))?;
        if let Some(s) = path.to_str() {
            paths.push(s.to_owned());
        }
    }

    Ok(paths)
}

/// Load routes from a file or glob pattern.
pub fn load_routes(pattern: &str) -> Result<Vec<Route>, ConfigError> {
    let paths = expand_glob(pattern)?;
    let mut routes = Vec::new();

    for p in paths {
        if !is_supported_config_file(&p) {
            continue;
        }
        let content = fs::read_to_string(&p).map_err(|e| ConfigError::Io {
            source: e,
            path: p.clone(),
        })?;
        let parsed: Route = parse_config(&content, &p)?;
        routes.push(parsed);
    }

    Ok(routes)
}

/// Load collections from a file.
/// Supports both single collection and array of collections.
pub fn load_collections(path: &str) -> Result<Vec<Collection>, ConfigError> {
    let content = fs::read_to_string(path).map_err(|e| ConfigError::Io {
        source: e,
        path: path.to_string(),
    })?;

    // Try to parse as array first, then as single collection
    match parse_config::<Vec<Collection>>(&content, path) {
        Ok(collections) => Ok(collections),
        Err(_) => {
            // If array parsing fails, try single collection
            let collection = parse_config::<Collection>(&content, path)?;
            Ok(vec![collection])
        }
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

    #[rstest]
    #[case("test.yaml", true)]
    #[case("test.yml", true)]
    #[case("test.json", true)]
    #[case("test.jsonc", true)]
    #[case("test.txt", false)]
    #[case("test.unknown", false)]
    #[case("", false)]
    fn test_is_supported_config_file(#[case] path: &str, #[case] expected: bool) {
        assert_eq!(is_supported_config_file(path), expected);
    }

    #[rstest]
    fn test_expand_glob_valid_pattern() {
        // Create a temporary test file
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_route.json");
        let test_content = r#"{"id": "test", "url": "/api", "transport": "HTTP", "presets": []}"#;
        std::fs::write(&test_file, test_content).unwrap();

        let pattern = test_file.to_str().unwrap();
        let result = expand_glob(pattern);
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert!(!paths.is_empty());
        assert!(paths.contains(&pattern.to_string()));

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_expand_glob_invalid_pattern() {
        let result = expand_glob("[invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::GlobPattern(_)));
    }

    #[rstest]
    fn test_expand_glob_no_matches() {
        let result = expand_glob("nonexistent_*.json");
        assert!(result.is_ok());
        let paths = result.unwrap();
        assert!(paths.is_empty());
    }

    #[rstest]
    fn test_load_routes_single_file() {
        // Create a temporary test file
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_route_load.json");
        let test_content = r#"{"id": "test", "url": "/api", "transport": "HTTP", "presets": []}"#;
        std::fs::write(&test_file, test_content).unwrap();

        let pattern = test_file.to_str().unwrap();
        let result = load_routes(pattern);
        assert!(result.is_ok());
        let routes = result.unwrap();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].id, "test");

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_load_routes_nonexistent_file() {
        let result = load_routes("nonexistent_file.json");
        assert!(result.is_ok()); // Glob returns empty, so no error
        let routes = result.unwrap();
        assert!(routes.is_empty());
    }

    #[rstest]
    fn test_load_routes_invalid_glob_pattern() {
        let result = load_routes("[invalid");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::GlobPattern(_)));
    }

    #[rstest]
    fn test_load_routes_invalid_json() {
        // Create a temporary test file with invalid JSON
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_invalid.json");
        let test_content = "invalid json content";
        std::fs::write(&test_file, test_content).unwrap();

        let pattern = test_file.to_str().unwrap();
        let result = load_routes(pattern);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Json(_)));

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_load_routes_unsupported_file_type() {
        // Create a temporary test file with unsupported extension
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_route.txt");
        let test_content = "some content";
        std::fs::write(&test_file, test_content).unwrap();

        let pattern = test_file.to_str().unwrap();
        let result = load_routes(pattern);
        assert!(result.is_ok());
        let routes = result.unwrap();
        assert!(routes.is_empty()); // Unsupported files are skipped

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_load_collections_json() {
        // Create a temporary test file
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_collection.json");
        let test_content = r#"{"id": "test-collection", "routes": ["route1:preset1:variant1"]}"#;
        std::fs::write(&test_file, test_content).unwrap();

        let path = test_file.to_str().unwrap();
        let result = load_collections(path);
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].id, "test-collection");
        assert_eq!(collections[0].routes.len(), 1);

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_load_collections_yaml() {
        // Create a temporary test file
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_collection.yaml");
        let test_content = "id: test-collection\nroutes:\n  - route1:preset1:variant1";
        std::fs::write(&test_file, test_content).unwrap();

        let path = test_file.to_str().unwrap();
        let result = load_collections(path);
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].id, "test-collection");

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_load_collections_with_from() {
        // Create a temporary test file with 'from' field
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_collection_from.json");
        let test_content =
            r#"{"id": "child", "from": "parent", "routes": ["route1:preset1:variant1"]}"#;
        std::fs::write(&test_file, test_content).unwrap();

        let path = test_file.to_str().unwrap();
        let result = load_collections(path);
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 1);
        assert_eq!(collections[0].id, "child");
        assert_eq!(collections[0].from, Some("parent".to_string()));

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_load_collections_nonexistent_file() {
        let result = load_collections("nonexistent_file.json");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Io { .. }));
    }

    #[rstest]
    fn test_load_collections_invalid_json() {
        // Create a temporary test file with invalid JSON
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_invalid_collection.json");
        let test_content = "invalid json content";
        std::fs::write(&test_file, test_content).unwrap();

        let path = test_file.to_str().unwrap();
        let result = load_collections(path);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::Json(_)));

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_load_collections_unknown_file_type() {
        // Create a temporary test file with unsupported extension
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_collection.txt");
        let test_content = "some content";
        std::fs::write(&test_file, test_content).unwrap();

        let path = test_file.to_str().unwrap();
        let result = load_collections(path);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            ConfigError::UnknownFileType(_)
        ));

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }

    #[rstest]
    fn test_load_collections_array() {
        // Test parsing array of collections
        let test_dir = std::env::temp_dir();
        let test_file = test_dir.join("test_collections_array.json");
        let test_content =
            r#"[{"id": "collection1", "routes": []}, {"id": "collection2", "routes": []}]"#;
        std::fs::write(&test_file, test_content).unwrap();

        let path = test_file.to_str().unwrap();
        let result = load_collections(path);
        assert!(result.is_ok());
        let collections = result.unwrap();
        assert_eq!(collections.len(), 2);
        assert_eq!(collections[0].id, "collection1");
        assert_eq!(collections[1].id, "collection2");

        // Cleanup
        let _ = std::fs::remove_file(&test_file);
    }
}
