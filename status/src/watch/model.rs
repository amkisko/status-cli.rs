//! Watchlist dashboard state and refresh helpers.

use super::row::{apply_result, RowKind, WatchRow};
use status_lib::{fetch_status, Catalog};
use std::time::{Duration, Instant};

pub struct WatchModel {
    pub rows: Vec<WatchRow>,
    pub selected: usize,
    pub interval: Duration,
    pub max_length: usize,
    pub timeout: Option<u64>,
    pub last_refresh: Option<Instant>,
    pub refresh_index: Option<usize>,
    pub message: String,
}

impl WatchModel {
    pub fn new(
        targets: Vec<String>,
        interval_secs: u64,
        max_length: usize,
        timeout: Option<u64>,
    ) -> Result<Self, String> {
        let catalog = Catalog::load().map_err(|error| error.to_string())?;
        let mut rows = Vec::with_capacity(targets.len());
        for target in targets {
            let (status_url, service) = catalog
                .resolve_status_url(&target)
                .map_err(|error| error.to_string())?;
            let label = service
                .as_ref()
                .map(|item| item.name.clone())
                .unwrap_or_else(|| target.clone());
            rows.push(WatchRow {
                label,
                status_url,
                kind: RowKind::Pending,
                status_text: "—".into(),
                detail: String::new(),
            });
        }
        Ok(Self {
            rows,
            selected: 0,
            interval: Duration::from_secs(interval_secs.max(1)),
            max_length,
            timeout,
            last_refresh: None,
            refresh_index: Some(0),
            message: "Refreshing…".into(),
        })
    }

    pub fn select_next(&mut self) {
        if !self.rows.is_empty() {
            self.selected = (self.selected + 1) % self.rows.len();
        }
    }

    pub fn select_previous(&mut self) {
        if !self.rows.is_empty() {
            self.selected = if self.selected == 0 {
                self.rows.len() - 1
            } else {
                self.selected - 1
            };
        }
    }

    pub fn request_refresh(&mut self) {
        if self.refresh_index.is_none() {
            self.refresh_index = Some(0);
            self.message = "Refreshing…".into();
        }
    }

    pub fn due_for_refresh(&self, now: Instant) -> bool {
        self.refresh_index.is_none()
            && self
                .last_refresh
                .is_none_or(|stamp| now.duration_since(stamp) >= self.interval)
    }

    pub fn tick_refresh(&mut self) {
        let Some(index) = self.refresh_index else {
            return;
        };
        if index >= self.rows.len() {
            self.refresh_index = None;
            self.last_refresh = Some(Instant::now());
            self.message = format!(
                "Updated · next in {}s · q quit · r refresh",
                self.interval.as_secs()
            );
            return;
        }
        self.rows[index].kind = RowKind::Loading;
        let status_url = self.rows[index].status_url.clone();
        let result = fetch_status(&status_url, self.max_length, self.timeout);
        apply_result(&mut self.rows[index], &result);
        self.refresh_index = Some(index + 1);
        self.message = format!("Refreshing {}/{}…", index + 1, self.rows.len());
    }
}
