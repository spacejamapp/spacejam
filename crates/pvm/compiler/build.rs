use std::env;
use std::path::PathBuf;

fn main() {
    // Compile C setjmp wrapper
    cc::Build::new()
        .file("csrc/setjmp.c")
        .include("csrc")
        .compile("setjmp");

    // Generate bindings for the C wrapper
    let bindings = bindgen::Builder::default()
        .header("csrc/setjmp.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("pvm_setjmp")
        .allowlist_function("pvm_longjmp")
        .allowlist_function("pvm_install_sigsegv_handler")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("setjmp.rs"))
        .expect("Couldn't write bindings!");

    // Tell cargo to rerun if C files change
    println!("cargo:rerun-if-changed=csrc/setjmp.c");
    println!("cargo:rerun-if-changed=csrc/setjmp.h");
}
