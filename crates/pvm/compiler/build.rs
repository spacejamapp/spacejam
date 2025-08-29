use std::env;
use std::path::PathBuf;

fn main() {
    // Compile C setjmp wrapper with proper feature macros for cross-platform compatibility
    let mut build = cc::Build::new();
    build
        .file("csrc/setjmp.rs.c")
        .include("csrc")
        .flag_if_supported("-Wno-implicit-function-declaration");

    // Platform-specific compiler flags
    if cfg!(target_os = "macos") {
        // macOS specific flags - use Darwin source features
        build.define("_DARWIN_C_SOURCE", "1");
        build.define("_POSIX_C_SOURCE", "200809L");
    } else {
        // Linux and other Unix platforms
        build.define("_POSIX_C_SOURCE", "200809L");
        build.define("_GNU_SOURCE", "1");
    }

    build.compile("setjmp");

    // Generate bindings for the C wrapper
    let bindings = bindgen::Builder::default()
        .header("csrc/setjmp.rs.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("setjmp_rs")
        .allowlist_function("longjmp_rs")
        .allowlist_function("install_signal_handlers")
        .allowlist_type("pvm_siginfo_t")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("setjmp.rs"))
        .expect("Couldn't write bindings!");

    // Tell cargo to rerun if C files change
    println!("cargo:rerun-if-changed=csrc/setjmp.rs.c");
    println!("cargo:rerun-if-changed=csrc/setjmp.rs.h");
}
