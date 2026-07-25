//! Text purification and truncation helpers.

pub fn purify_text(text: Option<&str>) -> Option<String> {
    let mut text = text?.to_string();
    let patterns = [
        r"(?is)Notifications.*?signed in.*?reload",
        r"(?is)You must be signed in.*?reload",
        r"(?is)There was an error.*?reload",
        r"(?is)Please reload this page.*?",
        r"(?is)Loading.*?",
    ];
    for pattern in patterns {
        if let Ok(regex) = regex::Regex::new(pattern) {
            text = regex.replace_all(&text, "").to_string();
        }
    }
    if text.len() < 50 {
        if let Ok(regex) = regex::Regex::new(r"(?i)Cookie|Privacy|Accept|Decline") {
            text = regex.replace_all(&text, "").to_string();
        }
    }
    text = regex::Regex::new(r"\n{3,}")
        .ok()
        .map(|regex| regex.replace_all(&text, "\n\n").to_string())
        .unwrap_or(text);
    text = regex::Regex::new(r"[ \t]{2,}")
        .ok()
        .map(|regex| regex.replace_all(&text, " ").to_string())
        .unwrap_or(text);
    let text = text.trim().to_string();
    if text.is_empty() {
        None
    } else {
        Some(text)
    }
}

pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        return text.to_string();
    }
    let truncated = &text[..max_length];
    let cut = truncated
        .rfind(['.', '!', '?'])
        .or_else(|| truncated.rfind("\n\n"))
        .or_else(|| truncated.rfind('\n'))
        .unwrap_or(max_length);
    format!("{}...", truncated[..cut].trim())
}

pub fn truncate_array(items: &[String], max_total_length: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current_length = 0usize;
    for item in items {
        let item_length = item.len();
        if current_length + item_length <= max_total_length {
            result.push(item.clone());
            current_length += item_length;
            continue;
        }
        let remaining = max_total_length.saturating_sub(current_length);
        if remaining > 100 {
            result.push(truncate_text(item, remaining));
        }
        break;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn purify_strips_noise() {
        let cleaned = purify_text(Some("Hello   world\n\n\nthere"));
        assert_eq!(cleaned.as_deref(), Some("Hello world\n\nthere"));
    }

    #[test]
    fn truncate_array_respects_budget() {
        let items = vec!["a".repeat(80), "b".repeat(80), "c".repeat(80)];
        let truncated = truncate_array(&items, 150);
        assert_eq!(truncated.len(), 1);
        assert!(truncated[0].len() <= 150);
    }
}
