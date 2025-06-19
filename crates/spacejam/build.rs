//! build script for spacejam

use std::{fs, process::Command};

const TINY_DEV_SPEC: &str = "https://gist.githubusercontent.com/clearloop/52b9d5c16d3bd2a2d900b756fc64a9d1/raw/fbf84b774254cb68071a8a37cf8faac699bebf48/spec.json";

fn main() {
    println!("cargo:rerun-if-changed=src/chain/spec.rs");
    println!("cargo:rerun-if-changed=build.rs");
    let root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"),
    );

    let dev = root.join("spec/dev");
    let target = dev.join("spec.json");
    if target.exists() {
        return;
    }

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
