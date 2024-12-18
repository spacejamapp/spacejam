use anyhow::Result;
use git2::{build::RepoBuilder, FetchOptions};
use proc_macro2::Span;
use quote::ToTokens;
use std::{
    env, fs,
    path::{Path, PathBuf},
};
use syn::{parse_quote, Ident, ItemFn, LitStr};

const REPO: &str = "https://github.com/clearloop/jam-test-vectors.git";
const INTO: &str = "jamtestvectors";

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=jamtestvectors");

    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join(INTO);
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    download_tests()?;
    build_tests(&root.join("pvm/programs"), &out_dir.join("pvm.rs"))
}

fn build_tests(tests: &Path, out_dir: &Path) -> Result<()> {
    let json_tests = fs::read_dir(tests)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().unwrap_or_default() == "json")
        .collect::<Vec<_>>();

    let mut tests: Vec<ItemFn> = Vec::new();
    for json in json_tests {
        let path = json.path().with_extension("");
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .expect("path must have a file name");

        let json_path: LitStr = LitStr::new(
            json.path().to_str().expect("path must be a valid string"),
            Span::call_site(),
        );

        let test_name = Ident::new(&format!("test_{name}"), Span::call_site());
        tests.push(parse_quote! {
            #[test]
            fn #test_name() {
                crate::init_tracing();
                let test = Test::from_json(include_str!(#json_path)).expect(&format!("Failed to parse {}", #json_path));
                test.run();
            }
        });

        fs::write(
            out_dir,
            quote::quote! {
                #(#tests)*
            }
            .to_token_stream()
            .to_string(),
        )?;
    }

    Ok(())
}

fn download_tests() -> Result<()> {
    let into = Path::new(INTO);
    if into.exists() {
        return Ok(());
    }

    let mut builder = RepoBuilder::new();
    let mut opts = FetchOptions::new();

    opts.depth(1);
    builder.fetch_options(opts);
    builder.clone(REPO, into)?;
    Ok(())
}
