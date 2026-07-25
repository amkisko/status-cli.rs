//! Status CLI library entrypoint.

mod cli;
mod commands;
mod completions;
mod exit;
mod jsonl;
mod man;
mod output;
mod watchlist;

use clap::Parser;
use cli::{Cli, Commands, OutputFormatArg};
use commands::{CheckOptions, FetchOptions};
use exit::AppExit;
use output::{resolve_output_mode, OutputFormat};
use std::process::ExitCode;

pub fn run() -> ExitCode {
    let cli = Cli::parse();
    let mode = resolve_output_mode(
        match cli.output {
            OutputFormatArg::Plain => OutputFormat::Plain,
            OutputFormatArg::Json => OutputFormat::Json,
        },
        cli.json,
        cli.plain,
    );

    let result = match cli.command {
        Commands::Search { query, limit } => commands::run_search(&query, limit, mode),
        Commands::List { limit } => commands::run_list(limit, mode),
        Commands::Show { name } => commands::run_show(&name, mode),
        Commands::Check {
            target,
            from,
            fail_if_degraded,
            append_jsonl,
            max_length,
        } => commands::run_check(CheckOptions {
            target: target.as_deref(),
            from: from.as_deref(),
            max_length,
            timeout: cli.timeout,
            fail_if_degraded,
            append_jsonl: append_jsonl.as_deref(),
            mode,
        }),
        Commands::Fetch {
            url,
            fail_if_degraded,
            append_jsonl,
            max_length,
        } => commands::run_fetch(FetchOptions {
            url: &url,
            max_length,
            timeout: cli.timeout,
            fail_if_degraded,
            append_jsonl: append_jsonl.as_deref(),
            mode,
        }),
        Commands::Completions { shell } => {
            completions::print_completions(shell);
            Ok(())
        }
        Commands::Man => man::print_man().map_err(|message| {
            eprintln!("error: {message}");
            AppExit::General
        }),
        Commands::Version => {
            println!("status {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
    };

    match result {
        Ok(()) => AppExit::Success.into(),
        Err(code) => code.into(),
    }
}
