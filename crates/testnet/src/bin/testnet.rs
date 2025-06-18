//! The testnet binary.

use clap::Parser;
use spacejam_testnet::App;

fn main() {
    let app = App::parse();
    app.run().expect("failed to run testnet");
}
