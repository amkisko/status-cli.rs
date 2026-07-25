//! Merge partial fetch results with Ruby-compatible priority rules.

use crate::models::{FetchResult, PartialStatus};
use chrono::Utc;

#[allow(clippy::too_many_arguments)]
pub fn merge_results(
    status_url: &str,
    api_url: Option<String>,
    feed_url: Option<String>,
    history_url: Option<String>,
    api_info: Option<PartialStatus>,
    feed_info: Option<PartialStatus>,
    main_info: Option<PartialStatus>,
    history_info: Option<PartialStatus>,
) -> FetchResult {
    let mut history = Vec::new();
    if let Some(api) = &api_info {
        history.extend(api.history.clone());
    }
    if let Some(feed) = &feed_info {
        history.extend(feed.history.clone());
    }
    if let Some(main) = &main_info {
        history.extend(main.history.clone());
    }
    if let Some(page) = &history_info {
        history.extend(page.history.clone());
    }

    let mut seen = std::collections::HashSet::new();
    history.retain(|item| {
        let key = item.chars().take(100).collect::<String>();
        seen.insert(key)
    });
    history.truncate(20);

    let latest_status = api_info
        .as_ref()
        .and_then(|info| info.latest_status.clone())
        .or_else(|| {
            main_info
                .as_ref()
                .and_then(|info| info.latest_status.clone())
        })
        .or_else(|| {
            feed_info
                .as_ref()
                .and_then(|info| info.latest_status.clone())
        });

    let messages = api_info
        .as_ref()
        .map(|info| info.messages.clone())
        .filter(|items| !items.is_empty())
        .or_else(|| {
            feed_info
                .as_ref()
                .map(|info| info.messages.clone())
                .filter(|items| !items.is_empty())
        })
        .or_else(|| main_info.as_ref().map(|info| info.messages.clone()))
        .unwrap_or_default();

    let http_status_code = main_info.as_ref().and_then(|info| info.http_status_code);
    let error = resolve_error(&api_info, &feed_info, &main_info);

    FetchResult {
        status_url: status_url.to_string(),
        api_url,
        feed_url,
        history_url,
        latest_status,
        history,
        messages,
        extracted_at: Some(Utc::now().to_rfc3339()),
        error,
        http_status_code,
    }
}

fn resolve_error(
    api_info: &Option<PartialStatus>,
    feed_info: &Option<PartialStatus>,
    main_info: &Option<PartialStatus>,
) -> Option<String> {
    let has_useful_alt = |info: &Option<PartialStatus>| {
        info.as_ref()
            .is_some_and(|partial| !partial.history.is_empty() || partial.latest_status.is_some())
    };
    let has_alt = has_useful_alt(feed_info) || has_useful_alt(api_info);

    if let Some(main) = main_info {
        if let Some(error) = &main.error {
            if !error.is_empty() {
                if has_alt && is_suppressible_html_error(error) {
                    return None;
                }
                if !has_alt {
                    return Some(error.clone());
                }
            }
        }
    }

    if main_info
        .as_ref()
        .and_then(|info| info.latest_status.as_ref())
        .is_some()
        || has_useful_alt(api_info)
        || has_useful_alt(feed_info)
    {
        return None;
    }

    if let Some(feed) = feed_info {
        if let Some(error) = &feed.error {
            if !error.is_empty()
                && feed.history.is_empty()
                && feed.latest_status.is_none()
                && !error.contains("Not a valid RSS or Atom feed")
            {
                return Some(error.clone());
            }
        }
    }

    if let Some(api) = api_info {
        if let Some(error) = &api.error {
            if !error.is_empty() && api.history.is_empty() && api.latest_status.is_none() {
                return Some(error.clone());
            }
        }
    }
    None
}

fn is_suppressible_html_error(error: &str) -> bool {
    error.contains("JavaScript-rendered")
        || error.contains("crawler protection")
        || error.contains("empty or too short")
        || error.contains("does not appear to be HTML")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prefers_api_status() {
        let result = merge_results(
            "https://status.example.com",
            Some("https://status.example.com/proxy/x".into()),
            None,
            None,
            Some(PartialStatus {
                latest_status: Some("Investigating".into()),
                history: vec!["incident".into()],
                ..PartialStatus::default()
            }),
            Some(PartialStatus {
                latest_status: Some("Operational".into()),
                ..PartialStatus::default()
            }),
            Some(PartialStatus {
                latest_status: Some("Degraded".into()),
                http_status_code: Some(200),
                ..PartialStatus::default()
            }),
            None,
        );
        assert_eq!(result.latest_status.as_deref(), Some("Investigating"));
        assert_eq!(result.http_status_code, Some(200));
    }

    #[test]
    fn suppresses_crawler_error_when_api_succeeds() {
        let result = merge_results(
            "https://www.cloudflarestatus.com",
            Some("https://www.cloudflarestatus.com/api/v2/status.json".into()),
            None,
            None,
            Some(PartialStatus {
                latest_status: Some("Minor Service Outage".into()),
                ..PartialStatus::default()
            }),
            None,
            Some(PartialStatus {
                error: Some("Response appears to be a crawler protection page".into()),
                http_status_code: Some(200),
                ..PartialStatus::default()
            }),
            None,
        );
        assert_eq!(
            result.latest_status.as_deref(),
            Some("Minor Service Outage")
        );
        assert!(result.error.is_none());
    }
}
