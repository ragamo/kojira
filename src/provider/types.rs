use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct JiraUser {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email: Option<String>,
    #[serde(rename = "accountId")]
    pub account_id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraProject {
    pub key: String,
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraProjectSearchResponse {
    pub values: Vec<JiraProject>,
}
