use assert_cmd::Command;
use predicates::prelude::*;

fn status() -> Command {
    Command::cargo_bin("status").expect("binary")
}

#[test]
fn version_prints_package_version() {
    status()
        .arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("status 0.1.0"));
}

#[test]
fn help_lists_core_commands() {
    status()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("search"))
        .stdout(predicate::str::contains("check"))
        .stdout(predicate::str::contains("watch"))
        .stdout(predicate::str::contains("fetch"));
}

#[test]
fn search_finds_github() {
    status()
        .args(["search", "GitHub", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("GitHub"))
        .stdout(predicate::str::contains("githubstatus.com"));
}

#[test]
fn list_limits_results() {
    status()
        .args(["list", "--limit", "3", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"shown\":3"));
}

#[test]
fn show_missing_service_exits_usage() {
    status()
        .args(["show", "zzz-not-a-real-service-name-zzzz"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn fetch_requires_url() {
    status()
        .args(["fetch", "not-a-url"])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn check_requires_target_or_from() {
    status().arg("check").assert().failure().code(2);
}

#[test]
fn watch_requires_target_or_from() {
    status().arg("watch").assert().failure().code(2);
}

#[test]
fn watch_help_lists_keys_and_interval() {
    status()
        .args(["watch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("interval"))
        .stdout(predicate::str::contains("refresh"));
}

#[test]
fn check_rejects_target_with_from() {
    let path = std::env::temp_dir().join("status-cli-watchlist-both.toml");
    std::fs::write(&path, "targets = [\"github\"]\n").expect("write");
    status()
        .args(["check", "github", "--from", path.to_str().expect("utf8")])
        .assert()
        .failure()
        .code(2);
    let _ = std::fs::remove_file(path);
}

#[test]
fn check_from_missing_watchlist_exits_usage() {
    status()
        .args([
            "check",
            "--from",
            "/tmp/status-cli-missing-watchlist-zzzz.toml",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn help_mentions_fail_if_degraded() {
    status()
        .args(["check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("fail-if-degraded"))
        .stdout(predicate::str::contains("append-jsonl"));
}

#[test]
fn completions_bash_emits_script() {
    status()
        .args(["completions", "bash"])
        .assert()
        .success()
        .stdout(predicate::str::contains("status"));
}

#[test]
fn man_emits_troff() {
    status()
        .arg("man")
        .assert()
        .success()
        .stdout(predicate::str::contains(".TH"));
}
