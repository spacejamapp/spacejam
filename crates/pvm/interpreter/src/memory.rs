//! Memory management for the interpreter

use crate::{Error, Result};
use pvm::{Reason, Value};
use smallvec::SmallVec;
use std::borrow::Cow;
use std::collections::BTreeMap;

/// The size of a page in the memory.
pub const PAGE_SIZE: u32 = 4096;

/// The memory of the interpreter.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Memory {
    /// The pages of the memory.
    pub pages: BTreeMap<u32, Page>,

    /// Current heap pointer for sbrk implementation
    pub current_heap_pointer: u32,

    /// The initial heap pointer
    initial_heap: u32,
}

impl Memory {
    /// Read a value from the memory.
    pub fn read<V: Value>(&mut self, address: u32) -> Result<V> {
        self.read_offset(address, 0)
    }

    /// Read a value from the memory at an offset.
    pub fn read_offset<V: Value>(&mut self, address: u32, offset: u32) -> Result<V> {
        let start = address + offset;
        let page = start / PAGE_SIZE;

        let bytes = self.read_bytes(page, start % PAGE_SIZE, V::SIZE as u32)?;
        V::from_bytes(&bytes).ok_or(Error::MemoryInaccessible { page })
    }

    /// Read bytes from the memory.
    pub fn read_bytes(&self, page: u32, offset: u32, len: u32) -> Result<Vec<u8>> {
        let page_data = self.access(page)?;
        let data = page_data.data.as_slice();
        let data_len = data.len() as u32;

        // fill with 0s if necessary
        let mut bytes = vec![0; len as usize];

        if offset < data_len {
            let to_copy = (data_len - offset).min(len) as usize;
            bytes[..to_copy].copy_from_slice(&data[offset as usize..(offset as usize + to_copy)]);
        }

        Ok(bytes)
    }

    /// Write a value to the memory.
    pub fn write<V: Value>(&mut self, address: u32, value: V) -> Result<()> {
        self.write_bytes(address / PAGE_SIZE, address % PAGE_SIZE, &value.to_vec())
    }

    /// Write a value to the memory at an offset.
    pub fn write_offset<V: Value>(&mut self, address: u32, offset: u32, value: V) -> Result<()> {
        let start = address.wrapping_add(offset);
        let page = start / PAGE_SIZE;
        let offset = start % PAGE_SIZE;
        if offset + V::SIZE as u32 > PAGE_SIZE {
            return Err(Error::MemoryInaccessible { page });
        }

        self.write_bytes(page, offset, &value.to_vec())
    }

    /// Write bytes to the memory.
    pub fn write_bytes(&mut self, page: u32, offset: u32, bytes: &[u8]) -> Result<()> {
        if offset + bytes.len() as u32 > PAGE_SIZE {
            return Err(Error::MemoryInaccessible { page });
        }

        let page = self.mutate(page)?;

        // extend page if necessary
        let data_len = page.data.len() as u32;
        let to_write = bytes.len() as u32;
        if data_len < to_write + offset {
            page.data.resize(to_write as usize + offset as usize, 0);
        }

        // copy data
        page.data[offset as usize..(offset + to_write) as usize].copy_from_slice(bytes);

        Ok(())
    }

    /// Convert the memory to a data map.
    pub fn to_data_maps(&self) -> BTreeMap<u32, Vec<u8>> {
        let mut maps = BTreeMap::new();

        for (&page_num, page) in &self.pages {
            if page.data.is_empty() {
                continue;
            }

            let base = page_num * PAGE_SIZE;
            let mut current = None;
            let mut data = Vec::new();

            // Scan through each byte in the page
            for (offset, &byte) in page.data.iter().enumerate() {
                if byte == 0 {
                    if !data.is_empty() {
                        maps.insert(current.unwrap(), data);
                        data = Vec::new();
                        current = None;
                    }
                } else {
                    if current.is_none() {
                        current = Some(base + offset as u32);
                    }
                    data.push(byte);
                }
            }

            // Store any remaining data at the end of the page
            if !data.is_empty() {
                if let Some(addr) = current {
                    maps.insert(addr, data);
                }
            }
        }

        maps
    }

    /// Get the access type of a memory slot.
    fn access(&self, page: u32) -> Result<&Page> {
        self.pages
            .get(&page)
            .ok_or(Error::MemoryInaccessible { page })
    }

    /// Get the access type of a page.
    fn mutate(&mut self, pagenum: u32) -> Result<&mut Page> {
        let page = self
            .pages
            .get_mut(&pagenum)
            .ok_or(Error::MemoryInaccessible { page: pagenum })?;
        if page.is_immutable() {
            return Err(Error::MemoryImmutable { page: pagenum });
        }

        Ok(page)
    }

    /// Allocate a memory page if it doesn't exist
    pub fn allocate_page(&mut self, page_num: u32) -> Result<()> {
        self.pages.entry(page_num).or_insert(Page {
            data: SmallVec::new(),
            access: Access::Mutable,
        });

        Ok(())
    }

    /// Initialize heap pointer based on memory layout
    pub fn init_heap_pointer(&mut self, ro_len: u32, rw_len: u32) {
        // Following the Go implementation from README:
        // rw_data_address = 2 * Z_Z
        // rw_data_address_end = rw_data_address + Z_func(ro_len)
        // current_heap_pointer = rw_data_address_end + Z_P (extra Z_P is debatable)

        const Z_Z: u32 = 0x10000; // 2^16
        const Z_P: u32 = 0x1000; // PAGE_SIZE

        let rw_data_address = 2 * Z_Z;
        let z_func_ro_len = ((ro_len + Z_Z - 1) / Z_Z) * Z_Z; // Quantized RO data size
        let rw_data_address_end = rw_data_address + z_func_ro_len;

        // Heap starts after RW data section with page alignment
        self.current_heap_pointer = rw_data_address_end + rw_len + Z_P;

        tracing::debug!(
            "heap initialized: ro_len={}, rw_len={}, rw_data_end=0x{:x}, heap_start=0x{:x}",
            ro_len,
            rw_len,
            rw_data_address_end,
            self.current_heap_pointer
        );
    }

    /// Allocate pages for heap expansion (following Go implementation)
    pub fn allocate_heap_pages(&mut self, start_page: u32, page_count: u32) -> Result<()> {
        for i in 0..page_count {
            let page_num = start_page + i;
            self.pages.entry(page_num).or_insert(Page {
                data: SmallVec::new(),
                access: Access::Mutable,
            });
        }
        Ok(())
    }

    /// Get current heap pointer
    pub fn get_heap_pointer(&self) -> u32 {
        self.current_heap_pointer
    }

    /// Advance heap pointer and allocate pages if needed
    pub fn advance_heap(&mut self, bytes: u32) -> Result<u32> {
        let old_heap_pointer = self.current_heap_pointer;
        let new_heap_pointer = self.current_heap_pointer + bytes;

        // Check if we need to allocate new pages
        let old_page_boundary = ((old_heap_pointer + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;
        let new_page_boundary = ((new_heap_pointer + PAGE_SIZE - 1) / PAGE_SIZE) * PAGE_SIZE;

        if new_heap_pointer > old_page_boundary {
            let start_page = old_page_boundary / PAGE_SIZE;
            let end_page = new_page_boundary / PAGE_SIZE;
            let page_count = end_page - start_page;

            tracing::debug!(
                "allocating pages: start_page={}, page_count={}, old_boundary=0x{:x}, new_boundary=0x{:x}",
                start_page, page_count, old_page_boundary, new_page_boundary
            );

            self.allocate_heap_pages(start_page, page_count)?;
        }

        self.current_heap_pointer = new_heap_pointer;
        Ok(old_heap_pointer)
    }

    /// Allocate specific low memory pages (for service execution contexts)
    pub fn allocate_low_memory_pages(&mut self, max_page: u32) -> Result<()> {
        for page_num in 0..=max_page {
            if !self.pages.contains_key(&page_num) {
                tracing::debug!(
                    "allocating low memory page {} for service execution",
                    page_num
                );
                self.pages.entry(page_num).or_insert(Page {
                    data: SmallVec::new(),
                    access: Access::Mutable,
                });
            }
        }
        Ok(())
    }
}

impl pvm::Memory for Memory {
    fn contains(&self, data: &[u8]) -> bool {
        // Simple implementation: check if the data matches any page content
        self.pages
            .values()
            .any(|page| page.data.windows(data.len()).any(|window| window == data))
    }

    fn from_raw(memory: BTreeMap<u32, (Cow<'_, [u8]>, bool)>, initial_heap: u64) -> Self {
        let mut pages = BTreeMap::new();

        for (page_num, (data, writable)) in memory {
            pages.insert(
                page_num,
                Page {
                    data: SmallVec::from_slice(&data),
                    access: if writable {
                        Access::Mutable
                    } else {
                        Access::Immutable
                    },
                },
            );
        }

        Self {
            pages,
            current_heap_pointer: initial_heap as u32,
            initial_heap: initial_heap as u32,
        }
    }

    fn read_bytes(&self, address: u32, len: u32) -> std::result::Result<Vec<u8>, Reason> {
        let page = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE;

        // For read operations from the trait, we use the non-allocating version
        if let Some(page_data) = self.pages.get(&page) {
            let data = page_data.data.as_slice();
            let data_len = data.len() as u32;

            // fill with 0s if necessary
            let mut bytes = vec![0; len as usize];
            let to_copy = (len).min(data_len.saturating_sub(offset));
            if to_copy > 0 {
                bytes[..to_copy as usize]
                    .copy_from_slice(&data[offset as usize..(offset + to_copy) as usize]);
            }

            Ok(bytes)
        } else {
            tracing::error!("memory read: page {page} not found");
            Err(Reason::Fault { page })
        }
    }

    fn write_bytes(&mut self, from: u32, bytes: &[u8]) -> std::result::Result<(), Reason> {
        let page = from / PAGE_SIZE;
        let offset = from % PAGE_SIZE;

        // bounds check
        if offset + bytes.len() as u32 > PAGE_SIZE {
            tracing::error!("memory write: page {page} not found");
            return Err(Reason::Fault { page });
        }

        // For write operations from the trait, we allocate if needed
        self.allocate_page(page)?;

        if let Some(page_data) = self.pages.get_mut(&page) {
            // Check if page is writable
            if page_data.is_immutable() {
                tracing::error!("memory write: page {page} is immutable");
                return Err(Reason::Fault { page });
            }

            // Extend page data if necessary
            let required_size = offset + bytes.len() as u32;
            if page_data.data.len() < required_size as usize {
                page_data.data.resize(required_size as usize, 0);
            }

            // Copy data to page
            page_data.data[offset as usize..(offset + bytes.len() as u32) as usize]
                .copy_from_slice(bytes);

            Ok(())
        } else {
            tracing::error!("memory write: page {page} not found");
            Err(Reason::Fault { page })
        }
    }

    fn allocate_page(&mut self, page_num: u32) -> std::result::Result<(), Reason> {
        self.pages.entry(page_num).or_insert(Page {
            data: SmallVec::new(),
            access: Access::Mutable,
        });
        Ok(())
    }

    fn initial_heap(&self) -> u32 {
        self.initial_heap
    }

    fn get_heap_pointer(&self) -> Option<u32> {
        Some(self.current_heap_pointer)
    }

    fn set_heap_pointer(&mut self, heap_ptr: u32) {
        self.current_heap_pointer = heap_ptr;
    }
}

/// A memory page.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Page {
    /// The data of the page.
    pub data: SmallVec<[u8; PAGE_SIZE as usize]>,

    /// The access type of the page.
    pub access: Access,
}

impl Page {
    /// Whether the access is mutable.
    pub fn is_mutable(&self) -> bool {
        matches!(self.access, Access::Mutable)
    }

    /// Whether the access is immutable.
    pub fn is_immutable(&self) -> bool {
        matches!(self.access, Access::Immutable)
    }

    /// Whether the access is inaccessible.
    pub fn is_inaccessible(&self) -> bool {
        matches!(self.access, Access::Inaccessible)
    }
}

/// The access type of a memory page.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Access {
    /// The page is mutable.
    Mutable,
    /// The page is immutable.
    Immutable,
    /// The page is inaccessible.
    Inaccessible,
}
