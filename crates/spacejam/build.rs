//! build script for spacejam

use std::{fs, process::Command};

const TINY_DEV_SPEC: &str = "https://gist.githubusercontent.com/zdave-parity/72eb9cfe07756d2c0c13c3064600190d/raw/dcf0b65694c2fdefe7f85dbe7ad91f435aa92098/dev-spec.json";

fn main() {
    println!("cargo:rerun-if-changed=src/chain/spec.rs");
    println!("cargo:rerun-if-changed=build.rs");
    let root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"),
    );

    let dev = root.join("spec/dev");
    let target = dev.join("spec.json");
    fs::create_dir_all(&dev).expect("failed to create tiny spec dir");
    Command::new("curl")
        .args([
            TINY_DEV_SPEC,
            "-o",
            target.to_str().expect("failed to convert target to str"),
        ])
        .output()
        .expect("failed to download tiny spec");
}
