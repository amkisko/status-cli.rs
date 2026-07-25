//! Decide whether a live status result counts as operational for automation.

use crate::models::FetchResult;

/// Returns true when the result looks like a healthy / all-clear page status.
pub fn is_operational_status(status: &str) -> bool {
    let lower = status.trim().to_ascii_lowercase();
    if lower.is_empty() {
        return false;
    }
    if lower == "operational" || lower.contains("all systems operational") {
        return true;
    }
    let unhealthy = [
        "degrad",
        "outage",
        "partial",
        "major",
        "critical",
        "maintenance",
        "unavailable",
        "down",
        "investigating",
        "identified",
        "incident",
    ];
    !unhealthy.iter().any(|needle| lower.contains(needle))
}

/// True when automation should treat the result as a failure under `--fail-if-degraded`.
pub fn should_fail_if_degraded(result: &FetchResult) -> bool {
    match &result.latest_status {
        Some(status) => !is_operational_status(status),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::FetchResult;

    #[test]
    fn accepts_operational_wording() {
        assert!(is_operational_status("Operational"));
        assert!(is_operational_status("All Systems Operational"));
    }

    #[test]
    fn rejects_degraded_wording() {
        assert!(!is_operational_status("Minor Service Outage"));
        assert!(!is_operational_status("Partially Degraded Service"));
        assert!(!is_operational_status("Partial System Degradation"));
        assert!(!is_operational_status("Under Maintenance"));
        assert!(!is_operational_status("identified"));
    }

    #[test]
    fn fail_if_degraded_treats_missing_status_as_failure() {
        let result = FetchResult::empty("https://example.com");
        assert!(should_fail_if_degraded(&result));
    }
}
