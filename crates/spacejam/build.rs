//! build script for spacejam

use std::{fs, process::Command};

const TINY_DEV_SPEC: &str = "https://gist.githubusercontent.com/clearloop/52b9d5c16d3bd2a2d900b756fc64a9d1/raw/fbf84b774254cb68071a8a37cf8faac699bebf48/spec.json";

fn main() {
    println!("cargo:rerun-if-changed=src/chain/spec.rs");
    println!("cargo:rerun-if-changed=build.rs");

    let root = std::path::PathBuf::from(
        std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR must be set"),
    );

    fuzz::generate(&root).expect("failed to generate fuzz tests");
    let dev = root.join("spec/dev");
    let target = dev.join("spec.json");
    if target.exists() {
        return;
    }

    fs::create_dir_all(&dev).expect("failed to create tiny spec dir");
    Command::new("curl")
        .args([
            TINY_DEV_SPEC,
            "-o",
            target.to_str().expect("failed to convert target to str"),
        ])
        .output()
        .expect("failed to download tiny spec");
}

mod fuzz {
    use proc_macro2::Span;
    use quote::quote;
    use std::{env, fs, path::Path};
    use syn::{Ident, ItemFn, LitStr, parse_quote};

    /// Generate the fuzz tests
    pub fn generate(root: &Path) -> std::io::Result<()> {
        let srouce = root.join("../../res/jam-conformance/fuzz-proto/examples/v1");
        let mut tests: Vec<ItemFn> = Vec::new();
        println!("reading from {srouce:?}");
        for entry in fs::read_dir(srouce)? {
            let entry = entry?;
            let path = entry.path();
            let ext = path.extension().expect("failed to get extension");
            if ext != "bin" {
                continue;
            }

            let fpath = path.with_extension("");
            let fname = fpath
                .file_name()
                .expect("failed to get file name")
                .to_str()
                .expect("failed to convert file name to str");

            let test = Ident::new(&format!("test_{fname}"), Span::call_site());
            let bytes = LitStr::new(path.to_string_lossy().as_ref(), Span::call_site());
            tests.push(parse_quote! {
                #[test]
                fn #test() {
                    let mut bytes = include_bytes!(#bytes).to_vec();
                    if bytes[0] == 255 {
                        bytes[0] = 6;
                    }
                    let message: Message = codec::decode(&bytes).unwrap();
                    let encoded = codec::encode(&message).unwrap();
                    assert_eq!(encoded, bytes);
                }
            });
        }

        let out_dir = env::var("OUT_DIR").expect("OUT_DIR must be set");
        let out = Path::new(&out_dir).join("fuzz.rs");
        println!("writing to {out:?}");
        fs::write(out, quote! { #(#tests)* }.to_string())?;
        Ok(())
    }
}
