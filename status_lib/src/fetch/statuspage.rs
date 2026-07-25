//! Atlassian Statuspage public JSON API (`/api/v2/*`).

use crate::models::PartialStatus;
use crate::text::{purify_text, truncate_array};
use serde_json::Value;

use super::http;

pub fn build_status_url(status_url: &str) -> Option<String> {
    api_url(status_url, "/api/v2/status.json")
}

pub fn build_summary_url(status_url: &str) -> Option<String> {
    api_url(status_url, "/api/v2/summary.json")
}

fn api_url(status_url: &str, path: &str) -> Option<String> {
    let mut parsed = url::Url::parse(status_url).ok()?;
    if parsed.scheme() != "http" && parsed.scheme() != "https" {
        return None;
    }
    parsed.set_path(path);
    parsed.set_query(None);
    parsed.set_fragment(None);
    Some(parsed.to_string())
}

pub fn try_fetch(
    status_url: &str,
    max_length: usize,
    timeout: u64,
) -> Option<(String, PartialStatus)> {
    let status_api = build_status_url(status_url)?;
    let response = http::fetch(&status_api, "application/json", timeout).ok()?;
    if !(200..300).contains(&response.status) || !looks_like_json(&response.body) {
        return None;
    }
    let mut partial = parse_status_json(&response.body)?;
    if let Some(summary_api) = build_summary_url(status_url) {
        if let Ok(summary_response) = http::fetch(&summary_api, "application/json", timeout) {
            if (200..300).contains(&summary_response.status)
                && looks_like_json(&summary_response.body)
            {
                if let Some(summary) = parse_summary_json(&summary_response.body, max_length) {
                    if partial.latest_status.is_none() {
                        partial.latest_status = summary.latest_status;
                    }
                    if partial.history.is_empty() {
                        partial.history = summary.history;
                    }
                    if partial.messages.is_empty() {
                        partial.messages = summary.messages;
                    }
                }
            }
        }
    }
    if partial.latest_status.is_none() && partial.history.is_empty() {
        return None;
    }
    Some((status_api, partial))
}

pub fn parse_status_json(body: &str) -> Option<PartialStatus> {
    let data: Value = serde_json::from_str(body).ok()?;
    let status = data.get("status")?;
    let description = status
        .get("description")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string);
    let indicator = status
        .get("indicator")
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty());
    let latest_status = description.or_else(|| indicator.map(indicator_to_status))?;
    Some(PartialStatus {
        latest_status: Some(latest_status),
        ..PartialStatus::default()
    })
}

pub fn parse_summary_json(body: &str, max_length: usize) -> Option<PartialStatus> {
    let data: Value = serde_json::from_str(body).ok()?;
    let mut history = Vec::new();
    for incident in data
        .get("incidents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .take(20)
    {
        let name = string_field(&incident, "name").unwrap_or_else(|| "Incident".into());
        let status = string_field(&incident, "status").unwrap_or_else(|| "unknown".into());
        let impact = string_field(&incident, "impact").unwrap_or_default();
        let mut item = format!("{name} - Status: {status}");
        if !impact.is_empty() {
            item.push_str(&format!(" - Impact: {impact}"));
        }
        if let Some(cleaned) = purify_text(Some(&item)) {
            history.push(cleaned);
        }
    }
    for maintenance in data
        .get("scheduled_maintenances")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default()
        .into_iter()
        .take(10)
    {
        let name =
            string_field(&maintenance, "name").unwrap_or_else(|| "Scheduled Maintenance".into());
        let status = string_field(&maintenance, "status").unwrap_or_else(|| "scheduled".into());
        if let Some(cleaned) = purify_text(Some(&format!("{name} - Status: {status}"))) {
            history.push(cleaned);
        }
    }
    if history.join("\n").len() > max_length {
        history = truncate_array(&history, max_length);
    }
    let latest_status = data.get("status").and_then(|status| {
        status
            .get("description")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|text| !text.is_empty())
            .map(str::to_string)
            .or_else(|| {
                status
                    .get("indicator")
                    .and_then(Value::as_str)
                    .map(indicator_to_status)
            })
    });
    Some(PartialStatus {
        latest_status,
        history,
        ..PartialStatus::default()
    })
}

pub fn looks_like_json(body: &str) -> bool {
    let trimmed = body.trim_start();
    trimmed.starts_with('{') || trimmed.starts_with('[')
}

fn indicator_to_status(indicator: &str) -> String {
    match indicator.to_ascii_lowercase().as_str() {
        "none" => "Operational".into(),
        "minor" => "Minor Service Outage".into(),
        "major" => "Major Service Outage".into(),
        "critical" => "Critical Service Outage".into(),
        "maintenance" => "Under Maintenance".into(),
        other => other.to_string(),
    }
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_operational_status() {
        let body = include_str!("../../tests/fixtures/statuspage_status_operational.json");
        let parsed = parse_status_json(body).expect("status");
        assert_eq!(
            parsed.latest_status.as_deref(),
            Some("All Systems Operational")
        );
    }

    #[test]
    fn parses_minor_status() {
        let body = include_str!("../../tests/fixtures/statuspage_status_minor.json");
        let parsed = parse_status_json(body).expect("status");
        assert_eq!(
            parsed.latest_status.as_deref(),
            Some("Minor Service Outage")
        );
    }

    #[test]
    fn parses_summary_history() {
        let body = include_str!("../../tests/fixtures/statuspage_summary.json");
        let parsed = parse_summary_json(body, 10_000).expect("summary");
        assert!(parsed
            .history
            .iter()
            .any(|item| item.contains("API latency")));
        assert_eq!(
            parsed.latest_status.as_deref(),
            Some("Minor Service Outage")
        );
    }

    #[test]
    fn builds_api_paths() {
        assert_eq!(
            build_status_url("https://www.githubstatus.com"),
            Some("https://www.githubstatus.com/api/v2/status.json".into())
        );
    }
}
