use anyhow::Result;
use proc_macro2::Span;
use quote::ToTokens;
use specjam::{Entry, Registry};
use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};
use syn::{parse_quote, Ident, ItemFn};

fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let workspace = PathBuf::from(env::var("CARGO_MANIFEST_DIR")?).join("../../..");

    // set up the registry
    try_download(&workspace)?;
    let registry = Registry::new(workspace.join("res/jam-test-vectors"));

    // build PVM tests for compiler
    build_pvm_tests(registry.pvm()?, &out_dir.join("pvm_compiler_tests.rs"))?;

    Ok(())
}

/// Builds the PVM compiler tests
fn build_pvm_tests(entry: Entry, out: &Path) -> Result<()> {
    let mut tests: Vec<ItemFn> = Vec::new();
    let section = entry.section;
    let ss = section.as_ref();

    for (i, test) in entry.into_iter().enumerate() {
        let name = &test.name;
        let test_name = Ident::new(&format!("test_compiler_{name}"), Span::call_site());

        tests.push(parse_quote! {
            #[test]
            fn #test_name() {
                let test = specjam::Registry::new("../../../res/jam-test-vectors").entry(#ss).unwrap().get(#i).unwrap();
                crate::Runner::step(&test).expect("failed to run compiler test");
            }
        });
    }

    fs::write(out, quote::quote!(#(#tests)*).to_token_stream().to_string())?;
    Ok(())
}

fn try_download(workspace: &Path) -> Result<()> {
    if !workspace.join("res/jam-test-vectors").exists() {
        fs::create_dir_all(workspace.join("res"))?;
        Command::new("git")
            .args([
                "clone",
                "https://github.com/spacejamapp/jam-test-vectors",
                "res/jam-test-vectors",
                "--depth",
                "1",
            ])
            .current_dir(workspace)
            .output()?;
    }

    Ok(())
}
