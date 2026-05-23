//! Spacejam binary.

use spacejam::cmd::App;

#[tokio::main]
async fn main() {
    self::init_rayon();

    #[cfg(feature = "dhat")]
    dhat::init();

    App::run().await;
}

/// Cap the rayon global pool at 32
fn init_rayon() {
    let threads = std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get().min(32))
                .unwrap_or(8)
        });

    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .thread_name(|i| format!("rayon-{i}"))
        .build_global();
}

#[cfg(feature = "dhat")]
mod dhat {
    use std::sync::Mutex;

    #[global_allocator]
    static ALLOC: dhat::Alloc = dhat::Alloc;

    pub fn init() {
        // Leak the profiler so the signal handler task can drop it on Ctrl-C.
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
