pub mod key_config;
pub use key_config::KeyConfig;

use std::{fs, path::{Path, PathBuf}, time::Duration};
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use directories::ProjectDirs;

// Constants used to create the project's config directory and config file paths.
const AUTHOR:           &str = "rhasler1";
const DOMAIN:           &str = "io";
const SOFTWARE_NAME:    &str = "tui-template-rs";
const CONFIG_FILE_NAME: &str = "tui_template_rs_config.toml";

/// The project's global configuration structure.
/// 
/// Serialization and deserialization is derived using the crate `serde`.
/// Additionally, the structure's associated functions and methods assume
/// the project's config file is written in the `toml` data format.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Config {
    pub event_tick_rate: Duration,
    pub event_mpsc_channel_capacity: usize,
    pub key_config: KeyConfig
}

impl Default for Config {
    /// Default configuration.
    fn default() -> Self {
        Self {
            event_tick_rate: Duration::from_millis(248),
            event_mpsc_channel_capacity: 16,
            key_config: KeyConfig::default()
        }
    }
}

impl Config {
    /// Creates a `Config` struct by loading contents from the project's config file.
    /// 
    /// Propagates errors from `get_config_file_path()` and `load_inner()`.
    pub fn load() -> Result<Self> {
        let config_file_path = get_config_file_path()?;
        Self::load_inner(&config_file_path)
    }

    /// Creates a `Config` struct by reading contents from path and deserializing the contents.
    /// 
    /// Propagates errors from `fs::read_to_string() and toml::from_str()`.
    fn load_inner(path: &PathBuf) -> Result<Self> {
        let config: String = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&config)?;
        Ok(config)
    }

    /// Serialize the default Config, then write to project `config_example.toml` file and the
    /// project config file.
    ///
    /// This function is for development purposes.
    #[cfg(debug_assertions)]
    pub fn serialize_config_to_example_and_replace(&self) -> Result<()> {
        let example_config_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("config_example.toml");
        let content = toml::to_string(&self)?;
        fs::write(&example_config_path, &content)?;
        
        let config_dir_path = get_config_dir_path()?;
        let config_file_path = get_config_file_path()?;

        build_dir_and_overwrite_to_file(&config_dir_path, &config_file_path, &content)?;
        Ok(())
    }
}

/// Setup the project's config file if it does not already exists.
/// 
/// This function propagates errors from functions `get_config_dir_path()`, `get_config_file_path()`,
/// and `build_dir_and_write_to_file()`
pub fn setup_config_file() -> Result<()> {
    // Build project config directory if it does not already exist
    let config_dir_path = get_config_dir_path()?;
    let config_file_path = get_config_file_path()?;
    let config_example_content = include_str!("../config_example.toml");
    build_dir_and_write_to_file(&config_dir_path, &config_file_path, config_example_content)?;

    Ok(())
}

/// Recursively builds directory path and writes content to file if it does not already exist.
/// 
/// This function propagates errors from functions `fs::create_dir_all(dir_path)`, `fs::exists(file_path)`, 
/// and `fs::write(content)`.
fn build_dir_and_write_to_file(dir_path: &PathBuf, file_path: &PathBuf, content: &str) -> Result<()> {
    // Recursively create a directory and all of its parent components if they are missing.
    fs::create_dir_all(dir_path)?;

    if !fs::exists(file_path)? {
        fs::write(file_path, content)?;
    }

    Ok(())
}

/// Recursively builds directory path and writes content to file. If the file already exists, it's contents
/// are overwritten.
/// 
/// This function propagates errors from functions `fs::create_dir_all(dir_path)`, `fs::exists(file_path)`, 
/// and `fs::write(content)`.
/// 
/// This function is for development purposes.
#[cfg(debug_assertions)]
fn build_dir_and_overwrite_to_file(dir_path: &PathBuf, file_path: &PathBuf, content: &str) -> Result<()> {
    // Recursively create a directory and all of its parent components if they are missing.
    fs::create_dir_all(dir_path)?;
    fs::write(file_path, content)?;

    Ok(())
}

/// Builds the path to the project's config directory using the crate `ProjectDirs`.
///
/// This function coerces the result of `ProjectDirs::from()` from type Option to type Result.
fn get_config_dir_path() -> Result<PathBuf> {
    // coerce option to result
    if let Some(proj_dir) = ProjectDirs::from(DOMAIN, AUTHOR, SOFTWARE_NAME) {
        Ok(proj_dir.config_dir().to_path_buf())
    } else {
        Err(anyhow!("A valid home directory path could be retrieved from the operating system."))
    }
}

/// Builds the path to the project's config file using the crate `ProjectDirs`.
///
/// This function coerces the result of `ProjectDirs::from()` from type Option to type Result.
fn get_config_file_path() -> Result<PathBuf> {
    // coerce option to result
    if let Some(proj_dir) = ProjectDirs::from(DOMAIN, AUTHOR, SOFTWARE_NAME) {
        Ok(proj_dir.config_dir().join(CONFIG_FILE_NAME).to_path_buf())
    } else {
        Err(anyhow!("A valid home directory path could be retrieved from the operating system."))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::fs;

    #[test]
    fn test_build_dir_and_write_to_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_config_dir = temp_dir.path().join("foo_app");
        let temp_config_file = temp_config_dir.join("foo_config.toml");
        // Content does not need to be valid here
        let content = "Test";

        assert!(!temp_config_dir.exists());
        assert!(!temp_config_file.exists());

        build_dir_and_write_to_file(&temp_config_dir, &temp_config_file, &content).unwrap();
        assert!(temp_config_dir.is_dir());
        assert!(temp_config_file.is_file());
    }

    #[test]
    fn test_build_dir_and_write_to_file_already_exists() {
        let temp_dir = tempfile::tempdir().unwrap();
        let temp_config_dir = temp_dir.path().join("foo_app");
        let temp_config_file = temp_config_dir.join("foo_config.toml");

        fs::create_dir_all(&temp_config_dir).unwrap();
        // Content does not need to be valid here
        fs::write(&temp_config_file, "Test").unwrap();

        build_dir_and_write_to_file(&temp_config_dir, &temp_config_file, "Test2").unwrap();
        let content = fs::read_to_string(&temp_config_file).unwrap();
        assert_eq!(content, "Test");
    }

    #[test]
    fn test_config_load_inner() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test_config.toml");

        // Content must be a toml string that deserializes into Config, otherwise
        // the assertion will fail.
        let content = toml::to_string(&Config::default()).unwrap();
        fs::write(&file_path, content).unwrap();

        let result = Config::load_inner(&file_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_get_config_dir_path() {
        let config_dir_path = get_config_dir_path();
        assert!(config_dir_path.is_ok());

        let config_dir_path = config_dir_path.unwrap();
        assert!(config_dir_path.to_string_lossy().contains(DOMAIN));
        assert!(config_dir_path.to_string_lossy().contains(AUTHOR));
        assert!(config_dir_path.to_string_lossy().contains(SOFTWARE_NAME));
    }

    #[test]
    fn test_get_config_file_path() {
        let config_file_path = get_config_file_path();
        assert!(config_file_path.is_ok());

        let config_file_path = config_file_path.unwrap();
        assert!(config_file_path.to_string_lossy().contains(DOMAIN));
        assert!(config_file_path.to_string_lossy().contains(AUTHOR));
        assert!(config_file_path.to_string_lossy().contains(SOFTWARE_NAME));
        assert!(config_file_path.to_string_lossy().contains(CONFIG_FILE_NAME));
    }
}
