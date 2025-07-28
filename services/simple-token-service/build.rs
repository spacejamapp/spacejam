//! Build the service

fn main() {
    cjam::util::build(env!("CARGO_PKG_NAME")).expect("Failed to build service");
}
