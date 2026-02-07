//! Spacejam binary.

use spacejam::cmd::App;

#[cfg(feature = "dhat")]
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

#[tokio::main]
async fn main() {
    #[cfg(feature = "dhat")]
    {
        // Leak the profiler so the signal handler task can drop it on Ctrl-C.
        let profiler: &'static std::sync::Mutex<Option<dhat::Profiler>> = Box::leak(Box::new(
            std::sync::Mutex::new(Some(dhat::Profiler::new_heap())),
        ));

        tokio::spawn(async move {
            tokio::signal::ctrl_c().await.ok();
            eprintln!("dhat: caught ctrl-c, writing profile...");
            drop(profiler.lock().unwrap().take());
            std::process::exit(0);
        });
    }

    App::run().await;
}
