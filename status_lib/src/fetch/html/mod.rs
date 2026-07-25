//! HTML status page extraction heuristics.

mod extract;
mod validate;

use crate::models::PartialStatus;
use crate::text::{truncate_array, truncate_text};
use scraper::Html;

pub fn build_history_url(status_url: &str) -> Option<String> {
    let mut parsed = url::Url::parse(status_url).ok()?;
    if parsed.path().ends_with("/history") {
        return None;
    }
    let base = parsed.path().trim_end_matches('/');
    let history_path = if base.is_empty() {
        "/history".to_string()
    } else {
        format!("{base}/history")
    };
    parsed.set_path(&history_path);
    parsed.set_query(None);
    parsed.set_fragment(None);
    Some(parsed.to_string())
}

pub fn extract_from_html(
    body: &str,
    max_length: usize,
    history_only: bool,
) -> Result<PartialStatus, String> {
    validate::validate_html(body)?;
    let document = Html::parse_document(body);

    if history_only {
        return Ok(PartialStatus {
            latest_status: None,
            history: extract::extract_history(&document),
            messages: Vec::new(),
            error: None,
            http_status_code: None,
        });
    }

    let mut latest_status = extract::extract_latest_status(&document);
    let mut history = extract::extract_history(&document);
    let mut messages = extract::extract_messages(&document);

    let status_text = latest_status.clone().unwrap_or_default();
    let history_text = history.join("\n");
    let messages_text = messages.join("\n");
    let total_len = status_text.len() + history_text.len() + messages_text.len();

    if total_len > max_length {
        if !status_text.is_empty() {
            let status_max = ((max_length as f64) * 0.3) as usize;
            latest_status = latest_status.map(|text| truncate_text(&text, status_max));
        }
        if !history_text.is_empty() {
            let history_max = ((max_length as f64) * 0.5) as usize;
            history = truncate_array(&history, history_max);
        }
        if !messages_text.is_empty() {
            let messages_max = ((max_length as f64) * 0.2) as usize;
            messages = truncate_array(&messages, messages_max);
        }
    }

    Ok(PartialStatus {
        latest_status,
        history,
        messages,
        error: None,
        http_status_code: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_operational_banner() {
        let html = r#"<!DOCTYPE html><html><body>
          <main><div class="status-indicator">All Systems Operational</div>
          <div class="incident">Resolved incident about API on 2024-01-01 with details</div>
          </main></body></html>"#;
        let parsed = extract_from_html(html, 10_000, false).expect("html");
        assert!(parsed
            .latest_status
            .as_deref()
            .unwrap_or("")
            .to_lowercase()
            .contains("operational"));
        assert!(!parsed.history.is_empty());
    }

    #[test]
    fn builds_history_url() {
        assert_eq!(
            build_history_url("https://status.example.com"),
            Some("https://status.example.com/history".into())
        );
    }
}
