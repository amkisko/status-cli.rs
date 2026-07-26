//! incident.io proxy API parsing for known status hosts.

use crate::models::PartialStatus;
use crate::text::{purify_text, truncate_array};
#[cfg(not(feature = "html"))]
use regex::Regex;
#[cfg(feature = "html")]
use scraper::Html;
use serde_json::Value;

use super::http;
use super::statuspage::looks_like_json;

const INCIDENT_IO_HOSTS: &[&str] = &[
    "status.openai.com",
    "status.notion.so",
    "status.zapier.com",
    "status.buffer.com",
];

pub fn might_be_incident_io(status_url: &str) -> bool {
    url::Url::parse(status_url)
        .ok()
        .and_then(|parsed| parsed.host_str().map(|host| host.to_string()))
        .is_some_and(|host| INCIDENT_IO_HOSTS.contains(&host.as_str()))
}

pub fn build_api_url(status_url: &str) -> Option<String> {
    let mut parsed = url::Url::parse(status_url).ok()?;
    let host = parsed.host_str()?.to_string();
    parsed.set_path(&format!("/proxy/{host}"));
    parsed.set_query(None);
    parsed.set_fragment(None);
    Some(parsed.to_string())
}

pub fn try_fetch(
    status_url: &str,
    max_length: usize,
    timeout: u64,
) -> Option<(String, PartialStatus)> {
    if !might_be_incident_io(status_url) {
        return None;
    }
    let api_url = build_api_url(status_url)?;
    let response = http::fetch(&api_url, "application/json", timeout).ok()?;
    if !(200..300).contains(&response.status) || !looks_like_json(&response.body) {
        return None;
    }
    let partial = parse_incident_io_api(&response.body, max_length);
    if partial.error.is_some() || (partial.latest_status.is_none() && partial.history.is_empty()) {
        return None;
    }
    Some((api_url, partial))
}

pub fn parse_incident_io_api(json_body: &str, max_length: usize) -> PartialStatus {
    let data: Value = match serde_json::from_str(json_body) {
        Ok(value) => value,
        Err(error) => {
            return PartialStatus {
                error: Some(format!("Error parsing API JSON: {error}")),
                ..PartialStatus::default()
            };
        }
    };
    let summary = data.get("summary").cloned().unwrap_or(Value::Null);
    let ongoing = summary
        .get("ongoing_incidents")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let maintenances = summary
        .get("scheduled_maintenances")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let components = summary
        .get("components")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let affected = summary
        .get("affected_components")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut history = Vec::new();
    for incident in &ongoing {
        if let Some(cleaned) = purify_text(Some(&format_incident(incident))) {
            history.push(cleaned);
        }
    }
    for maintenance in &maintenances {
        if let Some(cleaned) = purify_text(Some(&format_maintenance(maintenance))) {
            history.push(cleaned);
        }
    }
    let latest_status = Some(overall_status(
        &ongoing,
        &maintenances,
        &components,
        &affected,
    ));
    if history.join("\n").len() > max_length {
        history = truncate_array(&history, max_length);
    }
    PartialStatus {
        latest_status,
        history,
        messages: Vec::new(),
        error: None,
        http_status_code: None,
    }
}

fn overall_status(
    ongoing: &[Value],
    maintenances: &[Value],
    components: &[Value],
    affected: &[Value],
) -> String {
    if let Some(worst) = worst_component_status(affected) {
        return component_status_label(&worst);
    }
    if !ongoing.is_empty() {
        return "Degraded Performance".into();
    }
    if !maintenances.is_empty() {
        return "Under Maintenance".into();
    }
    let all_operational = components.iter().all(|component| {
        let status = string_field(component, "status")
            .or_else(|| string_field(component, "operational_status"));
        match status {
            Some(value) => value.to_lowercase().contains("operational"),
            None => true,
        }
    });
    if all_operational {
        return "Operational".into();
    }
    "Degraded Performance".into()
}

fn worst_component_status(affected: &[Value]) -> Option<String> {
    let ranks = [
        "major_outage",
        "partial_outage",
        "degraded_performance",
        "under_maintenance",
    ];
    for rank in ranks {
        if affected.iter().any(|component| {
            string_field(component, "status")
                .is_some_and(|status| status.eq_ignore_ascii_case(rank))
        }) {
            return Some(rank.into());
        }
    }
    None
}

fn component_status_label(status: &str) -> String {
    match status.to_ascii_lowercase().as_str() {
        "major_outage" => "Major Outage".into(),
        "partial_outage" => "Partial Outage".into(),
        "degraded_performance" => "Degraded Performance".into(),
        "under_maintenance" => "Under Maintenance".into(),
        "operational" => "Operational".into(),
        other => other.replace('_', " "),
    }
}

fn format_incident(incident: &Value) -> String {
    let title = string_field(incident, "name").unwrap_or_else(|| "Ongoing Incident".into());
    let status = string_field(incident, "status").unwrap_or_else(|| "Investigating".into());
    let description = clean_html(string_field(incident, "description").unwrap_or_default());
    let mut item = format!("{title} - Status: {status}");
    if !description.is_empty() {
        let snippet: String = description.chars().take(300).collect();
        item.push_str(&format!(" - {snippet}"));
    }
    item
}

fn format_maintenance(maintenance: &Value) -> String {
    let title = string_field(maintenance, "name").unwrap_or_else(|| "Scheduled Maintenance".into());
    let status = string_field(maintenance, "status").unwrap_or_else(|| "Scheduled".into());
    let description = clean_html(string_field(maintenance, "description").unwrap_or_default());
    let scheduled_for = string_field(maintenance, "scheduled_for").unwrap_or_default();
    let scheduled_until = string_field(maintenance, "scheduled_until").unwrap_or_default();
    let mut item = format!("{title} - Status: {status}");
    if !scheduled_for.is_empty() {
        item.push_str(&format!(" - Scheduled: {scheduled_for}"));
    }
    if !scheduled_until.is_empty() {
        item.push_str(&format!(" until {scheduled_until}"));
    }
    if !description.is_empty() {
        let snippet: String = description.chars().take(300).collect();
        item.push_str(&format!(" - {snippet}"));
    }
    item
}

fn string_field(value: &Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|text| !text.is_empty())
        .map(str::to_string)
}

#[cfg(feature = "html")]
fn clean_html(text: String) -> String {
    if !text.contains('<') {
        return text;
    }
    Html::parse_fragment(&text)
        .root_element()
        .text()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(not(feature = "html"))]
fn clean_html(text: String) -> String {
    let without_tags = Regex::new(r"<[^>]*>")
        .ok()
        .map(|regex| regex.replace_all(&text, " ").into_owned())
        .unwrap_or(text);
    without_tags
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_known_host() {
        assert!(might_be_incident_io("https://status.openai.com"));
        assert!(!might_be_incident_io("https://www.githubstatus.com"));
    }

    #[test]
    fn parses_operational_summary() {
        let json = r#"{
          "summary": {
            "ongoing_incidents": [],
            "scheduled_maintenances": [],
            "components": [{"name":"API","status":"Operational"}]
          }
        }"#;
        let parsed = parse_incident_io_api(json, 10_000);
        assert_eq!(parsed.latest_status.as_deref(), Some("Operational"));
    }

    #[test]
    fn maps_affected_components_not_incident_lifecycle() {
        let body = include_str!("../../tests/fixtures/incident_io_degraded.json");
        let parsed = parse_incident_io_api(body, 10_000);
        assert_eq!(
            parsed.latest_status.as_deref(),
            Some("Degraded Performance")
        );
        assert!(parsed
            .history
            .iter()
            .any(|item| item.contains("Threads comments")));
    }

    #[test]
    fn strips_html_from_incident_descriptions() {
        let json = r#"{
          "summary": {
            "ongoing_incidents": [{
              "name":"API incident",
              "status":"Investigating",
              "description":"<p>API latency is elevated</p>"
            }],
            "scheduled_maintenances": [],
            "components": []
          }
        }"#;
        let parsed = parse_incident_io_api(json, 10_000);
        assert!(parsed
            .history
            .iter()
            .any(|item| item.contains("API latency is elevated")));
        assert!(parsed.history.iter().all(|item| !item.contains('<')));
    }
}
