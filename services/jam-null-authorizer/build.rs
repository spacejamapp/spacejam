//! Build the service

fn main() {
    cjam::util::build(env!("CARGO_PKG_NAME"), None).expect("Failed to build service");
}
