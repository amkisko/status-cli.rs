//! Live status fetch pipeline: Statuspage → incident.io → RSS/Atom → HTML.

mod feed;
#[cfg(feature = "html")]
mod html;
mod http;
mod incident_io;
mod merge;
mod statuspage;

use crate::error::{Error, Result};
use crate::models::{FetchResult, PartialStatus};
use http::DEFAULT_TIMEOUT_SECS;

pub use http::MAX_RESPONSE_SIZE;

pub fn fetch_status(status_url: &str, max_length: usize, timeout_secs: Option<u64>) -> FetchResult {
    let timeout = timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS);
    match fetch_status_inner(status_url, max_length, timeout) {
        Ok(result) => result,
        Err(error) => FetchResult {
            status_url: status_url.to_string(),
            error: Some(match error {
                Error::ResponseTooLarge { actual, limit } => {
                    format!("Response size limit exceeded: {actual} bytes (max {limit})")
                }
                other => format!("Error fetching status: {other}"),
            }),
            latest_status: None,
            history: Vec::new(),
            messages: Vec::new(),
            api_url: None,
            feed_url: None,
            history_url: None,
            extracted_at: None,
            http_status_code: None,
        },
    }
}

fn fetch_status_inner(status_url: &str, max_length: usize, timeout: u64) -> Result<FetchResult> {
    let mut api_url = None;
    let mut api_info = None;

    if let Some((url, partial)) = statuspage::try_fetch(status_url, max_length, timeout) {
        api_url = Some(url);
        api_info = Some(partial);
    }

    if api_info
        .as_ref()
        .is_none_or(|info| info.latest_status.is_none())
    {
        if let Some((url, partial)) = incident_io::try_fetch(status_url, max_length, timeout) {
            api_url = Some(url);
            api_info = Some(merge_partial(api_info, partial));
        }
    } else if let Some((_, partial)) = incident_io::try_fetch(status_url, max_length, timeout) {
        api_info = Some(merge_partial(api_info, partial));
    }

    // Statuspage/incident.io already give a reliable overall status; skip slow feed probing.
    let need_feed = api_info
        .as_ref()
        .is_none_or(|info| info.latest_status.is_none());

    let mut feed_info = None;
    let mut successful_feed_url = None;
    if need_feed {
        for feed_url in feed::build_feed_urls(status_url) {
            match http::fetch(
                &feed_url,
                "application/rss+xml,application/atom+xml,application/xml,text/xml,*/*;q=0.9",
                timeout,
            ) {
                Ok(response) if (200..300).contains(&response.status) => {
                    let parsed = feed::parse_feed(&response.body, max_length);
                    if parsed.history.is_empty() && parsed.latest_status.is_none() {
                        continue;
                    }
                    successful_feed_url = Some(feed_url);
                    feed_info = Some(parsed);
                    break;
                }
                _ => continue,
            }
        }
    }

    #[cfg(feature = "html")]
    let (history_url, main_info, history_info) = {
        let main_info = fetch_html_page(status_url, max_length, timeout, feed_info.is_some());
        let history_url = html::build_history_url(status_url);
        let history_info = history_url.as_ref().and_then(|url| {
            let response = http::fetch(
                url,
                "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
                timeout,
            )
            .ok()?;
            if !(200..300).contains(&response.status) {
                return None;
            }
            html::extract_from_html(&response.body, max_length, true).ok()
        });
        (history_url, main_info, history_info)
    };

    #[cfg(not(feature = "html"))]
    let (history_url, main_info, history_info) = (None, None, None);

    Ok(merge::merge_results(
        status_url,
        api_url,
        successful_feed_url,
        history_url,
        api_info,
        feed_info,
        main_info,
        history_info,
    ))
}

fn merge_partial(existing: Option<PartialStatus>, incoming: PartialStatus) -> PartialStatus {
    let Some(mut base) = existing else {
        return incoming;
    };
    if base.latest_status.is_none() {
        base.latest_status = incoming.latest_status;
    }
    if base.history.is_empty() {
        base.history = incoming.history;
    } else if !incoming.history.is_empty() {
        base.history.extend(incoming.history);
    }
    if base.messages.is_empty() {
        base.messages = incoming.messages;
    }
    base
}

#[cfg(feature = "html")]
fn fetch_html_page(
    status_url: &str,
    max_length: usize,
    timeout: u64,
    have_feed: bool,
) -> Option<PartialStatus> {
    match http::fetch(
        status_url,
        "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
        timeout,
    ) {
        Ok(response) => {
            let status = response.status;
            if !(200..300).contains(&status) {
                Some(PartialStatus {
                    error: Some(format!("Failed to fetch: {status}")),
                    http_status_code: Some(status),
                    ..PartialStatus::default()
                })
            } else {
                match html::extract_from_html(&response.body, max_length, false) {
                    Ok(mut partial) => {
                        partial.http_status_code = Some(status);
                        Some(partial)
                    }
                    Err(error) => Some(PartialStatus {
                        error: Some(error),
                        http_status_code: Some(status),
                        ..PartialStatus::default()
                    }),
                }
            }
        }
        Err(error) => {
            if have_feed {
                None
            } else {
                Some(PartialStatus {
                    error: Some(error.to_string()),
                    ..PartialStatus::default()
                })
            }
        }
    }
}

#[cfg(test)]
mod live_tests {
    use super::*;

    #[test]
    #[ignore = "live network; run with --ignored"]
    fn statuspage_github_is_stable() {
        let result = fetch_status("https://www.githubstatus.com", 10_000, Some(15));
        assert!(result.error.is_none(), "error={:?}", result.error);
        assert!(result.latest_status.is_some(), "{result:?}");
        assert!(
            result
                .api_url
                .as_deref()
                .is_some_and(|url| url.contains("/api/v2/status.json")),
            "expected statuspage api, got {:?}",
            result.api_url
        );
    }

    #[test]
    #[ignore = "live network; run with --ignored"]
    fn statuspage_reports_degraded_when_present() {
        for url in [
            "https://www.cloudflarestatus.com",
            "https://status.twilio.com",
            "https://status.zoom.us",
        ] {
            let result = fetch_status(url, 10_000, Some(15));
            assert!(result.latest_status.is_some(), "{url} => {result:?}");
            assert!(
                result
                    .api_url
                    .as_deref()
                    .is_some_and(|api| api.contains("/api/v2/status.json")),
                "{url} missing statuspage api: {:?}",
                result.api_url
            );
        }
    }

    #[test]
    #[ignore = "live network; run with --ignored"]
    fn incident_io_openai_works() {
        let result = fetch_status("https://status.openai.com", 10_000, Some(15));
        assert!(result.latest_status.is_some(), "{result:?}");
        assert!(result.error.is_none(), "error={:?}", result.error);
    }
}
