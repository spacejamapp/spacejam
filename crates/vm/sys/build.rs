//! Link spacevm to this sys library

// use std::process::Command;

use std::{env, path::PathBuf};

fn main() {
    let _out = env::var("OUT_DIR").unwrap();
    let lib: PathBuf = env::current_dir().unwrap().join("../../../target/release");
    println!("cargo:rustc-link-search=native={}", lib.display());
    println!("cargo:rustc-link-lib=static=spacevm");
    println!("cargo:rerun-if-changed=build.rs");
}
