//! Generate test certificates and keys for the network
use std::{fs, path::Path};

const TEST_CERT: &str = "tests/data/test.server.cert";
const TEST_PEM: &str = "tests/data/test.server.pkcs8.pem";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if Path::new(TEST_CERT).exists() && Path::new(TEST_PEM).exists() {
        return;
    }

    let cert = rcgen::generate_simple_self_signed(vec!["spacejam".into()]).unwrap();
    fs::write(TEST_CERT, cert.cert.pem()).expect("failed to write certificate");
    fs::write(TEST_PEM, cert.key_pair.serialize_pem()).expect("failed to write private key");
}
