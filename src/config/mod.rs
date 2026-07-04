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

pub fn load_column_order_cache(project_key: &str) -> Option<Vec<String>> {
    let path = config_dir().join(format!("columns_{}.json", project_key));
    let content = fs::read_to_string(path).ok()?;
    serde_json::from_str(&content).ok()
}

pub fn save_column_order_cache(project_key: &str, columns: &[String]) {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let path = dir.join(format!("columns_{}.json", project_key));
    if let Ok(content) = serde_json::to_string(columns) {
        let _ = fs::write(path, content);
    }
}
