//! HTTP client with redirects, size limits, and timeouts.

use crate::error::{Error, Result};
use std::time::Duration;

pub const MAX_RESPONSE_SIZE: usize = 1024 * 1024;
pub const DEFAULT_TIMEOUT_SECS: u64 = 10;
pub const MAX_REDIRECTS: usize = 5;
pub const USER_AGENT: &str = "Mozilla/5.0 (compatible; StatusCli/0.1)";

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub body: String,
}

pub fn fetch(url: &str, accept: &str, timeout_secs: u64) -> Result<HttpResponse> {
    let client = reqwest::blocking::Client::builder()
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(timeout_secs))
        .user_agent(USER_AGENT)
        .build()
        .map_err(|error| Error::Network(error.to_string()))?;

    let mut current_url = url.to_string();
    for _ in 0..=MAX_REDIRECTS {
        let response = client
            .get(&current_url)
            .header(reqwest::header::ACCEPT, accept)
            .send()
            .map_err(|error| Error::Network(error.to_string()))?;

        let status = response.status();
        if status.is_redirection() {
            let location = response
                .headers()
                .get(reqwest::header::LOCATION)
                .and_then(|value| value.to_str().ok())
                .ok_or_else(|| Error::Network("redirect without location".into()))?;
            current_url = url::Url::parse(&current_url)
                .and_then(|base| base.join(location))
                .map(|joined| joined.to_string())
                .map_err(|error| Error::Network(error.to_string()))?;
            continue;
        }

        if let Some(content_length) = response.content_length() {
            if content_length as usize > MAX_RESPONSE_SIZE {
                return Err(Error::ResponseTooLarge {
                    actual: content_length as usize,
                    limit: MAX_RESPONSE_SIZE,
                });
            }
        }

        let body = response
            .text()
            .map_err(|error| Error::Network(error.to_string()))?;
        if body.len() > MAX_RESPONSE_SIZE {
            return Err(Error::ResponseTooLarge {
                actual: body.len(),
                limit: MAX_RESPONSE_SIZE,
            });
        }

        return Ok(HttpResponse {
            status: status.as_u16(),
            body,
        });
    }

    Err(Error::Network(format!(
        "too many redirects (max: {MAX_REDIRECTS})"
    )))
}
