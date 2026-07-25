//! Load a local watchlist of catalog names and status URLs.

use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct WatchlistFile {
    #[serde(default)]
    targets: Vec<String>,
}

pub fn load_targets(path: &Path) -> Result<Vec<String>, String> {
    let raw =
        fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("watchlist is empty: {}", path.display()));
    }

    if trimmed.starts_with('{') || path.extension().is_some_and(|ext| ext == "json") {
        return load_json_targets(trimmed, path);
    }
    if trimmed.starts_with('[') {
        return load_json_array(trimmed, path);
    }
    if path.extension().is_some_and(|ext| ext == "toml") || trimmed.contains("targets") {
        return load_toml_targets(trimmed, path);
    }
    normalize(load_line_targets(trimmed), path)
}

fn load_toml_targets(raw: &str, path: &Path) -> Result<Vec<String>, String> {
    let parsed: WatchlistFile =
        toml::from_str(raw).map_err(|error| format!("parse {}: {error}", path.display()))?;
    normalize(parsed.targets, path)
}

fn load_json_targets(raw: &str, path: &Path) -> Result<Vec<String>, String> {
    let parsed: WatchlistFile =
        serde_json::from_str(raw).map_err(|error| format!("parse {}: {error}", path.display()))?;
    normalize(parsed.targets, path)
}

fn load_json_array(raw: &str, path: &Path) -> Result<Vec<String>, String> {
    let parsed: Vec<String> =
        serde_json::from_str(raw).map_err(|error| format!("parse {}: {error}", path.display()))?;
    normalize(parsed, path)
}

fn load_line_targets(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_string)
        .collect()
}

fn normalize(targets: Vec<String>, path: &Path) -> Result<Vec<String>, String> {
    let targets: Vec<String> = targets
        .into_iter()
        .map(|target| target.trim().to_string())
        .filter(|target| !target.is_empty())
        .collect();
    if targets.is_empty() {
        return Err(format!("watchlist has no targets: {}", path.display()));
    }
    Ok(targets)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn loads_toml_targets() {
        let path = std::env::temp_dir().join("status-cli-watchlist-test.toml");
        let mut file = fs::File::create(&path).expect("create");
        write!(
            file,
            "targets = [\"github\", \"https://status.openai.com\"]"
        )
        .unwrap();
        let targets = load_targets(&path).expect("targets");
        assert_eq!(targets, vec!["github", "https://status.openai.com"]);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn loads_line_targets() {
        let path = std::env::temp_dir().join("status-cli-watchlist-test.txt");
        let mut file = fs::File::create(&path).expect("create");
        write!(file, "# comment\ngithub\n\ncloudflare\n").unwrap();
        let targets = load_targets(&path).expect("targets");
        assert_eq!(targets, vec!["github", "cloudflare"]);
        let _ = fs::remove_file(path);
    }
}
