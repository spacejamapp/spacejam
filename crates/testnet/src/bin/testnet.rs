//! The testnet binary.

use clap::Parser;
use spacejam_testnet::App;

fn main() {
    let app = App::parse();
    if let Err(e) = app.run() {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
