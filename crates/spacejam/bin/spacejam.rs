//! Spacejam binary.

use spacejam::cmd::App;

#[tokio::main]
async fn main() {
    #[cfg(feature = "dhat")]
    dhat::init();

    App::run().await;
}

#[cfg(feature = "dhat")]
mod dhat {
    use std::sync::Mutex;

    #[global_allocator]
    static ALLOC: dhat::Alloc = dhat::Alloc;

    /// Initialize the dhat profiler.
    pub fn init() {
        let profiler: &'static Mutex<Option<dhat::Profiler>> =
            Box::leak(Box::new(Mutex::new(Some(dhat::Profiler::new_heap()))));

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            eprintln!("dhat: caught ctrl-c, writing profile...");
            drop(profiler.lock().unwrap().take());
            std::process::exit(0);
        });
    }
}
