//! Man page generation.

use crate::cli::Cli;
use clap::CommandFactory;

pub fn print_man() -> Result<(), String> {
    let command = Cli::command();
    let man = clap_mangen::Man::new(command);
    let mut buffer = Vec::new();
    man.render(&mut buffer).map_err(|error| error.to_string())?;
    print!("{}", String::from_utf8_lossy(&buffer));
    Ok(())
}
