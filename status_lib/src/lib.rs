//! Status page catalog and live fetch library.

pub mod catalog;
pub mod error;
pub mod fetch;
pub mod fuzzy;
pub mod models;
pub mod operational;
pub mod text;

pub use catalog::{Catalog, ShowResult};
pub use error::{Error, Result};
pub use fetch::fetch_status;
pub use models::{FetchResult, FuzzyMatch, MatchType, Service};
pub use operational::{is_operational_status, should_fail_if_degraded};
