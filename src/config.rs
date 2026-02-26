pub mod error;

use error::{ConfigError, ConfigResult};
use serde::Deserialize;
use std::{fs, path::Path};

pub const DEFAULT_ACTOR_BUFFER_SIZE: usize = 100;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct SidsConfig {
    #[serde(default)]
    pub configuration: ActorSystemConfig,
}

#[derive(Debug, Clone, Deserialize)]
/// Configuration for the actor system, including settings for actor buffer size and shutdown timeout.
pub struct ActorSystemConfig {
    #[serde(default = "default_actor_buffer_size")]
    pub actor_buffer_size: usize,
    #[serde(default)]
    pub shutdown_timeout_ms: Option<u64>,
}

impl Default for ActorSystemConfig {
    fn default() -> Self {
        Self {
            actor_buffer_size: DEFAULT_ACTOR_BUFFER_SIZE,
            shutdown_timeout_ms: None,
        }
    }
}

impl SidsConfig {
    /// Parses a TOML string into a `SidsConfig` instance. If parsing fails, an error is returned.
    pub fn from_toml_str(toml_str: &str) -> ConfigResult<Self> {
        toml::from_str(toml_str).map_err(|source| ConfigError::ParseError { source })
    }

    /// Loads a configuration from a TOML file at the specified path. If reading or parsing the file fails, an error is returned.
    pub fn load_from_file(path: impl AsRef<Path>) -> ConfigResult<Self> {
        let path_ref = path.as_ref();
        let content =
            fs::read_to_string(path_ref).map_err(|source| ConfigError::FileReadError {
                path: path_ref.display().to_string(),
                source,
            })?;
        Self::from_toml_str(&content)
    }
}

fn default_actor_buffer_size() -> usize {
    DEFAULT_ACTOR_BUFFER_SIZE
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SidsConfig::default();
        assert_eq!(
            config.configuration.actor_buffer_size,
            DEFAULT_ACTOR_BUFFER_SIZE
        );
        assert!(config.configuration.shutdown_timeout_ms.is_none());
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[configuration]
actor_buffer_size = 256
shutdown_timeout_ms = 1000
"#;
        let config = SidsConfig::from_toml_str(toml_str).expect("Failed to parse config");
        assert_eq!(config.configuration.actor_buffer_size, 256);
        assert_eq!(config.configuration.shutdown_timeout_ms, Some(1000));
    }

    #[test]
    fn test_example_config_parses() {
        let toml_str = include_str!("../sids.config.example.toml");
        let config = SidsConfig::from_toml_str(toml_str).expect("Failed to parse example config");
        assert!(config.configuration.actor_buffer_size > 0);
    }
}
