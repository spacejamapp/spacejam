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

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../../res/jam-test-vectors");
    println!("cargo:rerun-if-changed=../../res/jam-conformance/fuzz-reports/0.7.2/traces");
    println!("cargo:rerun-if-changed=./build.rs");
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let workspace = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../../");

    // set up the registry
    try_download(&workspace)?;
    let registry = Registry::new(workspace.join("res/jam-test-vectors"));

    // build all tests
    build_tests(
        registry.accumulate(Scale::Tiny)?,
        &out_dir.join("accumulate.rs"),
    )?;
    build_tests(
        registry.assurances(Scale::Tiny)?,
        &out_dir.join("assurances.rs"),
    )?;
    build_tests(
        registry.authorizations(Scale::Tiny)?,
        &out_dir.join("authorizations.rs"),
    )?;
    build_tests(registry.codec(Scale::Tiny)?, &out_dir.join("codec.rs"))?;
    build_tests(
        registry.disputes(Scale::Tiny)?,
        &out_dir.join("disputes.rs"),
    )?;
    build_tests(registry.erasure(Scale::Tiny)?, &out_dir.join("erasure.rs"))?;
    build_tests(registry.history(Scale::Tiny)?, &out_dir.join("history.rs"))?;
    build_tests(registry.pvm()?, &out_dir.join("pvm.rs"))?;
    build_pvmc_tests(registry.pvm()?, &out_dir.join("pvmc.rs"))?;
    build_tests(
        registry.preimages(Scale::Tiny)?,
        &out_dir.join("preimages.rs"),
    )?;
    build_tests(registry.reports(Scale::Tiny)?, &out_dir.join("reports.rs"))?;
    build_tests(registry.safrole(Scale::Tiny)?, &out_dir.join("safrole.rs"))?;
    build_tests(registry.shuffle()?, &out_dir.join("shuffle.rs"))?;
    build_tests(
        registry.statistics(Scale::Tiny)?,
        &out_dir.join("statistics.rs"),
    )?;
    build_tests(registry.trie()?, &out_dir.join("trie.rs"))?;
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

    // build all sequential tests
    build_all_seq_test(&out_dir.join("traces_seq.rs"))?;
    Ok(())
}

/// Builds the PVM tests
fn build_tests(entry: Entry, out: &Path) -> Result<()> {
    let mut tests: Vec<ItemFn> = Vec::new();
    let section = entry.section;
    let ss = section.as_ref();

    // Iterate over file paths directly instead of parsing JSON files to avoid
    // reading thousands of files during build time. Test names are derived from
    // filenames, so parsing is unnecessary.
    for (i, path) in entry.files.iter().enumerate() {
        let name = Entry::file_name(path)?;
        let test_name = Ident::new(&format!("test_{name}"), Span::call_site());
        tests.push(parse_quote! {
            #[tokio::test]
            async fn #test_name() {
                let test = specjam::Registry::new("../../res/jam-test-vectors").entry(#ss).unwrap().get(#i).unwrap();
                crate::Runner::step(&test).await.expect("failed to run test");
            }
        });
    }

    fs::write(out, quote::quote!(#(#tests)*).to_token_stream().to_string())?;
    Ok(())
}

/// Builds the PVM tests
fn build_pvmc_tests(entry: Entry, out: &Path) -> Result<()> {
    let mut tests: Vec<ItemFn> = Vec::new();
    let section = entry.section;
    let ss = section.as_ref();

    // Iterate over file paths directly instead of parsing JSON files to avoid
    // reading thousands of files during build time. Test names are derived from
    // filenames, so parsing is unnecessary.
    for (i, path) in entry.files.iter().enumerate() {
        let name = Entry::file_name(path)?;
        let test_name = Ident::new(&format!("test_{name}"), Span::call_site());
        tests.push(parse_quote! {
            #[test]
            fn #test_name() {
                let test = specjam::Registry::new("../../res/jam-test-vectors").entry(#ss).unwrap().get(#i).unwrap();
                Runner::step(&test).expect("failed to run test");
            }
        });
    }

    fs::write(out, quote::quote!(#(#tests)*).to_token_stream().to_string())?;
    Ok(())
}

/// Builds all sequential tests
fn build_all_seq_test(out: &Path) -> Result<()> {
    let mut items = Vec::new();
    for entry in [TRACES, REPORTS] {
        for entry in fs::read_dir(entry)? {
            let entry = entry?;
            let path = entry.path();
            // let dname = path.file_name().unwrap().to_string_lossy();
            /* if dname.contains("_4872") {
                continue;
            } */

            if path.is_dir() {
                let testset = path.to_str().expect("failed to get testset");
                items.push(build_seq_test(testset)?);
            }
        }
    }

    fs::write(out, quote::quote!(#(#items)*).to_token_stream().to_string())?;
    Ok(())
}

/// Builds the sequential tests
fn build_seq_test(entry: &str) -> Result<ItemFn> {
    let fentry = Entry::seq(entry)?;
    let test_name = Path::new(entry).file_name().unwrap().to_str().unwrap();
    let mut tests = BTreeSet::<String>::new();

    // Iterate over file paths directly instead of parsing JSON files to avoid
    // reading thousands of files during build time. Extract test names from filenames.
    for path in &fentry.files {
        let name = Entry::file_name(path)?;
        let names = name.split('_').collect::<Vec<&str>>();
        let fname = names.last().unwrap().to_string();
        if fname.contains("genesis") {
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

fn try_download(workspace: &Path) -> Result<()> {
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

fn clone_if_missing(workspace: &Path, url: &str, dest: &str) -> Result<()> {
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
