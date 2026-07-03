pub mod types;

use std::fs;
use std::path::PathBuf;

use color_eyre::Result;
use types::AppConfig;

pub fn config_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".config")
        .join("kojira")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn load_config() -> Result<AppConfig> {
    let path = config_path();
    if !path.exists() {
        return Ok(AppConfig::default());
    }
    let content = fs::read_to_string(&path)?;
    let config: AppConfig = toml::from_str(&content)?;
    Ok(config)
}

pub fn save_config(config: &AppConfig) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let content = toml::to_string_pretty(config)?;
    fs::write(config_path(), content)?;
    Ok(())
}
