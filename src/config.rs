use serde::Deserialize;
use std::{fs, path::Path};

pub const DEFAULT_ACTOR_BUFFER_SIZE: usize = 100;

#[derive(Debug, Clone, Deserialize)]
pub struct SidsConfig {
    #[serde(default)]
    pub actor_system: ActorSystemConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ActorSystemConfig {
    #[serde(default = "default_actor_buffer_size")]
    pub actor_buffer_size: usize,
    #[serde(default)]
    pub shutdown_timeout_ms: Option<u64>,
}

impl Default for SidsConfig {
    fn default() -> Self {
        Self {
            actor_system: ActorSystemConfig::default(),
        }
    }
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
    pub fn from_toml_str(toml_str: &str) -> Result<Self, String> {
        toml::from_str(toml_str).map_err(|err| format!("Failed to parse config: {}", err))
    }

    pub fn load_from_file(path: impl AsRef<Path>) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|err| format!("Failed to read config file: {}", err))?;
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
        assert_eq!(config.actor_system.actor_buffer_size, DEFAULT_ACTOR_BUFFER_SIZE);
        assert!(config.actor_system.shutdown_timeout_ms.is_none());
    }

    #[test]
    fn test_parse_config() {
        let toml_str = r#"
[actor_system]
actor_buffer_size = 256
shutdown_timeout_ms = 1000
"#;
        let config = SidsConfig::from_toml_str(toml_str).expect("Failed to parse config");
        assert_eq!(config.actor_system.actor_buffer_size, 256);
        assert_eq!(config.actor_system.shutdown_timeout_ms, Some(1000));
    }

    #[test]
    fn test_example_config_parses() {
        let toml_str = include_str!("../sids.config.example.toml");
        let config = SidsConfig::from_toml_str(toml_str).expect("Failed to parse example config");
        assert!(config.actor_system.actor_buffer_size > 0);
    }
}
