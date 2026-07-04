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
    pub board_hide_subtasks: Option<bool>,
    pub board_hide_backlog_col: Option<bool>,
    pub content_bg_solid: Option<bool>,
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
    pub open_tabs: Vec<OpenTab>,
    #[serde(default)]
    pub user_tabs: Vec<UserTabs>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserTabs {
    pub instance_url: String,
    pub email: String,
    pub tabs: Vec<OpenTab>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FavoriteProject {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum OpenTab {
    List {
        project_key: String,
        project_name: String,
        id: u64,
    },
    Board {
        project_key: String,
        board_id: u64,
        board_name: String,
    },
}
