//! Command handlers.

use crate::exit::{exit_for_error, AppExit};
use crate::jsonl::append_json_line;
use crate::output::{emit_value, OutputMode};
use crate::watchlist;
use serde_json::{json, Value};
use status_lib::{fetch_status, should_fail_if_degraded, Catalog, FetchResult, Service};
use std::path::Path;

pub struct CheckOptions<'a> {
    pub target: Option<&'a str>,
    pub from: Option<&'a Path>,
    pub max_length: usize,
    pub timeout: Option<u64>,
    pub fail_if_degraded: bool,
    pub append_jsonl: Option<&'a Path>,
    pub mode: OutputMode,
}

pub struct FetchOptions<'a> {
    pub url: &'a str,
    pub max_length: usize,
    pub timeout: Option<u64>,
    pub fail_if_degraded: bool,
    pub append_jsonl: Option<&'a Path>,
    pub mode: OutputMode,
}

pub fn run_search(query: &str, limit: usize, mode: OutputMode) -> Result<(), AppExit> {
    let catalog = Catalog::load().map_err(|error| exit_for_error(&error))?;
    let matches = catalog.search(query, limit);
    if matches.is_empty() {
        emit_value(
            mode,
            &json!({"message": format!("No services found matching '{query}'")}),
        )
        .map_err(|_| AppExit::General)?;
        return Ok(());
    }
    let services: Vec<Value> = matches
        .iter()
        .map(|item| service_json(&item.service))
        .collect();
    emit_value(mode, &json!({ "services": services, "query": query })).map_err(|_| AppExit::General)
}

pub fn run_list(limit: usize, mode: OutputMode) -> Result<(), AppExit> {
    let catalog = Catalog::load().map_err(|error| exit_for_error(&error))?;
    let listed = catalog.list(limit);
    let names: Vec<&str> = listed.iter().map(|service| service.name.as_str()).collect();
    emit_value(
        mode,
        &json!({
            "names": names,
            "shown": listed.len(),
            "total": catalog.len(),
        }),
    )
    .map_err(|_| AppExit::General)
}

pub fn run_show(name: &str, mode: OutputMode) -> Result<(), AppExit> {
    let catalog = Catalog::load().map_err(|error| exit_for_error(&error))?;
    let Some(result) = catalog.best_match(name) else {
        eprintln!("error: service '{name}' not found");
        return Err(AppExit::Usage);
    };
    let mut value = service_json(&result.service);
    if let Value::Object(ref mut map) = value {
        map.insert("alternatives".into(), json!(result.alternatives));
    }
    emit_value(mode, &value).map_err(|_| AppExit::General)
}

pub fn run_check(options: CheckOptions<'_>) -> Result<(), AppExit> {
    let targets = resolve_targets(options.target, options.from)?;
    let catalog = Catalog::load().map_err(|error| exit_for_error(&error))?;
    let mut values = Vec::new();
    let mut network_failed = false;
    let mut degraded = false;

    for target in &targets {
        let (status_url, service) = catalog
            .resolve_status_url(target)
            .map_err(|error| exit_for_error(&error))?;
        let result = fetch_status(&status_url, options.max_length, options.timeout);
        let value = fetch_value(&result, service.as_ref())?;
        if let Some(path) = options.append_jsonl {
            append_json_line(path, &value).map_err(|message| {
                eprintln!("error: {message}");
                AppExit::Io
            })?;
        }
        if is_hard_fetch_failure(&result) {
            network_failed = true;
        }
        if options.fail_if_degraded && should_fail_if_degraded(&result) {
            degraded = true;
        }
        values.push(value);
    }

    let payload = if values.len() == 1 {
        values.remove(0)
    } else {
        json!({ "results": values, "count": targets.len() })
    };
    emit_value(options.mode, &payload).map_err(|_| AppExit::General)?;

    if network_failed {
        return Err(AppExit::Network);
    }
    if degraded {
        return Err(AppExit::Degraded);
    }
    Ok(())
}

pub fn run_fetch(options: FetchOptions<'_>) -> Result<(), AppExit> {
    if !(options.url.starts_with("http://") || options.url.starts_with("https://")) {
        eprintln!("error: fetch requires an http(s) URL");
        return Err(AppExit::Usage);
    }
    let result = fetch_status(options.url, options.max_length, options.timeout);
    let value = fetch_value(&result, None)?;
    if let Some(path) = options.append_jsonl {
        append_json_line(path, &value).map_err(|message| {
            eprintln!("error: {message}");
            AppExit::Io
        })?;
    }
    emit_value(options.mode, &value).map_err(|_| AppExit::General)?;
    if is_hard_fetch_failure(&result) {
        return Err(AppExit::Network);
    }
    if options.fail_if_degraded && should_fail_if_degraded(&result) {
        return Err(AppExit::Degraded);
    }
    Ok(())
}

fn resolve_targets(target: Option<&str>, from: Option<&Path>) -> Result<Vec<String>, AppExit> {
    match (target, from) {
        (Some(target), None) => Ok(vec![target.to_string()]),
        (None, Some(path)) => watchlist::load_targets(path).map_err(|message| {
            eprintln!("error: {message}");
            AppExit::Usage
        }),
        (Some(_), Some(_)) => {
            eprintln!("error: provide either a target or --from, not both");
            Err(AppExit::Usage)
        }
        (None, None) => {
            eprintln!("error: provide a target or --from <watchlist>");
            Err(AppExit::Usage)
        }
    }
}

fn fetch_value(result: &FetchResult, service: Option<&Service>) -> Result<Value, AppExit> {
    let mut value = serde_json::to_value(result).map_err(|_| AppExit::General)?;
    if let (Some(service), Value::Object(ref mut map)) = (service, &mut value) {
        map.insert("service".into(), service_json(service));
    }
    Ok(value)
}

fn is_hard_fetch_failure(result: &FetchResult) -> bool {
    result.error.as_ref().is_some_and(|error| {
        error.contains("Error fetching status")
            || error.contains("Response size limit exceeded")
            || error.contains("Failed to fetch:")
    }) && result.latest_status.is_none()
        && result.history.is_empty()
}

fn service_json(service: &Service) -> Value {
    json!({
        "name": service.name,
        "status_url": service.status_url,
        "website_url": service.website_url,
        "security_url": service.security_url,
        "support_url": service.support_url,
        "aux_urls": service.aux_urls,
    })
}
