use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub auth: AuthConfig,
    #[serde(default)]
    pub jira: JiraConfig,
    #[serde(default)]
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UiConfig {
    pub theme: Option<String>,
    pub header_bg: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AuthConfig {
    pub token: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct JiraConfig {
    pub base_url: Option<String>,
    pub project: Option<String>,
    #[serde(default)]
    pub favorites: Vec<FavoriteProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteProject {
    pub key: String,
    pub name: String,
}
