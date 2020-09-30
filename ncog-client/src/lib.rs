mod config;
mod native;

pub use native::{AuthState, Error, Ncog, NcogClient, NcogClientLogic};
pub use ncog_shared as shared;
