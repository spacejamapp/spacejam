//! Utility functions for the PVM CLI

use anyhow::Result;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::time::SystemTime;

/// Build the PVM blob
///
/// NOTE: this is used for the build script of services
pub fn build(package: &str) -> Result<()> {
    let target = env::var("TARGET")?;
    if target.contains("polkavm") {
        return Ok(());
    }

    // Build the service
    let target = etc::find_up("target")?;
    let jam = target.join("jam");
    let current = env::current_dir()?;
    let output = jam.join(format!("{package}.jam"));
    let rebuild = if !output.exists() {
        true
    } else {
        let modified = fs::metadata(&output)?.modified()?;
        check_modified(&current, modified)?
    };

    if rebuild {
        crate::cmd::Build::default().run()?;
    }

    // copy service to OUT_DIR
    let service = PathBuf::from(env::var("OUT_DIR")?).join("service.jam");
    println!("Copying service to OUT_DIR: {}", service.display());
    fs::copy(&output, &service)?;
    Ok(())
}

/// Check if any Rust source files have been modified after the given time
fn check_modified(dir: &Path, since: SystemTime) -> Result<bool> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let metadata = entry.metadata()?;
        if metadata.is_dir() {
            if check_modified(&path, since)? {
                return Ok(true);
            }
        } else if path.extension().and_then(|s| s.to_str()) == Some("rs")
            && metadata.modified()? > since
        {
            return Ok(true);
        }
    }

    Ok(false)
}
