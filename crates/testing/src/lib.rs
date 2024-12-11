//! Spacejam testing library
#![cfg(test)]
use tracing_subscriber::EnvFilter;

pub mod assurances;
pub mod codec;
pub mod disputes;
pub mod history;
pub mod reports;
pub mod safrole;
pub mod shuffle;
pub mod statistics;
pub mod trie;

/// Initialize tracing subscriber
pub fn init_tracing() {
    tracing_subscriber::fmt::Subscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .init();
}

fn include_all_tests(module: &str, path: &str) {
    init_tracing();
    let root = env!("CARGO_MANIFEST_DIR");
    let path = format!("{root}/jamtestvectors/{module}/{path}");
    let this = std::fs::read_to_string(format!("{root}/src/{module}.rs"))
        .unwrap_or_else(|_| panic!("could not find module {module}.rs"));

    let mut count = 0;
    for file in std::fs::read_dir(&path).unwrap_or_else(|_| panic!("could not find test vectors for {module}: {path:?}")) {
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

macro_rules! impl_include_all_tests {
    ($(($name:tt, $path:expr)),*) => {
        $(impl_include_all_tests!($name, $path);)*
    };
    ($name:tt, $path:expr) => {
        paste::paste!{
            #[test]
            fn [<test_include_all_ $name _tests>]() {
                include_all_tests(stringify!($name), $path);
            }
        }
    };
}

impl_include_all_tests! {
    (assurances, "tiny"),
    (codec, "data"),
    (disputes, "tiny"),
    (history, "data"),
    (reports, "tiny"),
    (safrole, "tiny"),
    (shuffle, ""),
    (statistics, "tiny"),
    (trie, "")
}
