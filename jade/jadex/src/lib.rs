//! JAM Index Framework

pub use config::Config;
pub use hook::JadexHook;
pub use runtime::Hook;
pub use service::start;

mod config;
mod hook;
mod service;
