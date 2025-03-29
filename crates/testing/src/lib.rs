//! Spacejam testing library
#![cfg(test)]

use runner::Runner;
use std::{fs, path::Path};
use tracing_subscriber::EnvFilter;
mod accumulate;

/// Initialize tracing subscriber
pub fn init_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

#[test]
fn coverage() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut tests = fs::read_dir(root)
        .expect("failed to read tests")
        .filter_map(|e| {
            let entry = e.ok()?;
            if entry.path().extension().unwrap_or_default() == "rs"
                && entry
                    .path()
                    .file_name()
                    .expect("failed to get file name")
                    .to_str()
                    .expect("failed to get file name")
                    != "lib"
            {
                println!("reading {}", entry.path().display());
                fs::read_to_string(entry.path()).ok()
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .join("");

    tests.push_str(
        &fs::read_to_string(Path::new(env!("OUT_DIR")).join("pvm.rs"))
            .expect("failed to read pvm.rs"),
    );

    for test in specjam::registry::ALL_TESTS {
        if test.is_full() {
            continue;
        }

        if !tests.contains(test.name) {
            panic!("test {} not found", test.name);
        }
    }
}

/// This macro accepts a list of test names and generates a test for each of them
#[macro_export]
macro_rules! impl_tests {
    (
        $module:ident,
        @scale
        $($scale_tests:ident),* $(,)?
    ) => {
        paste::paste! {
            $(
                #[test]
                fn [<$scale_tests _tiny>]() {
                    let test = specjam::registry::tests::[<TEST_ $module:upper _ $scale_tests:upper _TINY>];
                    $crate::Runner::step(&test)
                        .expect(&format!("could not run test {}::{}", &stringify!($module), &stringify!($scale_tests)));
                }
            )*
        }
    };
    (
        $module:ident,
        $($data_tests:ident),* $(,)?
    ) => {
        paste::paste! {
            $(
                #[test]
                fn $data_tests() {
                    let test = specjam::registry::tests::[<TEST_ $module:upper _ $data_tests:upper>];
                    $crate::Runner::step(&test)
                        .expect(&format!("could not run test {}::{}", &stringify!($module), &stringify!($data_tests)));
                }
            )*
        }
    }
}

pub mod assurances;
pub mod authorizations;
pub mod codec;
pub mod disputes;
pub mod history;
pub mod preimage;
pub mod pvm;
pub mod reports;
pub mod runner;
pub mod safrole;
pub mod shuffle;
pub mod statistics;
pub mod trie;
