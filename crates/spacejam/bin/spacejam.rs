//! Spacejam binary.

use spacejam::cmd::App;

#[tokio::main]
async fn main() {
    App::run().await;
}
