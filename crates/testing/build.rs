use anyhow::Result;
use proc_macro2::Span;
use quote::ToTokens;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use syn::{parse_quote, Ident, ItemFn};

fn main() -> Result<()> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    build_tests(&out_dir.join("pvm.rs"))
}

fn build_tests(out_dir: &Path) -> Result<()> {
    let pvm_tests = specjam::registry::PVM;

    let mut tests: Vec<ItemFn> = Vec::new();
    for test in pvm_tests {
        let name = test.name;
        let path = test.name.to_uppercase();
        let test_name = Ident::new(&format!("test_{name}"), Span::call_site());

        tests.push(parse_quote! {
            #[test]
            fn #test_name() {
                crate::Runner::step(&paste::paste!([< TEST_PVM_ #path >])).expect("failed to run test");
            }
        });

        fs::write(
            out_dir,
            quote::quote! {
                use specjam::registry::tests::*;

                #(#tests)*
            }
            .to_token_stream()
            .to_string(),
        )?;
    }

    Ok(())
}
