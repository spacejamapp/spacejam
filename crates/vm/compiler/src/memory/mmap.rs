//! Memory management for PVM programs using mmap for efficient virtual memory
#![cfg(target_os = "linux")]

use crate::memory::image::MemoryImage;
use anyhow::Result;
use libc::{
    MAP_ANONYMOUS, MAP_FIXED, MAP_NORESERVE, MAP_PRIVATE, PROT_NONE, PROT_READ, PROT_WRITE,
};
use parking_lot::Mutex;
use pvm::{MemoryLike, score::OpaqueHash};
use std::{collections::BTreeMap, io, ptr, sync::LazyLock};

/// Process-wide pool of pre-reserved 4 GB virtual regions.
static REGION_POOL: LazyLock<Mutex<Vec<Region>>> = LazyLock::new(|| Mutex::new(Vec::new()));

/// memory for PVM programs
pub struct Memory {
    /// Backing 4 GB virtual region, returned to the pool on drop.
    region: Region,

    /// Heap pointer
    ///
    /// NOTE: using [u64] for safe mapping in C API
    pub heap_ptr: u32,
}

impl Memory {
    /// Create a new memory instance from parser Memory
    pub fn new(hash: OpaqueHash, pmemory: &pvm::Memory) -> Result<Self> {
        let image = MemoryImage::get_or_build(hash, pmemory)?;
        let region = Region::acquire()?;
        let memory = Memory {
            region,
            heap_ptr: pmemory.heap_ptr,
        };
        memory.init(pmemory, &image)?;
        Ok(memory)
    }

    fn base(&self) -> *mut u8 {
        self.region.as_ptr()
    }

    /// Initialize memory regions from parser memory
    fn init(&self, memory: &pvm::Memory, image: &MemoryImage) -> Result<()> {
        let base = self.base();
        unsafe {
            // Bind RO range CoW from the image
            if image.ro_size > 0 {
                let start = memory.info.read.start as usize;
                let bound = libc::mmap(
                    base.add(start) as *mut _,
                    image.ro_size,
                    PROT_READ,
                    MAP_FIXED | MAP_PRIVATE,
                    image.raw_fd(),
                    0,
                );
                if bound == libc::MAP_FAILED {
                    anyhow::bail!("Failed to bind RO range: {}", io::Error::last_os_error());
                }
            }

            // Bind RW range CoW from the image — writes hit private pages
            if image.rw_size > 0 {
                let start = memory.info.write.start as usize;
                let bound = libc::mmap(
                    base.add(start) as *mut _,
                    image.rw_size,
                    PROT_READ | PROT_WRITE,
                    MAP_FIXED | MAP_PRIVATE,
                    image.raw_fd(),
                    image.ro_size as libc::off_t,
                );
                if bound == libc::MAP_FAILED {
                    anyhow::bail!("Failed to bind RW range: {}", io::Error::last_os_error());
                }
            }

            // Stack: anonymous CoW from the slot reservation, just unlock perms
            {
                let start = memory.info.stack.start as usize;
                let size = (memory.info.stack.end - memory.info.stack.start) as usize;
                if size > 0
                    && libc::mprotect(base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0
                {
                    anyhow::bail!(
                        "Failed to set stack region protection: {}",
                        io::Error::last_os_error()
                    );
                }
            }

            // Args vary per invocation — copy in, then lock read-only
            {
                let args = memory.args()?;
                let start = memory.info.args.start as usize;
                let size = args.len();
                if size > 0 {
                    if libc::mprotect(base.add(start) as *mut _, size, PROT_READ | PROT_WRITE) != 0
                    {
                        anyhow::bail!(
                            "Failed to make args region writable: {}",
                            io::Error::last_os_error()
                        );
                    }
                    ptr::copy_nonoverlapping(args.as_ptr(), base.add(start), size);
                    if libc::mprotect(base.add(start) as *mut _, size, PROT_READ) != 0 {
                        anyhow::bail!(
                            "Failed to set args region read-only: {}",
                            io::Error::last_os_error()
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Read bytes from memory
    #[inline]
    pub fn read_bytes(&self, addr: u32, len: u32) -> &[u8] {
        unsafe { std::slice::from_raw_parts(self.base().add(addr as usize), len as usize) }
    }

    /// Write bytes to memory
    #[inline]
    pub fn write_bytes(&mut self, addr: u32, data: &[u8]) {
        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.base().add(addr as usize), data.len());
        }
    }

    /// Convert the virtual memory back to pvm::Memory structure
    pub fn fill(&self, original: &pvm::Memory) -> pvm::Memory {
        let base = self.base();
        let mut memory_map = BTreeMap::new();
        for (&page_num, (_, perms)) in &original.memory {
            let page_addr = (page_num as usize) * (pvm::PAGE_SIZE as usize);
            let mut page_data = vec![0u8; pvm::PAGE_SIZE as usize];
            unsafe {
                ptr::copy_nonoverlapping(
                    base.add(page_addr),
                    page_data.as_mut_ptr(),
                    pvm::PAGE_SIZE as usize,
                );
            }

            // Only store non-zero pages
            if page_data.iter().any(|&b| b != 0) {
                memory_map.insert(page_num, (page_data, *perms));
            }
        }

        pvm::Memory {
            memory: memory_map,
            info: original.info.clone(),
            heap_ptr: original.heap_ptr,
        }
    }
}

unsafe impl Send for Memory {}
unsafe impl Sync for Memory {}

impl MemoryLike for Memory {
    fn read(&self, addr: u32, len: u32) -> Result<Vec<u8>> {
        Ok(self.read_bytes(addr, len).to_vec())
    }

    fn read_into(&self, addr: u32, buf: &mut [u8]) -> Result<()> {
        pvm::check_range(addr, buf.len() as u32)?;
        let src = self.read_bytes(addr, buf.len() as u32);
        buf.copy_from_slice(src);
        Ok(())
    }

    fn write(&mut self, addr: u32, data: &[u8]) -> Result<()> {
        self.write_bytes(addr, data);
        Ok(())
    }

    fn allocate(&mut self, page: u32, count: u32) -> Result<()> {
        if count == 0 {
            return Ok(());
        }

        let base = self.base();
        for page_num in page..(page + count) {
            let page_addr = (page_num as usize) * (pvm::PAGE_SIZE as usize);
            unsafe {
                if libc::mprotect(
                    base.add(page_addr) as *mut _,
                    pvm::PAGE_SIZE as usize,
                    PROT_READ | PROT_WRITE,
                ) != 0
                {
                    anyhow::bail!(
                        "Failed to commit page {} at addr {:#x}: {}",
                        page_num,
                        page_addr,
                        io::Error::last_os_error()
                    );
                }
            }
        }

        Ok(())
    }

    fn heap_ptr(&self) -> u32 {
        self.heap_ptr
    }

    fn set_heap_ptr(&mut self, heap_ptr: u32) {
        self.heap_ptr = heap_ptr;
    }
}

/// Reusable 4 GB virtual region.
struct Region(*mut u8);

// Safety: a `Region` is only handed out to one owner at a time.
unsafe impl Send for Region {}

impl Region {
    /// Pop from the pool, or `mmap` a fresh 4 GB reservation if empty.
    fn acquire() -> Result<Self> {
        if let Some(region) = REGION_POOL.lock().pop() {
            return Ok(region);
        }
        let base = unsafe {
            libc::mmap(
                ptr::null_mut(),
                pvm::PVM_MEMORY_SIZE as usize,
                PROT_NONE,
                MAP_PRIVATE | MAP_ANONYMOUS | MAP_NORESERVE,
                -1,
                0,
            )
        };
        if base == libc::MAP_FAILED {
            anyhow::bail!(
                "Failed to mmap virtual memory: {}",
                std::io::Error::last_os_error()
            );
        }
        Ok(Region(base as *mut u8))
    }

    fn as_ptr(&self) -> *mut u8 {
        self.0
    }
}

impl Drop for Region {
    fn drop(&mut self) {
        let ptr = std::mem::replace(&mut self.0, ptr::null_mut());
        if ptr.is_null() {
            return;
        }

        // Reset the region to a known state
        let size = pvm::PVM_MEMORY_SIZE as usize;
        let madv_ok = unsafe { libc::madvise(ptr as *mut _, size, libc::MADV_DONTNEED) } == 0;
        let mp_ok = unsafe { libc::mprotect(ptr as *mut _, size, PROT_NONE) } == 0;
        if madv_ok && mp_ok {
            REGION_POOL.lock().push(Region(ptr));
        } else {
            tracing::error!(
                "Region reset failed (madvise_ok={madv_ok}, mprotect_ok={mp_ok}); dropping slot to avoid cross-invocation data leak"
            );
            unsafe {
                libc::munmap(ptr as *mut _, size);
            }
        }
    }
}
