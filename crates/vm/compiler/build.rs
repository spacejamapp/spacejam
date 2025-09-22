use std::env;
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=csrc/setjmp.rs.c");
    println!("cargo:rerun-if-changed=csrc/setjmp.rs.h");

    // Get target and host
    let target = env::var("TARGET").expect("TARGET is not set");
    let host = env::var("HOST").expect("HOST is not set");
    let is_cross_compiling = target != host;
    let target_arch = env::var("CARGO_CFG_TARGET_ARCH").expect("CARGO_CFG_TARGET_ARCH is not set");
    let target_os = env::var("CARGO_CFG_TARGET_OS").expect("CARGO_CFG_TARGET_OS is not set");

    // Compile C setjmp wrapper with proper feature macros for cross-platform compatibility
    let mut build = cc::Build::new();
    build
        .file("csrc/setjmp.rs.c")
        .include("csrc")
        .flag_if_supported("-Wno-implicit-function-declaration");

    // Define target-specific macros like Wasmtime does
    build.define(&format!("CFG_TARGET_OS_{}", target_os), None);
    build.define(&format!("CFG_TARGET_ARCH_{}", target_arch), None);

    // Platform-specific compiler flags based on target OS, not host
    build.define("_POSIX_C_SOURCE", "200809L");
    match target_os.as_str() {
        "macos" => {
            build.define("_DARWIN_C_SOURCE", "1");
        }
        "linux" | "android" => {
            build.define("_GNU_SOURCE", "1");
        }
        "freebsd" | "dragonfly" | "netbsd" | "openbsd" => {
            // BSD platforms
        }
        _ => {
            // Other Unix-like platforms
        }
    }

    build.compile("setjmp");

    // Generate bindings for the C wrapper
    let mut bindgen_builder = bindgen::Builder::default()
        .header("csrc/setjmp.rs.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .allowlist_function("setjmp_rs")
        .allowlist_function("longjmp_rs")
        .allowlist_function("install_signal_handlers")
        .allowlist_type("pvm_siginfo_t");

    // Configure bindgen for cross-compilation
    if is_cross_compiling {
        let gcc_name = match target.as_str() {
            "aarch64-unknown-linux-gnu" => "aarch64-unknown-linux-gnu-gcc",
            "x86_64-unknown-linux-gnu" => "x86_64-unknown-linux-gnu-gcc",
            "arm-unknown-linux-gnueabihf" => "arm-unknown-linux-gnueabihf-gcc",
            _ => "",
        };

        if !gcc_name.is_empty() {
            // Try to find the cross-compiler sysroot
            if let Ok(output) = std::process::Command::new(gcc_name)
                .args(["-print-sysroot"])
                .output()
            {
                let sysroot = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !sysroot.is_empty() && std::path::Path::new(&sysroot).exists() {
                    bindgen_builder = bindgen_builder
                        .clang_arg(format!("--sysroot={}", sysroot))
                        .clang_arg("-target")
                        .clang_arg(&target);
                }
            } else if target == "aarch64-unknown-linux-gnu" {
                // Fallback to known homebrew location for aarch64
                let sysroot = "/opt/homebrew/Cellar/aarch64-unknown-linux-gnu/13.3.0/toolchain/aarch64-unknown-linux-gnu/sysroot";
                if std::path::Path::new(sysroot).exists() {
                    bindgen_builder = bindgen_builder
                        .clang_arg(format!("--sysroot={}", sysroot))
                        .clang_arg("-target")
                        .clang_arg("aarch64-unknown-linux-gnu");
                }
            }
        }
    }

    let bindings = bindgen_builder
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set"));
    bindings
        .write_to_file(out_path.join("setjmp.rs"))
        .expect("Couldn't write bindings!");
}
