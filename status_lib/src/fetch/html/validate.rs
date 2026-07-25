//! HTML response validation before extraction.

use regex::Regex;
use scraper::Html;

pub fn validate_html(body: &str) -> Result<(), String> {
    for pattern in [
        r"(?is)checking your browser.*?before accessing",
        r"(?is)ddos protection.*?checking",
        r"(?is)access denied.*?cloudflare",
        r"(?is)please wait.*?cloudflare",
        r"(?is)captcha.*?verification",
        r"(?is)rate limit.*?exceeded",
        r"(?is)blocked.*?security",
    ] {
        if Regex::new(pattern)
            .ok()
            .is_some_and(|regex| regex.is_match(body))
        {
            return Err("Response appears to be a crawler protection page".into());
        }
    }

    let trimmed = body.trim_start();
    if !(trimmed.starts_with("<!DOCTYPE")
        || trimmed.starts_with("<html")
        || trimmed.starts_with("<HTML")
        || body.contains("<html"))
    {
        return Err("Response does not appear to be HTML".into());
    }

    let document = Html::parse_document(body);
    let text_len = document
        .root_element()
        .text()
        .collect::<String>()
        .trim()
        .len();
    let js_rendered = [
        r#"(?is)<div[^>]*id=["']root["'][^>]*>\s*</div>"#,
        r#"(?is)<div[^>]*id=["']app["'][^>]*>\s*</div>"#,
        r"(?i)You need to enable JavaScript",
        r"(?is)<noscript>.*?enable.*?javascript",
    ]
    .iter()
    .any(|pattern| {
        Regex::new(pattern)
            .ok()
            .is_some_and(|regex| regex.is_match(body))
    });

    let min_length = if js_rendered { 20 } else { 50 };
    if text_len < min_length {
        if js_rendered {
            return Err(
                "HTML response appears to be a JavaScript-rendered page with no server-side content"
                    .into(),
            );
        }
        return Err("HTML response appears to be empty or too short".into());
    }
    Ok(())
}
