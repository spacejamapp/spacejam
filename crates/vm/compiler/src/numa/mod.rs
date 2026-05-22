//! NUMA-aware process placement for the AOT compiler.

use std::sync::OnceLock;

#[cfg(target_os = "linux")]
mod linux;

static PLAN: OnceLock<NumaPlan> = OnceLock::new();
static NARROW: OnceLock<rayon::ThreadPool> = OnceLock::new();

/// Topology decision applied at startup.
#[derive(Debug, Clone)]
pub struct NumaPlan {
    /// Chosen NUMA node, or `None` on UMA / non-Linux / on failure to pin.
    pub node: Option<u32>,
    /// CPUs the process is allowed to run on after pinning.
    pub cpus: Vec<usize>,
    /// Suggested worker count for thread pools (`cpus.len()`, never zero).
    pub num_threads: usize,
}

/// Detect topology, pin the process, cache and return the plan.
pub fn init() -> &'static NumaPlan {
    PLAN.get_or_init(detect)
}

/// Chosen NUMA node, if any. `None` before [`init`] runs.
pub fn chosen_node() -> Option<u32> {
    PLAN.get().and_then(|p| p.node)
}

/// Rayon pool whose workers are pinned to the chosen node's CPUs.
pub fn pool() -> &'static rayon::ThreadPool {
    NARROW.get_or_init(|| {
        let plan = init();
        let cpus = plan.cpus.clone();
        rayon::ThreadPoolBuilder::new()
            .num_threads(plan.num_threads)
            .thread_name(|i| format!("numa-narrow-{i}"))
            .start_handler(move |_| pin_worker(&cpus))
            .build()
            .expect("build narrow rayon pool")
    })
}

#[cfg(target_os = "linux")]
fn pin_worker(cpus: &[usize]) {
    if let Err(err) = linux::set_affinity(cpus) {
        tracing::warn!("numa: narrow worker pin failed: {err}");
    }
}

#[cfg(not(target_os = "linux"))]
fn pin_worker(_cpus: &[usize]) {}

#[cfg(target_os = "linux")]
fn detect() -> NumaPlan {
    linux::detect()
}

#[cfg(not(target_os = "linux"))]
fn detect() -> NumaPlan {
    fallback()
}

/// Fallback NUMA plan for non-Linux platforms.
pub fn fallback() -> NumaPlan {
    let num_threads = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    NumaPlan {
        node: None,
        cpus: (0..num_threads).collect(),
        num_threads,
    }
}
