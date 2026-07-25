//! Append compact JSON lines for cron-friendly local logs.

use serde_json::Value;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;

pub fn append_json_line(path: &Path, value: &Value) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|error| format!("open {}: {error}", path.display()))?;
    let line = serde_json::to_string(value).map_err(|error| error.to_string())?;
    writeln!(file, "{line}").map_err(|error| format!("write {}: {error}", path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn appends_one_line_per_call() {
        let path = std::env::temp_dir().join("status-cli-jsonl-test.jsonl");
        let _ = fs::remove_file(&path);
        append_json_line(&path, &json!({"a": 1})).expect("append");
        append_json_line(&path, &json!({"b": 2})).expect("append");
        let body = fs::read_to_string(&path).expect("read");
        let lines: Vec<_> = body.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"a\":1"));
        let _ = fs::remove_file(path);
    }
}
