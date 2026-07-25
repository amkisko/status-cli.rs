//! Status, history, and message extraction from HTML documents.

use crate::text::purify_text;
use regex::Regex;
use scraper::{Html, Selector};

pub fn extract_latest_status(document: &Html) -> Option<String> {
    let all_text = document.root_element().text().collect::<String>();
    if let Some(status) = status_from_components(&all_text) {
        return Some(status);
    }

    for selector in [
        ".status-indicator",
        ".status",
        "[class*='status-indicator']",
        "[data-status]",
        ".component-status",
        ".operational-status",
        ".current-status",
        ".page-status",
        "h1[class*='status']",
        "h2[class*='status']",
    ] {
        if let Some(text) = first_text(document, selector) {
            if text.len() >= 3 && text.len() <= 300 && status_keyword(&text) {
                if let Some(cleaned) = filter_status_text(purify_text(Some(&text))) {
                    return Some(cleaned);
                }
            }
        }
    }

    for heading in texts(
        document,
        "main h1, main h2, main h3, article h1, article h2",
    )
    .into_iter()
    .take(5)
    {
        if heading.len() > 3 && heading.len() <= 200 && status_keyword(&heading) {
            if let Some(cleaned) = filter_status_text(purify_text(Some(&heading))) {
                return Some(cleaned);
            }
        }
    }

    if let Some(title) = first_text(document, "title") {
        if title.len() < 100 && status_keyword(&title) {
            return filter_status_text(purify_text(Some(&title)));
        }
    }
    None
}

pub fn extract_history(document: &Html) -> Vec<String> {
    for selector in [
        ".incident",
        ".incident-list",
        ".history",
        ".timeline",
        ".status-update",
        "[class*='incident']",
        "[class*='history']",
        ".incident-item",
        ".history-item",
    ] {
        let mut history = Vec::new();
        for text in texts(document, selector).into_iter().take(15) {
            if text.len() >= 20 {
                if let Some(cleaned) = purify_text(Some(&text)) {
                    if cleaned.len() >= 20 {
                        history.push(cleaned);
                    }
                }
            }
        }
        if !history.is_empty() {
            return history.into_iter().take(20).collect();
        }
    }
    Vec::new()
}

pub fn extract_messages(document: &Html) -> Vec<String> {
    for selector in [
        ".message",
        ".announcement",
        ".alert",
        "[class*='message']",
        "[class*='announcement']",
        ".status-message",
    ] {
        let mut messages = Vec::new();
        for text in texts(document, selector).into_iter().take(5) {
            if text.len() >= 10 {
                if let Some(cleaned) = purify_text(Some(&text)) {
                    messages.push(cleaned);
                }
            }
        }
        if !messages.is_empty() {
            return messages.into_iter().take(5).collect();
        }
    }
    Vec::new()
}

fn status_from_components(all_text: &str) -> Option<String> {
    let pattern = Regex::new(
        r"(?i)([A-Za-z0-9\s-]+)\s+[?•]\s+(Operational|Degraded|Down|Outage|Maintenance)",
    )
    .ok()?;
    let matches: Vec<_> = pattern.captures_iter(all_text).collect();
    if matches.is_empty() {
        return None;
    }
    let non_operational: Vec<_> = matches
        .iter()
        .filter(|capture| {
            !capture
                .get(2)
                .map(|value| value.as_str())
                .unwrap_or("")
                .eq_ignore_ascii_case("operational")
        })
        .collect();
    if non_operational.is_empty() {
        return Some("Operational".into());
    }
    if Regex::new(r"(?i)down|outage").ok().is_some_and(|regex| {
        non_operational
            .iter()
            .any(|capture| regex.is_match(capture.get(2).map(|value| value.as_str()).unwrap_or("")))
    }) {
        return Some("Partial Outage".into());
    }
    Some("Degraded Performance".into())
}

fn filter_status_text(text: Option<String>) -> Option<String> {
    let text = text?;
    if text.split_whitespace().count() > 20 {
        return None;
    }
    if Regex::new(r"(?i)^(Support|Log in|Sign up|Subscribe|Home|About)")
        .ok()
        .is_some_and(|regex| regex.is_match(&text))
    {
        return None;
    }
    Some(text)
}

fn status_keyword(text: &str) -> bool {
    Regex::new(r"(?i)operational|degraded|down|outage|incident|maintenance|all systems|resolved|investigating|partial|major|minor")
        .ok()
        .is_some_and(|regex| regex.is_match(text))
}

fn first_text(document: &Html, selector: &str) -> Option<String> {
    texts(document, selector).into_iter().next()
}

fn texts(document: &Html, selector: &str) -> Vec<String> {
    let Ok(parsed) = Selector::parse(selector) else {
        return Vec::new();
    };
    document
        .select(&parsed)
        .map(|element| element.text().collect::<String>().trim().to_string())
        .filter(|text| !text.is_empty())
        .collect()
}
