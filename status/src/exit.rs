//! Process exit codes for script-friendly error handling.

use status_lib::Error;
use std::process::ExitCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AppExit {
    Success = 0,
    General = 1,
    Usage = 2,
    /// Opt-in: status is degraded/unknown when `--fail-if-degraded` is set.
    Degraded = 3,
    Network = 4,
    Io = 5,
}

impl AppExit {
    pub fn code(self) -> ExitCode {
        ExitCode::from(self as u8)
    }
}

impl From<AppExit> for ExitCode {
    fn from(value: AppExit) -> Self {
        value.code()
    }
}

pub fn exit_for_error(error: &Error) -> AppExit {
    match error {
        Error::Usage(_) => AppExit::Usage,
        Error::Network(_) | Error::ResponseTooLarge { .. } => AppExit::Network,
        Error::Io(_) => AppExit::Io,
        Error::Catalog(_) | Error::Other(_) => AppExit::General,
    }
}
