//! Prelude for the PVM testing library

use anyhow::{Context, Result};
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

/// Initialize the logger
pub fn init_logger() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}

/// Load the service
pub fn load_service(package: &str) -> Result<Vec<u8>> {
    let target = etc::find_up("target")
        .expect("Failed to find target directory")
        .join("jam")
        .join(format!("{}.jam", package));

    std::fs::read(&target).context(format!("Failed to read {}", target.display()))
}

/// Build the service
pub fn build_service(package: &str) {
    let mut config = cjam::cmd::Build::default();
    config.path = Some(PathBuf::from(package));
    config
        .run()
        .expect(&format!("Failed to build service at {}", package));
}

/// Load the current service
#[macro_export]
macro_rules! service {
    () => {{
        $crate::util::init_logger();

        match $crate::util::load_service(env!("CARGO_PKG_NAME")) {
            Ok(blob) => blob,
            Err(e) => {
                $crate::util::build_service(env!("CARGO_MANIFEST_DIR"));
                $crate::util::load_service(env!("CARGO_PKG_NAME")).expect("Failed to load service")
            }
        }
    }};
}
