use reqwest::Client;
use thiserror::Error;

use super::types::{
    JiraBoard, JiraBoardConfig, JiraBoardIssuesResponse, JiraBoardListResponse, JiraIssue,
    JiraProject, JiraProjectSearchResponse, JiraSearchResponse, JiraUser,
};

#[derive(Debug, Error)]
pub enum JiraError {
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Authentication failed: {0}")]
    Auth(String),
}

pub struct JiraProvider {
    client: Client,
    base_url: String,
    email: String,
    token: String,
}

impl JiraProvider {
    pub fn new(client: Client, base_url: String, email: String, token: String) -> Self {
        Self {
            client,
            base_url,
            email,
            token,
        }
    }

    pub async fn get_current_user(&self) -> Result<JiraUser, JiraError> {
        let url = format!("{}/rest/api/3/myself", self.base_url);
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.token))
            .send()
            .await?;

        if resp.status() == 401 || resp.status() == 403 {
            return Err(JiraError::Auth("Invalid email or token".into()));
        }

        if !resp.status().is_success() {
            return Err(JiraError::Auth(format!("HTTP {}", resp.status())));
        }

        let user: JiraUser = resp.json().await?;
        Ok(user)
    }

    pub async fn get_backlog(&self, project_key: &str) -> Result<Vec<JiraIssue>, JiraError> {
        let url = format!("{}/rest/api/3/search/jql", self.base_url);
        let jql = format!("project = {} ORDER BY updated DESC", project_key);
        let body = serde_json::json!({
            "jql": jql,
            "maxResults": 100,
            "fields": ["summary", "status", "issuetype", "priority", "assignee", "parent", "updated"]
        });
        let resp = self
            .client
            .post(&url)
            .basic_auth(&self.email, Some(&self.token))
            .json(&body)
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(JiraError::Auth(format!("HTTP {}: {}", status, &text[..text.len().min(200)])));
        }

        let text = resp.text().await?;
        let data: JiraSearchResponse = serde_json::from_str(&text)
            .map_err(|e| JiraError::Auth(format!("Parse error: {} — body: {}", e, &text[..text.len().min(300)])))?;
        Ok(data.issues)
    }

    pub async fn get_boards(&self, project_key: &str) -> Result<Vec<JiraBoard>, JiraError> {
        let url = format!("{}/rest/agile/1.0/board", self.base_url);
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.token))
            .query(&[("projectKeyOrId", project_key)])
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(JiraError::Auth(format!("HTTP {}: {}", status, &text[..text.len().min(200)])));
        }

        let data: JiraBoardListResponse = resp.json().await?;
        Ok(data.values)
    }

    pub async fn get_board_config(&self, board_id: u64) -> Result<JiraBoardConfig, JiraError> {
        let url = format!("{}/rest/agile/1.0/board/{}/configuration", self.base_url, board_id);
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.token))
            .send()
            .await?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(JiraError::Auth(format!("HTTP {}: {}", status, &text[..text.len().min(200)])));
        }

        let data: JiraBoardConfig = resp.json().await?;
        Ok(data)
    }

    pub async fn get_board_issues(&self, board_id: u64) -> Result<Vec<JiraIssue>, JiraError> {
        let url = format!("{}/rest/agile/1.0/board/{}/issue", self.base_url, board_id);
        let mut all_issues = Vec::new();
        let mut start_at = 0u32;

        loop {
            let resp = self
                .client
                .get(&url)
                .basic_auth(&self.email, Some(&self.token))
                .query(&[
                    ("startAt", &start_at.to_string()),
                    ("maxResults", &"100".to_string()),
                    ("fields", &"summary,status,issuetype,priority,assignee,parent,updated".to_string()),
                ])
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(JiraError::Auth(format!("HTTP {}: {}", status, &text[..text.len().min(200)])));
            }

            let data: JiraBoardIssuesResponse = resp.json().await?;
            let count = data.issues.len() as u32;
            all_issues.extend(data.issues);

            if count < 100 {
                break;
            }
            start_at += count;
        }

        Ok(all_issues)
    }

    pub async fn search_projects(&self, query: &str) -> Result<Vec<JiraProject>, JiraError> {
        let url = format!("{}/rest/api/3/project/search", self.base_url);
        let resp = self
            .client
            .get(&url)
            .basic_auth(&self.email, Some(&self.token))
            .query(&[("query", query), ("maxResults", "20")])
            .send()
            .await?;

        if !resp.status().is_success() {
            return Err(JiraError::Auth(format!("HTTP {}", resp.status())));
        }

        let data: JiraProjectSearchResponse = resp.json().await?;
        Ok(data.values)
    }
}
