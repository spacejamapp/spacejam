use anyhow::Result;
use proc_macro2::Span;
use quote::ToTokens;
use specjam::{Entry, Registry, Scale, Trace};
use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use syn::{Ident, ItemFn, parse_quote};

const REPORTS: &str = "../../res/jam-conformance/fuzz-reports/0.7.2/traces";
const TRACES: &str = "../../res/jam-test-vectors/traces";
const REPORT: &str = "../../res/report";
const SESSION: &str = "../../res/session/trace";

fn scale() -> Scale {
    if env::var("CARGO_FEATURE_FULL").is_ok() {
        Scale::Full
    } else {
        Scale::Tiny
    }
}

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../../res/jam-test-vectors");
    println!("cargo:rerun-if-changed=../../res/jam-conformance/fuzz-reports/0.7.2/traces");
    println!("cargo:rerun-if-changed=../../res/report");
    println!("cargo:rerun-if-changed=../../res/session/trace");
    println!("cargo:rerun-if-changed=./build.rs");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let workspace = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../../");
    let scale = scale();

    // set up the registry
    util::try_download(&workspace)?;
    let registry = Registry::with_scale(workspace.join("res/jam-test-vectors"), scale);

    // build all tests
    build_tests(registry.accumulate(scale)?, &out_dir.join("accumulate.rs"))?;
    build_tests(registry.assurances(scale)?, &out_dir.join("assurances.rs"))?;
    build_tests(
        registry.authorizations(scale)?,
        &out_dir.join("authorizations.rs"),
    )?;
    build_tests(registry.codec(scale)?, &out_dir.join("codec.rs"))?;
    build_tests(registry.disputes(scale)?, &out_dir.join("disputes.rs"))?;
    build_tests(registry.erasure(scale)?, &out_dir.join("erasure.rs"))?;
    build_tests(registry.history(scale)?, &out_dir.join("history.rs"))?;
    build_tests(registry.pvm()?, &out_dir.join("pvm.rs"))?;
    build_pvmc_tests(registry.pvm()?, &out_dir.join("pvmc.rs"))?;
    build_tests(registry.preimages(scale)?, &out_dir.join("preimages.rs"))?;
    build_tests(registry.reports(scale)?, &out_dir.join("reports.rs"))?;
    build_tests(registry.safrole(scale)?, &out_dir.join("safrole.rs"))?;
    build_tests(registry.shuffle()?, &out_dir.join("shuffle.rs"))?;
    build_tests(registry.statistics(scale)?, &out_dir.join("statistics.rs"))?;
    build_tests(registry.trie()?, &out_dir.join("trie.rs"))?;
    if scale == Scale::Tiny {
        build_tests(
            registry.trace(Trace::Fallback)?,
            &out_dir.join("traces_fallback.rs"),
        )?;
        build_tests(
            registry.trace(Trace::Fuzzy)?,
            &out_dir.join("traces_fuzzy.rs"),
        )?;
        build_tests(
            registry.trace(Trace::FuzzyLight)?,
            &out_dir.join("traces_fuzzy_light.rs"),
        )?;
        build_tests(
            registry.trace(Trace::Safrole)?,
            &out_dir.join("traces_safrole.rs"),
        )?;
        build_tests(
            registry.trace(Trace::Preimages)?,
            &out_dir.join("traces_preimages.rs"),
        )?;
        build_tests(
            registry.trace(Trace::PreimagesLight)?,
            &out_dir.join("traces_preimages_light.rs"),
        )?;
        build_tests(
            registry.trace(Trace::Storage)?,
            &out_dir.join("traces_storage.rs"),
        )?;
        build_tests(
            registry.trace(Trace::StorageLight)?,
            &out_dir.join("traces_storage_light.rs"),
        )?;
    } else {
        for name in [
            "traces_fallback",
            "traces_fuzzy",
            "traces_fuzzy_light",
            "traces_safrole",
            "traces_preimages",
            "traces_preimages_light",
            "traces_storage",
            "traces_storage_light",
        ] {
            fs::write(out_dir.join(format!("{name}.rs")), "")?;
        }
    }
    build_all_seq_test(&out_dir.join("traces_seq.rs"), scale)?;
    Ok(())
}

fn build_tests(entry: Entry, out: &Path) -> Result<()> {
    let mut tests: Vec<ItemFn> = Vec::new();
    let section = entry.section;
    let ss = section.as_ref();
    let registry = util::scale_constructor();
    for (i, path) in entry.files.iter().enumerate() {
        let name = Entry::file_name(path)?;
        let test_name = Ident::new(&format!("test_{name}"), Span::call_site());
        tests.push(parse_quote! {
            #[tokio::test]
            async fn #test_name() {
                let test = #registry.entry(#ss).unwrap().get(#i).unwrap();
                crate::Runner::step(&test).await.expect("failed to run test");
            }
        });
    }

    fs::write(out, quote::quote!(#(#tests)*).to_token_stream().to_string())?;
    Ok(())
}

fn build_pvmc_tests(entry: Entry, out: &Path) -> Result<()> {
    let mut tests: Vec<ItemFn> = Vec::new();
    let section = entry.section;
    let ss = section.as_ref();
    let registry = util::scale_constructor();
    for (i, path) in entry.files.iter().enumerate() {
        let name = Entry::file_name(path)?;
        let test_name = Ident::new(&format!("test_{name}"), Span::call_site());
        tests.push(parse_quote! {
            #[test]
            fn #test_name() {
                let test = #registry.entry(#ss).unwrap().get(#i).unwrap();
                Runner::step(&test).expect("failed to run test");
            }
        });
    }

    fs::write(out, quote::quote!(#(#tests)*).to_token_stream().to_string())?;
    Ok(())
}

/// Builds all sequential tests
fn build_all_seq_test(out: &Path, scale: Scale) -> Result<()> {
    let mut items = Vec::new();
    if scale == Scale::Tiny {
        for entry in [TRACES, REPORTS] {
            for entry in fs::read_dir(entry)? {
                let entry = entry?;
                let path = entry.path();
                if path.is_dir() {
                    let testset = path.to_str().expect("failed to get testset");
                    let name = Path::new(testset)
                        .file_name()
                        .and_then(|s| s.to_str())
                        .expect("failed to get testset name");
                    items.push(build_seq_test(testset, name)?);
                }
            }
        }
    }

    // report from fuzzer
    if Path::new(REPORT).is_dir() && util::report_scale()? == Some(scale) {
        items.push(build_seq_test(REPORT, "report")?);
    }

    // session from fuzzer
    if Path::new(SESSION).is_dir() && util::session_scale()? == Some(scale) {
        items.push(build_seq_test(SESSION, "session")?);
    }

    fs::write(out, quote::quote!(#(#items)*).to_token_stream().to_string())?;
    Ok(())
}

/// Builds the sequential tests
fn build_seq_test(entry: &str, test_name: &str) -> Result<ItemFn> {
    let fentry = Entry::seq(entry)?;
    let mut tests = BTreeSet::<String>::new();
    for path in &fentry.files {
        let name = Entry::file_name(path)?;
        let names = name.split('_').collect::<Vec<&str>>();
        let fname = names.last().unwrap().to_string();
        if fname.contains("genesis") || fname.parse::<u64>().is_err() {
            continue;
        }
        tests.insert(fname);
    }

    // Create function with proper name
    let test_name_ident = Ident::new(&format!("test_{test_name}"), Span::call_site());
    let mut testfn: ItemFn = parse_quote! {
        #[tokio::test]
        async fn #test_name_ident() {
            let mut processor = Processor::default();
        }
    };

    for fname in tests {
        testfn.block.stmts.push(parse_quote! {
            processor.process(specjam::Entry::seq(#entry).unwrap().test(#fname).unwrap()).await.unwrap();
        });
    }

    Ok(testfn)
}

mod util {
    use super::*;

    pub fn try_download(workspace: &Path) -> Result<()> {
        fs::create_dir_all(workspace.join("res"))?;
        clone_if_missing(
            workspace,
            "https://github.com/spacejamapp/jam-test-vectors",
            "res/jam-test-vectors",
        )?;
        clone_if_missing(
            workspace,
            "https://github.com/spacejamapp/jam-conformance",
            "res/jam-conformance",
        )?;
        Ok(())
    }

    pub fn clone_if_missing(workspace: &Path, url: &str, dest: &str) -> Result<()> {
        if workspace.join(dest).exists() {
            return Ok(());
        }
        let output = Command::new("git")
            .args(["clone", url, dest, "--depth", "1"])
            .current_dir(workspace)
            .output()
            .map_err(|e| anyhow::anyhow!("failed to spawn `git clone {url}`: {e}"))?;
        if !output.status.success() {
            anyhow::bail!(
                "git clone {url} into {dest} failed ({}): {}",
                output.status,
                String::from_utf8_lossy(&output.stderr).trim()
            );
        }
        Ok(())
    }

    /// Read a fuzz `report.json` at the given path and return the spec scale
    /// it declares, or None if the file is missing or unrecognized.
    pub fn spec_from_report(path: &Path) -> Result<Option<Scale>> {
        if !path.exists() {
            return Ok(None);
        }
        let content = fs::read_to_string(path)?;
        // Strip whitespace so we tolerate any pretty-printing of the JSON.
        let stripped: String = content.chars().filter(|c| !c.is_whitespace()).collect();
        if stripped.contains(r#""jam_spec":{"full""#) {
            Ok(Some(Scale::Full))
        } else if stripped.contains(r#""jam_spec":{"tiny""#) {
            Ok(Some(Scale::Tiny))
        } else {
            Ok(None)
        }
    }

    /// Spec declared by `res/report/report.json`, if present.
    pub fn report_scale() -> Result<Option<Scale>> {
        spec_from_report(&Path::new(REPORT).join("report.json"))
    }

    /// Spec declared by `res/session/report/report.json`, if present.
    pub fn session_scale() -> Result<Option<Scale>> {
        spec_from_report(
            &Path::new(SESSION)
                .with_file_name("report")
                .join("report.json"),
        )
    }

    pub fn scale_constructor() -> proc_macro2::TokenStream {
        if env::var("CARGO_FEATURE_FULL").is_ok() {
            quote::quote!(specjam::Registry::with_scale(
                "../../res/jam-test-vectors",
                specjam::Scale::Full
            ))
        } else {
            quote::quote!(specjam::Registry::new("../../res/jam-test-vectors"))
        }
    }
}
