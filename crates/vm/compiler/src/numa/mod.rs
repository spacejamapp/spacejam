//! NUMA-aware process placement for the AOT compiler.
//!
//! The process is left unpinned so default rayon (sig batches, merkle, etc.)
//! keeps using every cgroup-allowed CPU. [`pool`] returns a dedicated rayon
//! pool whose workers are pinned to one NUMA node — used by PVM dispatch so
//! AOT code-cache locality is preserved across nested `par_iter`s.
//!
//! Note: only the AOT ([`crate::exec::Executable`]) path is hinted; cranelift's
//! JIT manages its own code memory and bypasses [`hint_code_pages`].

use std::sync::OnceLock;

#[cfg(target_os = "linux")]
mod linux;

static PLAN: OnceLock<NumaPlan> = OnceLock::new();
static POOL: OnceLock<rayon::ThreadPool> = OnceLock::new();

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
    POOL.get_or_init(|| {
        let plan = init();
        let cpus = plan.cpus.clone();
        rayon::ThreadPoolBuilder::new()
            .num_threads(plan.num_threads)
            .thread_name(|i| format!("numa-{i}"))
            .start_handler(move |_| pin_worker(&cpus))
            .build()
            .expect("build numa rayon pool")
    })
}

/// Hint that an AOT code mmap should use huge pages and bind to the chosen
/// node. Call before the first byte is written. No-op on non-Linux.
pub fn hint_code_pages(addr: *mut u8, size: usize) {
    #[cfg(target_os = "linux")]
    linux::hint_code_pages(addr, size);
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (addr, size);
    }
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

fn pin_worker(cpus: &[usize]) {
    #[cfg(target_os = "linux")]
    if let Err(err) = linux::set_affinity(cpus) {
        tracing::warn!("numa: worker pin failed: {err}");
    }
    #[cfg(not(target_os = "linux"))]
    let _ = cpus;
}

fn detect() -> NumaPlan {
    #[cfg(target_os = "linux")]
    return linux::detect();

    #[cfg(not(target_os = "linux"))]
    fallback()
}
