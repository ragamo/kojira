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
    pub priority: Option<JiraPriority>,
    pub assignee: Option<JiraUser>,
    pub parent: Option<JiraParentField>,
    #[serde(default)]
    pub updated: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct JiraStatus {
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
