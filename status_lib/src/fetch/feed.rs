//! RSS/Atom feed discovery and parsing.

use crate::models::PartialStatus;
use crate::text::{purify_text, truncate_array};
use feed_rs::parser;
use regex::Regex;

const FEED_PATTERNS: &[&str] = &[
    "/feed.rss",
    "/feed.atom",
    "/rss",
    "/atom",
    "/feed",
    "/status.rss",
    "/status.atom",
];

const STATUS_WORDS: &str = r"^(Resolved|Operational|Degraded|Down|Investigating|Monitoring|Identified|Partial|Major|Minor)$";

pub fn build_feed_urls(status_url: &str) -> Vec<String> {
    let Ok(parsed) = url::Url::parse(status_url) else {
        return Vec::new();
    };
    let base_path = parsed.path().trim_end_matches('/');
    FEED_PATTERNS
        .iter()
        .map(|pattern| {
            let mut candidate = parsed.clone();
            let path = if base_path.is_empty() {
                (*pattern).to_string()
            } else {
                format!("{base_path}{pattern}")
            };
            candidate.set_path(&path);
            candidate.set_query(None);
            candidate.set_fragment(None);
            candidate.to_string()
        })
        .collect()
}

pub fn parse_feed(feed_body: &str, max_length: usize) -> PartialStatus {
    let feed = match parser::parse(feed_body.as_bytes()) {
        Ok(feed) => feed,
        Err(_) => {
            return PartialStatus {
                error: Some("Not a valid RSS or Atom feed".into()),
                ..PartialStatus::default()
            };
        }
    };

    let status_word = Regex::new(STATUS_WORDS).ok();
    let status_line = Regex::new(r"(?i)Status:\s*([A-Za-z]+)").ok();
    let affected = Regex::new(r"(?i)Affected components[^\n]*").ok();
    let operational_paren = Regex::new(r"(?i)\(Operational\)").ok();
    let blank_lines = Regex::new(r"\n{2,}").ok();
    let mut history = Vec::new();
    let mut latest_status = None;

    for entry in feed.entries {
        let title = entry
            .title
            .as_ref()
            .map(|value| value.content.clone())
            .unwrap_or_default();
        let mut description = entry
            .summary
            .as_ref()
            .map(|value| value.content.clone())
            .or_else(|| entry.content.as_ref().and_then(|value| value.body.clone()))
            .unwrap_or_default();
        description = strip_html(&description);

        if latest_status.is_none() {
            if let Some(regex) = &status_line {
                for source in [&description, &title] {
                    if let Some(captures) = regex.captures(source) {
                        let word = captures.get(1).map(|value| value.as_str()).unwrap_or("");
                        if status_word
                            .as_ref()
                            .is_some_and(|pattern| pattern.is_match(word))
                        {
                            latest_status = Some(word.to_string());
                            break;
                        }
                    }
                }
            }
        }

        let mut clean_description = description.clone();
        if let Some(regex) = &status_line {
            clean_description = regex.replace_all(&clean_description, "").to_string();
        }
        if let Some(regex) = &affected {
            clean_description = regex.replace_all(&clean_description, "").to_string();
        }
        if let Some(regex) = &operational_paren {
            clean_description = regex.replace_all(&clean_description, "").to_string();
        }
        clean_description = blank_lines
            .as_ref()
            .map(|regex| regex.replace_all(&clean_description, "\n").to_string())
            .unwrap_or(clean_description)
            .trim()
            .to_string();

        let mut item = title;
        if clean_description.len() > 10 {
            let snippet: String = clean_description.chars().take(500).collect();
            item.push_str(&format!(" - {snippet}"));
        }
        if let Some(published) = entry.published.or(entry.updated) {
            item.push_str(&format!(" ({published})"));
        }
        if item.len() >= 20 {
            if let Some(cleaned) = purify_text(Some(&item)) {
                history.push(cleaned);
            }
        }
    }

    if latest_status.is_none() {
        latest_status = infer_status_from_history(
            &history,
            feed.title.as_ref().map(|value| value.content.as_str()),
        );
    }

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

fn infer_status_from_history(history: &[String], feed_title: Option<&str>) -> Option<String> {
    if let Some(title) = feed_title {
        if title.len() < 50
            && Regex::new(r"(?i)^(operational|degraded|down|outage|incident|maintenance|all systems operational)$")
                .ok()
                .is_some_and(|regex| regex.is_match(title))
        {
            return Some(title.to_string());
        }
    }
    if history.is_empty() {
        return None;
    }
    let scheduled = history
        .iter()
        .filter(|item| {
            Regex::new(r"(?i)scheduled|maintenance")
                .ok()
                .is_some_and(|regex| regex.is_match(item))
                && Regex::new(r"(?i)incident|outage|degraded|down|investigating")
                    .ok()
                    .is_none_or(|regex| !regex.is_match(item))
        })
        .count();
    let resolved = history
        .iter()
        .filter(|item| {
            Regex::new(r"(?i)resolved|operational")
                .ok()
                .is_some_and(|regex| regex.is_match(item))
                && Regex::new(r"(?i)investigating|monitoring|identified")
                    .ok()
                    .is_none_or(|regex| !regex.is_match(item))
        })
        .count();
    if scheduled == history.len() || (resolved == history.len() && !history.is_empty()) {
        return Some("Operational".into());
    }
    let active = history.iter().any(|item| {
        Regex::new(r"(?i)investigating|monitoring|identified|degraded|down|outage")
            .ok()
            .is_some_and(|regex| regex.is_match(item))
            && Regex::new(r"(?i)resolved|operational")
                .ok()
                .is_none_or(|regex| !regex.is_match(item))
    });
    if active {
        Some("See recent incidents".into())
    } else {
        Some("Operational".into())
    }
}

fn strip_html(text: &str) -> String {
    if !text.contains('<') {
        return text.to_string();
    }
    scraper::Html::parse_fragment(text)
        .root_element()
        .text()
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_feed_candidates() {
        let urls = build_feed_urls("https://status.example.com");
        assert!(urls.iter().any(|url| url.ends_with("/feed.rss")));
    }

    #[test]
    fn parses_rss_fixture() {
        let rss = r#"<?xml version="1.0"?>
        <rss version="2.0"><channel>
          <title>Example Status</title>
          <item>
            <title>API Outage</title>
            <description>Status: Investigating - API latency</description>
            <pubDate>Mon, 01 Jan 2024 00:00:00 GMT</pubDate>
          </item>
        </channel></rss>"#;
        let parsed = parse_feed(rss, 10_000);
        assert_eq!(parsed.latest_status.as_deref(), Some("Investigating"));
        assert!(!parsed.history.is_empty());
    }
}
