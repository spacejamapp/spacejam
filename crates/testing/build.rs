use anyhow::Result;
use proc_macro2::Span;
use quote::ToTokens;
use specjam::{Entry, Registry, Scale};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use syn::{parse_quote, Ident, ItemFn};

fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let workspace = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?);

    // set up the registry
    self::try_download(&workspace)?;
    let stf = workspace.join("../../res/jam-test-vectors");
    let registry = Registry::new(stf.to_path_buf());

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
    build_tests(registry.codec()?, &out_dir.join("codec.rs"))?;
    build_tests(
        registry.disputes(Scale::Tiny)?,
        &out_dir.join("disputes.rs"),
    )?;
    build_tests(registry.history()?, &out_dir.join("history.rs"))?;
    build_tests(registry.pvm()?, &out_dir.join("pvm.rs"))?;
    build_tests(registry.preimages()?, &out_dir.join("preimages.rs"))?;
    build_tests(registry.reports(Scale::Tiny)?, &out_dir.join("reports.rs"))?;
    build_tests(registry.safrole(Scale::Tiny)?, &out_dir.join("safrole.rs"))?;
    build_tests(registry.shuffle()?, &out_dir.join("shuffle.rs"))?;
    build_tests(
        registry.statistics(Scale::Tiny)?,
        &out_dir.join("statistics.rs"),
    )?;
    build_tests(registry.trie()?, &out_dir.join("trie.rs"))?;

    Ok(())
}

/// Builds the PVM tests
fn build_tests(entry: Entry, out: &Path) -> Result<()> {
    let mut tests: Vec<ItemFn> = Vec::new();
    let section = entry.section.clone();
    let ss = section.as_ref();

    for (i, test) in entry.into_iter().enumerate() {
        let name = &test.name;
        let test_name = Ident::new(&format!("test_{name}"), Span::call_site());

        tests.push(parse_quote! {
            #[test]
            fn #test_name() {
                let test = specjam::Registry::new("../../res/jam-test-vectors").entry(#ss).unwrap().get(#i).unwrap();
                crate::Runner::step(&test).expect("failed to run test");
            }
        });

        fs::write(out, quote::quote!(#(#tests)*).to_token_stream().to_string())?;
    }

    Ok(())
}

fn try_download(workspace: &Path) -> Result<()> {
    if !workspace.join("res").exists() {
        fs::create_dir_all(workspace.join("res"))?;
        Command::new("git")
            .args(&[
                "clone",
                "https://github.com/spacejamapp/jam-test-vectors",
                "res/jam-test-vectors",
                "--depth",
                "1",
            ])
            .current_dir(&workspace)
            .output()?;
    }

    Ok(())
}
