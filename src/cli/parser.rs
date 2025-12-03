use crate::result::{OpenCliError, Result};
use std::path::PathBuf;

pub struct CliParser;

impl CliParser {
    pub fn validate_config_path(path: &str) -> Result<PathBuf> {
        let config_path = PathBuf::from(path);

        if !config_path.exists() {
            return Err(OpenCliError::NotFound(
                format!("Config file not found: {}", path).into(),
            ));
        }

        if !config_path.is_file() {
            return Err(OpenCliError::Config("Path is not a file".into()));
        }

        Ok(config_path)
    }

    pub fn validate_port(port: u16) -> Result<u16> {
        match port {
            1..=65535 => Ok(port),
            _ => Err(OpenCliError::Config(
                "Port must be between 1 and 65535".into(),
            )),
        }
    }
}
