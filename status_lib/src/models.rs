//! Shared data models for catalog services and live fetch results.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Service {
    pub name: String,
    #[serde(default)]
    pub status_url: Option<String>,
    #[serde(default)]
    pub website_url: Option<String>,
    #[serde(default)]
    pub security_url: Option<String>,
    #[serde(default)]
    pub support_url: Option<String>,
    #[serde(default)]
    pub aux_urls: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatchType {
    Exact,
    Substring,
    Fuzzy,
}

#[derive(Debug, Clone)]
pub struct FuzzyMatch {
    pub service: Service,
    pub score: f64,
    pub match_type: MatchType,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FetchResult {
    pub status_url: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub feed_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub history_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latest_status: Option<String>,
    #[serde(default)]
    pub history: Vec<String>,
    #[serde(default)]
    pub messages: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extracted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status_code: Option<u16>,
}

impl FetchResult {
    pub fn empty(status_url: impl Into<String>) -> Self {
        Self {
            status_url: status_url.into(),
            api_url: None,
            feed_url: None,
            history_url: None,
            latest_status: None,
            history: Vec::new(),
            messages: Vec::new(),
            extracted_at: None,
            error: None,
            http_status_code: None,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PartialStatus {
    pub latest_status: Option<String>,
    pub history: Vec<String>,
    pub messages: Vec<String>,
    pub error: Option<String>,
    pub http_status_code: Option<u16>,
}
