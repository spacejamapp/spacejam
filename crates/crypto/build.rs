use anyhow::Result;
use std::{path::Path, process::Command};

const REPO: &str = "https://github.com/davxy/bandersnatch-vrfs-spec.git";
const INTO: &str = "bandersnatch-vrfs-spec";

fn main() -> Result<()> {
    let into = Path::new(INTO);
    if into.exists() {
        return Ok(());
    }

    Command::new("git").args(["clone", REPO, INTO]).status()?;
    Command::new("git")
        .args(["checkout", "cc99f5c"])
        .current_dir(into)
        .status()?;
    Ok(())
}
