//! Per-program memfd image cache for guest memory init data.
//!
//! ref: https://github.com/bytecodealliance/wasmtime/pull/3697
#![cfg(target_os = "linux")]

use anyhow::Result;
use pvm::{Cache, score::OpaqueHash};
use std::{
    io,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    sync::{Arc, LazyLock},
};

/// Process-wide cache of per-program memory images.
static IMAGES: LazyLock<Cache<MemoryImage>> = LazyLock::new(Cache::default);

/// Memfd-backed snapshot of a program's RO + RW initial bytes.
pub struct MemoryImage {
    memfd: OwnedFd,
    /// The size of the read-only data.
    pub ro_size: usize,
    /// The size of the read-write data.
    pub rw_size: usize,
}

impl MemoryImage {
    /// Look up the image for `hash`, building from `pmemory` on miss.
    pub fn get_or_build(hash: OpaqueHash, pmemory: &pvm::Memory) -> Result<Arc<Self>> {
        if let Some(image) = IMAGES.get(&hash) {
            return Ok(image);
        }
        let image = Arc::new(Self::build(pmemory)?);
        IMAGES.put(hash, image.clone());
        Ok(image)
    }

    /// Get the raw file descriptor of the memory image.
    pub fn raw_fd(&self) -> libc::c_int {
        self.memfd.as_raw_fd()
    }

    fn build(pmemory: &pvm::Memory) -> Result<Self> {
        let ro = pmemory.ro_data()?;
        let rw = pmemory.rw_data()?;
        let page = pvm::PAGE_SIZE as usize;
        let ro_size = ro.len().next_multiple_of(page);
        let rw_size = rw.len().next_multiple_of(page);
        let total = ro_size + rw_size;
        let name = c"spacejam-pvm-image";
        let raw = unsafe { libc::memfd_create(name.as_ptr(), libc::MFD_CLOEXEC) };
        if raw < 0 {
            anyhow::bail!("memfd_create failed: {}", io::Error::last_os_error());
        }

        let memfd = unsafe { OwnedFd::from_raw_fd(raw) };
        if total > 0 {
            if unsafe { libc::ftruncate(memfd.as_raw_fd(), total as libc::off_t) } != 0 {
                anyhow::bail!("ftruncate failed: {}", io::Error::last_os_error());
            }
            if !ro.is_empty() {
                Self::pwrite_all(&memfd, &ro, 0)?;
            }
            if !rw.is_empty() {
                Self::pwrite_all(&memfd, &rw, ro_size as libc::off_t)?;
            }
        }

        Ok(Self {
            memfd,
            ro_size,
            rw_size,
        })
    }

    fn pwrite_all(fd: &OwnedFd, buf: &[u8], mut offset: libc::off_t) -> Result<()> {
        let mut remaining = buf.len();
        let mut ptr = buf.as_ptr();
        while remaining > 0 {
            let written =
                unsafe { libc::pwrite(fd.as_raw_fd(), ptr as *const _, remaining, offset) };
            if written < 0 {
                let err = io::Error::last_os_error();
                if err.kind() == io::ErrorKind::Interrupted {
                    continue;
                }
                anyhow::bail!("pwrite failed: {err}");
            }
            let n = written as usize;
            remaining -= n;
            offset += n as libc::off_t;
            ptr = unsafe { ptr.add(n) };
        }
        Ok(())
    }
}
