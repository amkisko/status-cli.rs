//! Output formatting for human, script, and JSON consumers.

use serde_json::{json, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputMode {
    #[default]
    HumanPlain,
    ScriptPlain,
    JsonCompact,
    JsonPretty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Plain,
    Json,
}

pub fn resolve_output_mode(
    output: OutputFormat,
    json_flag: bool,
    plain_script_flag: bool,
) -> OutputMode {
    if json_flag {
        return OutputMode::JsonCompact;
    }
    if plain_script_flag {
        return OutputMode::ScriptPlain;
    }
    match output {
        OutputFormat::Plain => OutputMode::HumanPlain,
        OutputFormat::Json => OutputMode::JsonPretty,
    }
}

pub fn emit_value(mode: OutputMode, value: &Value) -> Result<(), String> {
    let text = match mode {
        OutputMode::HumanPlain => format_human(value),
        OutputMode::ScriptPlain => format_script(value),
        OutputMode::JsonCompact => {
            serde_json::to_string(value).map_err(|error| error.to_string())?
        }
        OutputMode::JsonPretty => {
            serde_json::to_string_pretty(value).map_err(|error| error.to_string())?
        }
    };
    println!("{text}");
    Ok(())
}

fn format_human(value: &Value) -> String {
    if let Some(message) = value.get("message").and_then(Value::as_str) {
        return message.to_string();
    }
    if let Some(services) = value.get("services").and_then(Value::as_array) {
        if services.is_empty() {
            return "No services found".into();
        }
        return services
            .iter()
            .map(format_service_human)
            .collect::<Vec<_>>()
            .join("\n\n");
    }
    if value.get("name").is_some() {
        let mut text = format_service_human(value);
        if let Some(alternatives) = value.get("alternatives").and_then(Value::as_array) {
            if !alternatives.is_empty() {
                let names = alternatives
                    .iter()
                    .filter_map(Value::as_str)
                    .collect::<Vec<_>>()
                    .join(", ");
                text.push_str(&format!("\n\nDid you mean one of these? {names}"));
            }
        }
        return text;
    }
    if value.get("latest_status").is_some() || value.get("status_url").is_some() {
        return format_fetch_human(value);
    }
    if let Some(names) = value.get("names").and_then(Value::as_array) {
        let shown = value.get("shown").and_then(Value::as_u64).unwrap_or(0);
        let total = value.get("total").and_then(Value::as_u64).unwrap_or(0);
        let joined = names
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        let mut text = format!("Available services ({shown}/{total}):\n{joined}");
        if total > shown {
            text.push_str(&format!("\n... and {} more.", total - shown));
        }
        return text;
    }
    value.to_string()
}

fn format_service_human(value: &Value) -> String {
    let name = value
        .get("name")
        .and_then(Value::as_str)
        .unwrap_or("Unknown");
    let mut lines = vec![name.to_string()];
    for (label, key) in [
        ("Status", "status_url"),
        ("Website", "website_url"),
        ("Security", "security_url"),
        ("Support", "support_url"),
    ] {
        if let Some(url) = value.get(key).and_then(Value::as_str) {
            lines.push(format!("  {label}: {url}"));
        }
    }
    if let Some(aux) = value.get("aux_urls").and_then(Value::as_array) {
        let urls = aux
            .iter()
            .filter_map(Value::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        if !urls.is_empty() {
            lines.push(format!("  Other: {urls}"));
        }
    }
    lines.join("\n")
}

fn format_fetch_human(value: &Value) -> String {
    let mut lines = Vec::new();
    if let Some(status) = value.get("latest_status").and_then(Value::as_str) {
        lines.push(format!("Status: {status}"));
    } else {
        lines.push("Status: unknown".into());
    }
    if let Some(url) = value.get("status_url").and_then(Value::as_str) {
        lines.push(format!("URL: {url}"));
    }
    if let Some(code) = value.get("http_status_code") {
        lines.push(format!("HTTP: {code}"));
    }
    if let Some(error) = value.get("error").and_then(Value::as_str) {
        lines.push(format!("Error: {error}"));
    }
    if let Some(history) = value.get("history").and_then(Value::as_array) {
        if !history.is_empty() {
            lines.push("History:".into());
            for item in history.iter().take(10) {
                if let Some(text) = item.as_str() {
                    lines.push(format!("  - {text}"));
                }
            }
        }
    }
    if let Some(messages) = value.get("messages").and_then(Value::as_array) {
        if !messages.is_empty() {
            lines.push("Messages:".into());
            for item in messages {
                if let Some(text) = item.as_str() {
                    lines.push(format!("  - {text}"));
                }
            }
        }
    }
    lines.join("\n")
}

fn format_script(value: &Value) -> String {
    if let Some(services) = value.get("services").and_then(Value::as_array) {
        return services
            .iter()
            .map(|service| {
                format!(
                    "name={}\tstatus_url={}",
                    service.get("name").and_then(Value::as_str).unwrap_or(""),
                    service
                        .get("status_url")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
    }
    if value.get("latest_status").is_some() || value.get("status_url").is_some() {
        return [
            (
                "latest_status",
                value
                    .get("latest_status")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ),
            (
                "status_url",
                value
                    .get("status_url")
                    .and_then(Value::as_str)
                    .unwrap_or(""),
            ),
            (
                "error",
                value.get("error").and_then(Value::as_str).unwrap_or(""),
            ),
        ]
        .iter()
        .map(|(key, val)| format!("{key}={val}"))
        .collect::<Vec<_>>()
        .join("\n");
    }
    json!(value).to_string()
}
