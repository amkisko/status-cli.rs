//! Release script: format check, clippy, test, then tag and publish.

use std::io::{self, Write};
use std::process::Command;

const GREEN: &str = "\x1b[0;32m";
const RED: &str = "\x1b[1;31m";
const YELLOW: &str = "\x1b[1;33m";
const NC: &str = "\x1b[0m";

fn main() {
    let root = std::env::current_dir().expect("current dir");
    let root_display = root.to_string_lossy();

    run_cmd(&root_display, "cargo", &["fmt", "--all", "--", "--check"]);
    run_cmd(
        &root_display,
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    );
    run_cmd(&root_display, "cargo", &["test", "--workspace"]);
    run_cmd(
        &root_display,
        "cargo",
        &["run", "-p", "release", "--bin", "check-loc"],
    );

    if let Err(mismatches) = release::check_packaging(&root) {
        eprintln!(
            "{}packaging is out of sync with workspace version:{}",
            RED, NC
        );
        for mismatch in mismatches {
            eprintln!("  - {mismatch}");
        }
        eprintln!(
            "{}Run `cargo run -p release --bin sync-packaging` and commit before releasing.{}",
            YELLOW, NC
        );
        std::process::exit(1);
    }

    if !Command::new("git")
        .args(["diff", "--quiet"])
        .current_dir(&root)
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
    {
        eprintln!(
            "{}git working directory not clean, please commit your changes first{}",
            RED, NC
        );
        std::process::exit(1);
    }

    let version = release::workspace_version(&root);
    println!("Ready to release version {}", version);
    print!("Continue? [Y/n] ");
    let _ = io::stdout().flush();
    let mut line = String::new();
    if io::stdin().read_line(&mut line).is_err()
        || (!line.trim().is_empty() && !line.trim().eq_ignore_ascii_case("y"))
    {
        println!("Exiting");
        std::process::exit(1);
    }

    run_cmd(
        &root_display,
        "cargo",
        &["publish", "-p", "status_lib", "--allow-dirty"],
    );
    run_cmd(
        &root_display,
        "cargo",
        &["publish", "-p", "status", "--allow-dirty"],
    );
    run_cmd(&root_display, "git", &["tag", &format!("v{version}")]);
    run_cmd(&root_display, "git", &["push", "--tags"]);
    if Command::new("gh").arg("--version").output().is_ok() {
        run_cmd(
            &root_display,
            "gh",
            &[
                "release",
                "create",
                &format!("v{version}"),
                "--generate-notes",
            ],
        );
    }
    println!("{}Released v{version}{}", GREEN, NC);
}

fn run_cmd(root: &str, program: &str, args: &[&str]) {
    let status = Command::new(program)
        .args(args)
        .current_dir(root)
        .status()
        .unwrap_or_else(|error| panic!("failed to run {program}: {error}"));
    if !status.success() {
        eprintln!("{}command failed: {program} {}{}", RED, args.join(" "), NC);
        std::process::exit(status.code().unwrap_or(1));
    }
}
