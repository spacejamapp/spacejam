//! Build script for the RISC-V parser

use anyhow::Result;
use std::{env, path::PathBuf, process::Command};

const RISCV_OPCODES_REPO: &str = "https://github.com/riscv/riscv-opcodes.git";
const PARSE_ARGS: [&str; 3] = ["-rust", "rv_i", "rv_m"];
const TYPES: [(&str, u8); 6] = [
    ("R", 0b0010011),
    ("I", 0b0000011),
    ("S", 0b0100011),
    ("B", 0b1100011),
    ("U", 0b0110111),
    ("J", 0b1101111),
];

fn main() -> Result<()> {
    println!("cargo::rerun-if-changed=src/instr.rs");

    let root = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);
    download_opcodes(&root)?;

    Ok(())
}

// Download the RISC-V opcodes repository
fn download_opcodes(root: &PathBuf) -> Result<()> {
    let repo = PathBuf::from(root.join("riscv-opcodes"));
    if repo.exists() {
        return Ok(());
    }

    Command::new("git")
        .args(["clone", RISCV_OPCODES_REPO, "--depth", "1"])
        .current_dir(root)
        .status()
        .expect("Failed to downlaod riscv/riscv-opcodes");

    Command::new("./parse.py")
        .args(PARSE_ARGS)
        .current_dir(repo)
        .status()
        .expect("Failed to build riscv/riscv-opcodes");

    Ok(())
}
