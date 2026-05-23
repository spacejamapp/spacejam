//! NUMA-aware hints for AOT code mappings.
//!
//! Picks one NUMA node at startup ([`init`]) and binds AOT code pages to it
//! via [`bind_pages`] (mbind + MADV_HUGEPAGE), so JIT'd code (MB-scale) stays
//! local to the cores that execute it. No thread pinning, no separate rayon
//! pool — rayon's global pool keeps balancing across all CPUs.

use std::sync::OnceLock;

#[cfg(target_os = "linux")]
mod linux;

static PLAN: OnceLock<NumaPlan> = OnceLock::new();

/// Topology decision applied at startup.
#[derive(Debug, Clone)]
pub struct NumaPlan {
    /// Chosen NUMA node, or `None` on UMA / non-Linux / on detection failure.
    pub node: Option<u32>,
}

/// Detect topology, cache and return the plan.
pub fn init() -> &'static NumaPlan {
    PLAN.get_or_init(detect)
}

/// Chosen NUMA node, if any. `None` before [`init`] runs.
pub fn chosen_node() -> Option<u32> {
    PLAN.get().and_then(|p| p.node)
}

/// Bind the given mapping to the chosen NUMA node and hint it for huge
/// pages. Call before the first byte is faulted in — `mbind` only affects
/// not-yet-faulted pages. No-op on non-Linux.
pub fn bind_pages(addr: *mut u8, size: usize) {
    #[cfg(target_os = "linux")]
    linux::bind_pages(addr, size);
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (addr, size);
    }
}

fn detect() -> NumaPlan {
    #[cfg(target_os = "linux")]
    return linux::detect();

    #[cfg(not(target_os = "linux"))]
    NumaPlan { node: None }
}
