//! Generate test certificates and keys for the network
use std::{path::Path, process::Command};

const TEST_CRT: &str = "tests/data/test.server.cert";
const TEST_PEM: &str = "tests/data/test.server.pkcs8.pem";
const GENERATE_ARGS: [&str; 13] = [
    "req",
    "-x509",
    "-newkey",
    "rsa:2048",
    "-keyout",
    TEST_PEM,
    "-out",
    TEST_CRT,
    "--days",
    "365",
    "--nodes",
    "-subj",
    "/CN=Test Server",
];

fn main() {
    if Path::new(TEST_CRT).exists() && Path::new(TEST_PEM).exists() {
        return;
    }

    let status = Command::new("openssl")
        .args(GENERATE_ARGS)
        .status()
        .expect("Failed to generate test certificate and key");

    if !status.success() {
        panic!("Failed to generate test certificate and key");
    }
}
