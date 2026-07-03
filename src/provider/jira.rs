use reqwest::Client;
use thiserror::Error;

use super::types::{JiraProject, JiraProjectSearchResponse, JiraUser};

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
