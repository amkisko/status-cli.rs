//! Per-target row status mapping for the watch dashboard.

use status_lib::{is_operational_status, should_fail_if_degraded, FetchResult};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    Pending,
    Loading,
    Ok,
    Degraded,
    Error,
}

#[derive(Debug, Clone)]
pub struct WatchRow {
    pub label: String,
    pub status_url: String,
    pub kind: RowKind,
    pub status_text: String,
    pub detail: String,
}

pub fn apply_result(row: &mut WatchRow, result: &FetchResult) {
    if let Some(error) = &result.error {
        if result.latest_status.is_none() {
            row.kind = RowKind::Error;
            row.status_text = "Error".into();
            row.detail = error.clone();
            return;
        }
    }
    match &result.latest_status {
        Some(status) if is_operational_status(status) => {
            row.kind = RowKind::Ok;
            row.status_text = status.clone();
            row.detail = first_message(result);
        }
        Some(status) => {
            row.kind = RowKind::Degraded;
            row.status_text = status.clone();
            row.detail = first_message(result);
        }
        None if should_fail_if_degraded(result) => {
            row.kind = RowKind::Degraded;
            row.status_text = "Unknown".into();
            row.detail = result
                .error
                .clone()
                .unwrap_or_else(|| "No status extracted".into());
        }
        None => {
            row.kind = RowKind::Pending;
            row.status_text = "—".into();
            row.detail = String::new();
        }
    }
}

fn first_message(result: &FetchResult) -> String {
    result
        .messages
        .first()
        .cloned()
        .or_else(|| result.history.first().cloned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apply_result_marks_operational() {
        let mut row = WatchRow {
            label: "GitHub".into(),
            status_url: "https://www.githubstatus.com".into(),
            kind: RowKind::Pending,
            status_text: "—".into(),
            detail: String::new(),
        };
        let mut result = FetchResult::empty(&row.status_url);
        result.latest_status = Some("All Systems Operational".into());
        apply_result(&mut row, &result);
        assert_eq!(row.kind, RowKind::Ok);
    }

    #[test]
    fn apply_result_marks_degraded() {
        let mut row = WatchRow {
            label: "x".into(),
            status_url: "https://example.com".into(),
            kind: RowKind::Pending,
            status_text: "—".into(),
            detail: String::new(),
        };
        let mut result = FetchResult::empty(&row.status_url);
        result.latest_status = Some("Partial System Outage".into());
        apply_result(&mut row, &result);
        assert_eq!(row.kind, RowKind::Degraded);
    }
}
