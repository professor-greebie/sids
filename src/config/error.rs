use std::fmt;
use std::io;

/// Configuration errors
#[derive(Debug)]
pub enum ConfigError {
    /// Failed to read configuration file
    FileReadError { path: String, source: io::Error },
    /// Failed to parse TOML configuration
    ParseError { source: toml::de::Error },
    /// Configuration validation failed
    ValidationError { reason: String },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::FileReadError { path, source } => {
                write!(f, "Failed to read config file '{}': {}", path, source)
            }
            ConfigError::ParseError { source } => {
                write!(f, "Failed to parse config: {}", source)
            }
            ConfigError::ValidationError { reason } => {
                write!(f, "Config validation failed: {}", reason)
            }
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::FileReadError { source, .. } => Some(source),
            ConfigError::ParseError { source } => Some(source),
            ConfigError::ValidationError { .. } => None,
        }
    }
}

/// Result type for configuration operations
pub type ConfigResult<T> = Result<T, ConfigError>;
