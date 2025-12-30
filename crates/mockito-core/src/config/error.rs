//! Error types for configuration parsing.

use thiserror::Error;

/// Configuration parsing error
#[derive(Debug, Error)]
pub enum ConfigError {
    /// JSON parsing error
    #[error("JSON parsing error: {0}")]
    Json(#[from] serde_json::Error),
    /// YAML parsing error
    #[error("YAML parsing error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    /// Unknown file type
    #[error("Unknown file type: {0}")]
    UnknownFileType(String),
    /// Glob pattern error
    #[error("Glob pattern error: {0}")]
    GlobPattern(String),
    /// IO error when reading file
    #[error("Failed to read file {path}: {source}")]
    Io {
        #[source]
        source: std::io::Error,
        path: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

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

    #[rstest]
    fn test_config_error_glob_pattern_display() {
        let error = ConfigError::GlobPattern("Invalid glob pattern: *[".to_string());
        let display = format!("{}", error);
        assert!(display.contains("Glob pattern error"));
        assert!(display.contains("Invalid glob pattern"));
    }

    #[rstest]
    fn test_config_error_io_display() {
        use std::io;
        let io_err = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let error = ConfigError::Io {
            source: io_err,
            path: "test.yaml".to_string(),
        };
        let display = format!("{}", error);
        assert!(display.contains("Failed to read file"));
        assert!(display.contains("test.yaml"));
        assert!(display.contains("File not found"));
    }
}
