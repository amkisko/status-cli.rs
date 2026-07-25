//! Shell completion generation.

use crate::cli::Cli;
use clap::CommandFactory;
use clap_complete::{generate, Shell};
use std::io;

pub fn print_completions(shell: Shell) {
    let mut command = Cli::command();
    generate(shell, &mut command, "status", &mut io::stdout());
}
