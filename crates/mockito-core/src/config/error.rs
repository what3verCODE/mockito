//! Error types for configuration parsing.

use std::fmt;

/// Configuration parsing error
#[derive(Debug)]
pub enum ConfigError {
    /// JSON parsing error
    Json(serde_json::Error),
    /// YAML parsing error
    Yaml(serde_yaml::Error),
    /// Unknown file type
    UnknownFileType(String),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Json(e) => write!(f, "JSON parsing error: {}", e),
            ConfigError::Yaml(e) => write!(f, "YAML parsing error: {}", e),
            ConfigError::UnknownFileType(path) => write!(f, "Unknown file type: {}", path),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Json(e) => Some(e),
            ConfigError::Yaml(e) => Some(e),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for ConfigError {
    fn from(err: serde_json::Error) -> Self {
        ConfigError::Json(err)
    }
}

impl From<serde_yaml::Error> for ConfigError {
    fn from(err: serde_yaml::Error) -> Self {
        ConfigError::Yaml(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use std::error::Error;

    #[rstest]
    fn test_config_error_json_display() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid json").unwrap_err();
        let error = ConfigError::from(json_err);
        let display = format!("{}", error);
        assert!(display.contains("JSON parsing error"));
    }

    #[rstest]
    fn test_config_error_yaml_display() {
        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: yaml: [").unwrap_err();
        let error = ConfigError::from(yaml_err);
        let display = format!("{}", error);
        assert!(display.contains("YAML parsing error"));
    }

    #[rstest]
    #[case("test.txt")]
    #[case("unknown.extension")]
    #[case("")]
    fn test_config_error_unknown_file_type_display(#[case] path: &str) {
        let error = ConfigError::UnknownFileType(path.to_string());
        let display = format!("{}", error);
        assert!(display.contains("Unknown file type"));
        assert!(display.contains(path));
    }

    #[rstest]
    fn test_config_error_source() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let error = ConfigError::from(json_err);
        assert!(error.source().is_some());

        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: [").unwrap_err();
        let error = ConfigError::from(yaml_err);
        assert!(error.source().is_some());

        let error = ConfigError::UnknownFileType("test.txt".to_string());
        assert!(error.source().is_none());
    }

    #[rstest]
    fn test_config_error_from_serde_json() {
        let json_err = serde_json::from_str::<serde_json::Value>("invalid").unwrap_err();
        let error: ConfigError = json_err.into();
        assert!(matches!(error, ConfigError::Json(_)));
    }

    #[rstest]
    fn test_config_error_from_serde_yaml() {
        let yaml_err = serde_yaml::from_str::<serde_yaml::Value>("invalid: [").unwrap_err();
        let error: ConfigError = yaml_err.into();
        assert!(matches!(error, ConfigError::Yaml(_)));
    }
}
