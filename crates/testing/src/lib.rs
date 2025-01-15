//! Spacejam testing library
#![cfg(test)]

use runner::Runner;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

/// Initialize tracing subscriber
pub fn init_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

/// Load a test vector from a test vector file
pub fn load_test(module: &str, scale: &str, path: &str, repl: bool) -> String {
    let mut root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    root.extend(["jamtestvectors", module, scale]);

    // replace underscores with dashes
    let patts = path.split("_").collect::<Vec<&str>>();
    if repl {
        let mut name = patts[..patts.len() - 1].join("_");
        name.push_str(&format!(
            "-{}",
            patts
                .last()
                .expect("pattern must have at least one element")
        ));
        root.push(name);
    } else {
        root.push(patts.join("-"));
    };

    // set the extension to json and read the file
    root.set_extension("json");
    std::fs::read_to_string(&root)
        .unwrap_or_else(|_| panic!("could not read test vector: {root:?}"))
}

fn include_all_tests(module: &str, path: &str) {
    init_tracing();
    let root = env!("CARGO_MANIFEST_DIR");
    let path = format!("{root}/jamtestvectors/{module}/{path}");
    let this = std::fs::read_to_string(format!("{root}/src/{module}.rs"))
        .unwrap_or_else(|_| panic!("could not find module {module}.rs"));

    let mut count = 0;
    for file in std::fs::read_dir(&path)
        .unwrap_or_else(|_| panic!("could not find test vectors for {module}: {path:?}"))
    {
        let path = file.expect("could not read file {file:?}").path();
        if !path.with_extension("json").exists() {
            continue;
        }

        let test = path
            .with_extension("")
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .replace('-', "_");

        assert!(
            this.contains(&test),
            "{test} of {module} not exist: {path:?}"
        );
        count += 1;
    }

    tracing::info!("found {count} tests for {module}.");
}

macro_rules! impl_all_tests {
    (
        @data $($name:ident),*,
        @scale $($scale_name:ident),*,
        @unformatted $($unformatted_name:ident),*
    ) => {
        $(impl_all_tests!($name, "data");)*
        $(impl_all_tests!(@scale $scale_name, true, $);)*
        $(impl_all_tests!(@scale $unformatted_name, false, $);)*
    };
    ($name:ident, $path:expr) => {
        paste::paste!{
            #[test]
            fn [<test_include_all_ $name _tests>]() {
                include_all_tests(stringify!($name), $path);
            }
        }
    };
    (@scale $name:ident, $repl:expr, $dol:tt) => {
        impl_all_tests!($name, "tiny");

        paste::paste! {
            #[macro_export]
            macro_rules! [<_impl_ $name _tests>] {
                ($dol($test:ident),*) => {
                    mod tiny {
                        use super::*;
                        $dol([<_impl_ $name _tests>]!($test, "tiny");)*
                    }
                };
                ($test:ident, $scale:expr) => {
                    #[test]
                    fn $test() {
                        let module = stringify!($name);
                        let test = stringify!($test);
                        let scale = $scale;
                        Test::from_json($crate::load_test(module, scale, test, $repl))
                            .expect(&format!(
                                "could not parse test vector: {module}::{scale}::{test}"
                            ))
                            .run()
                    }
                }
            }

            pub use [<_ impl_ $name _tests>] as [<impl_ $name _tests>];
        }
    };
}

impl_all_tests! {
    @data
    codec,
    history,

    @scale
    authorizations,
    assurances,
    disputes,
    reports,
    statistics,

    @unformatted
    safrole
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
                    crate::Runner::step(&test)
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
                    crate::Runner::step(&test)
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
