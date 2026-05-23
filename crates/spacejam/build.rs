//! build script for spacejam

use std::{fs, path::Path, process::Command};

const TINY_DEV_SPEC: &str = "https://gist.githubusercontent.com/clearloop/52b9d5c16d3bd2a2d900b756fc64a9d1/raw/fbf84b774254cb68071a8a37cf8faac699bebf48/spec.json";

fn main() {
    println!("cargo:rerun-if-changed=src/chain/spec.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"),
    );

    self::emit_graypaper_version(&root);
    self::fetch_tiny_dev_spec(&root);
}

/// Read `[workspace.metadata.graypaper] version` from the root Cargo.toml
fn emit_graypaper_version(crate_root: &Path) {
    let workspace_manifest = crate_root.join("../../Cargo.toml");
    println!("cargo:rerun-if-changed={}", workspace_manifest.display());

    let text = fs::read_to_string(&workspace_manifest)
        .expect("failed to read workspace Cargo.toml for graypaper version");
    let version = parse_graypaper_version(&text)
        .expect("`[workspace.metadata.graypaper] version` missing from Cargo.toml");
    println!("cargo:rustc-env=GRAYPAPER_VERSION={version}");
}

fn parse_graypaper_version(manifest: &str) -> Option<String> {
    manifest
        .parse::<toml::Value>()
        .ok()?
        .get("workspace")?
        .get("metadata")?
        .get("graypaper")?
        .get("version")?
        .as_str()
        .map(str::to_string)
}

fn fetch_tiny_dev_spec(crate_root: &Path) {
    let dev = crate_root.join("spec/dev");
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
