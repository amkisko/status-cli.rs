//! Offline service catalog loaded from embedded awesome-status data.

use std::io::Read;

use flate2::read::GzDecoder;

use crate::error::{Error, Result};
use crate::fuzzy::{dedupe_by_name, find_services_fuzzy};
use crate::models::{FuzzyMatch, MatchType, Service};

const DATA_JSON_GZ: &[u8] = include_bytes!("../assets/data.json.gz");

#[derive(Debug, Clone)]
pub struct Catalog {
    services: Vec<Service>,
}

impl Catalog {
    pub fn load() -> Result<Self> {
        Self::load_from_gzip(DATA_JSON_GZ)
    }

    fn load_from_gzip(bytes: &[u8]) -> Result<Self> {
        let mut decoder = GzDecoder::new(bytes);
        let mut json = String::new();
        decoder
            .read_to_string(&mut json)
            .map_err(|error| Error::Catalog(format!("failed to decompress catalog: {error}")))?;
        let services: Vec<Service> = serde_json::from_str(&json).map_err(|error| {
            Error::Catalog(format!("failed to parse embedded catalog: {error}"))
        })?;
        Ok(Self { services })
    }

    pub fn services(&self) -> &[Service] {
        &self.services
    }

    pub fn len(&self) -> usize {
        self.services.len()
    }

    pub fn is_empty(&self) -> bool {
        self.services.is_empty()
    }

    pub fn list(&self, limit: usize) -> &[Service] {
        let end = limit.min(self.services.len());
        &self.services[..end]
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<FuzzyMatch> {
        let matches = find_services_fuzzy(&self.services, query, 0.5);
        dedupe_by_name(matches).into_iter().take(limit).collect()
    }

    pub fn best_match(&self, name: &str) -> Option<ShowResult> {
        let matches = find_services_fuzzy(&self.services, name, 0.6);
        let matches = dedupe_by_name(matches);
        let first = matches.first()?.clone();
        if first.match_type == MatchType::Exact || first.score >= 0.9 || matches.len() == 1 {
            return Some(ShowResult {
                service: first.service,
                alternatives: Vec::new(),
            });
        }
        let alternatives = matches
            .iter()
            .skip(1)
            .take(2)
            .map(|item| item.service.name.clone())
            .collect();
        Some(ShowResult {
            service: first.service,
            alternatives,
        })
    }

    pub fn resolve_status_url(&self, name_or_url: &str) -> Result<(String, Option<Service>)> {
        let trimmed = name_or_url.trim();
        if trimmed.starts_with("http://") || trimmed.starts_with("https://") {
            return Ok((trimmed.to_string(), None));
        }
        let result = self
            .best_match(trimmed)
            .ok_or_else(|| Error::Usage(format!("service '{trimmed}' not found")))?;
        let status_url = result.service.status_url.clone().ok_or_else(|| {
            Error::Usage(format!(
                "service '{}' has no status_url",
                result.service.name
            ))
        })?;
        Ok((status_url, Some(result.service)))
    }
}

#[derive(Debug, Clone)]
pub struct ShowResult {
    pub service: Service,
    pub alternatives: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_loads_and_finds_github() {
        let catalog = Catalog::load().expect("catalog");
        assert!(catalog.len() > 1000);
        let results = catalog.search("GitHub", 5);
        assert!(!results.is_empty());
        assert_eq!(results[0].service.name, "GitHub");
    }

    #[test]
    fn resolve_accepts_url() {
        let catalog = Catalog::load().expect("catalog");
        let (url, service) = catalog
            .resolve_status_url("https://status.example.com")
            .expect("url");
        assert_eq!(url, "https://status.example.com");
        assert!(service.is_none());
    }

    #[test]
    fn corrupt_gzip_returns_catalog_error() {
        let error = Catalog::load_from_gzip(b"not-gzip").expect_err("corrupt");
        assert!(error.to_string().contains("decompress"));
    }
}
