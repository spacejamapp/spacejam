//! Spacejam binary.

use spacejam::cmd::App;

#[tokio::main]
async fn main() {
    self::init_rayon();

    #[cfg(feature = "dhat")]
    dhat::init();

    App::run().await;
}

/// Spec-matched rayon worker cap: matches the upper bound on par_iter sizes
/// across our hot paths.
///
/// - on tiny that's `VALIDATORS_COUNT = 6`;
/// - on full the ed25519 sig batch dominates with ~64 chunks of 32, so
/// 32 workers gives ~2 chunks per worker
#[cfg(all(feature = "tiny", not(feature = "full")))]
const RAYON_DEFAULT_CAP: usize = 6;
#[cfg(feature = "full")]
const RAYON_DEFAULT_CAP: usize = 32;

fn init_rayon() {
    let threads = std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n > 0)
        .unwrap_or_else(|| {
            std::thread::available_parallelism()
                .map(|n| n.get().min(RAYON_DEFAULT_CAP))
                .unwrap_or(RAYON_DEFAULT_CAP.min(4))
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
