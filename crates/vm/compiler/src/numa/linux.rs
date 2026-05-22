//! Linux NUMA detection.

use crate::numa::{NumaPlan, fallback};
use std::collections::BTreeSet;

/// Detect NUMA topology and pick a node for code-local execution.
pub fn detect() -> NumaPlan {
    let allowed: BTreeSet<usize> = match get_allowed_cpus() {
        Some(set) if !set.is_empty() => set,
        _ => return fallback(),
    };

    let buckets: Vec<(u32, Vec<usize>)> = read_node_cpulists()
        .into_iter()
        .map(|(id, cpus)| {
            let kept: Vec<usize> = cpus.into_iter().filter(|c| allowed.contains(c)).collect();
            (id, kept)
        })
        .filter(|(_, cpus)| !cpus.is_empty())
        .collect();

    if buckets.len() <= 1 {
        let cpus: Vec<usize> = allowed.into_iter().collect();
        let num_threads = cpus.len().max(1);
        return NumaPlan {
            node: None,
            cpus,
            num_threads,
        };
    }

    let (node, cpus) = buckets.into_iter().max_by_key(|(_, c)| c.len()).unwrap();
    let num_threads = cpus.len().max(1);
    tracing::info!("numa: chose node {node} ({num_threads} cpus) for code-local execution");
    NumaPlan {
        node: Some(node),
        cpus,
        num_threads,
    }
}

pub fn set_affinity(cpus: &[usize]) -> std::io::Result<()> {
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    for &cpu in cpus {
        unsafe { libc::CPU_SET(cpu, &mut set) };
    }
    let size = std::mem::size_of::<libc::cpu_set_t>();
    if unsafe { libc::sched_setaffinity(0, size, &set) } != 0 {
        return Err(std::io::Error::last_os_error());
    }
    Ok(())
}

/// Hint that the AOT code mapping should use huge pages, and bind its
/// not-yet-faulted pages to the chosen node (if any).
pub fn hint_code_pages(addr: *mut u8, size: usize) {
    if unsafe { libc::madvise(addr.cast(), size, libc::MADV_HUGEPAGE) } != 0 {
        tracing::warn!(
            "numa: madvise(MADV_HUGEPAGE) on AOT code buffer failed: {}",
            std::io::Error::last_os_error()
        );
    }

    if let Some(node) = super::chosen_node() {
        bind_to_node(addr, size, node);
    }
}

fn get_allowed_cpus() -> Option<BTreeSet<usize>> {
    let mut set: libc::cpu_set_t = unsafe { std::mem::zeroed() };
    let size = std::mem::size_of::<libc::cpu_set_t>();
    if unsafe { libc::sched_getaffinity(0, size, &mut set) } != 0 {
        return None;
    }
    let mut allowed = BTreeSet::new();
    for cpu in 0..(libc::CPU_SETSIZE as usize) {
        if unsafe { libc::CPU_ISSET(cpu, &set) } {
            allowed.insert(cpu);
        }
    }
    Some(allowed)
}

fn read_node_cpulists() -> Vec<(u32, Vec<usize>)> {
    let Ok(entries) = std::fs::read_dir("/sys/devices/system/node") else {
        return Vec::new();
    };
    let mut nodes = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name();
        let name = name.to_string_lossy();
        let Some(rest) = name.strip_prefix("node") else {
            continue;
        };
        let Ok(id) = rest.parse::<u32>() else {
            continue;
        };
        let Ok(text) = std::fs::read_to_string(entry.path().join("cpulist")) else {
            continue;
        };
        nodes.push((id, parse_cpulist(text.trim())));
    }
    nodes
}

fn parse_cpulist(s: &str) -> Vec<usize> {
    let mut out = Vec::new();
    for part in s.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        if let Some((a, b)) = part.split_once('-') {
            if let (Ok(a), Ok(b)) = (a.parse::<usize>(), b.parse::<usize>()) {
                out.extend(a..=b);
            }
        } else if let Ok(n) = part.parse::<usize>() {
            out.push(n);
        }
    }
    out
}

fn bind_to_node(addr: *mut u8, size: usize, node: u32) {
    // MPOL_BIND from linux/mempolicy.h; not exposed by the libc crate.
    const MPOL_BIND: libc::c_int = 2;
    if node >= 64 {
        tracing::warn!("numa: chosen node {node} out of mbind range, skipping");
        return;
    }
    let mask: u64 = 1u64 << node;
    let ret = unsafe {
        libc::syscall(
            libc::SYS_mbind,
            addr,
            size as libc::c_ulong,
            MPOL_BIND,
            &mask as *const u64,
            64u64,
            0u32,
        )
    };
    if ret != 0 {
        tracing::warn!(
            "numa: mbind AOT code buffer to node {node} failed: {}",
            std::io::Error::last_os_error()
        );
    }
}
