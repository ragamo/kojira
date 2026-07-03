use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct JiraUser {
    #[serde(rename = "displayName")]
    pub display_name: String,
    #[serde(rename = "emailAddress")]
    pub email: Option<String>,
    #[allow(dead_code)]
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

#[derive(Debug, Clone, Deserialize)]
pub struct JiraSearchResponse {
    #[serde(default)]
    pub issues: Vec<JiraIssue>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraIssue {
    pub key: String,
    pub fields: JiraIssueFields,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraIssueFields {
    #[serde(default)]
    pub summary: String,
    pub status: JiraStatus,
    #[serde(rename = "issuetype")]
    pub issue_type: JiraIssueType,
    #[allow(dead_code)]
    pub priority: Option<JiraPriority>,
    pub assignee: Option<JiraUser>,
    pub parent: Option<JiraParentField>,
    #[serde(default)]
    pub updated: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraStatus {
    pub id: Option<String>,
    pub name: String,
    #[serde(rename = "statusCategory")]
    pub status_category: Option<JiraStatusCategory>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraStatusCategory {
    pub key: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraIssueType {
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraPriority {
    #[allow(dead_code)]
    pub name: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraParentField {
    pub key: String,
    pub fields: Option<JiraParentFields>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraParentFields {
    pub summary: String,
}

// Agile API types

#[derive(Debug, Clone, Deserialize)]
pub struct JiraBoardListResponse {
    #[serde(default)]
    pub values: Vec<JiraBoard>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraBoard {
    pub id: u64,
    pub name: String,
    #[serde(rename = "type")]
    pub board_type: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraBoardConfig {
    #[serde(rename = "columnConfig")]
    pub column_config: JiraColumnConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraColumnConfig {
    pub columns: Vec<JiraBoardColumn>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraBoardColumn {
    pub name: String,
    #[serde(default)]
    pub statuses: Vec<JiraColumnStatus>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraColumnStatus {
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraBoardIssuesResponse {
    #[serde(default)]
    pub issues: Vec<JiraIssue>,
}
