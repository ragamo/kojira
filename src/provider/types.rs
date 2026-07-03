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
