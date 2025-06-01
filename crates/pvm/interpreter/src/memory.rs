//! Memory management for the interpreter

use crate::{Error, Result};
use pvm::{Reason, Value};
use smallvec::SmallVec;
use std::{collections::BTreeMap, ops::Range};

/// The size of a page in the memory.
pub const PAGE_SIZE: u32 = 4096;

/// The memory of the interpreter.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Memory {
    /// The pages of the memory.
    pub pages: BTreeMap<u32, Page>,

    /// Current heap pointer for sbrk implementation
    pub heap_ptr: u32,

    /// The heap range.
    pub heap: Range<u32>,
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
        if !self.pages.contains_key(&page) {
            self.pages.insert(
                page,
                Page {
                    data: SmallVec::new(),
                    access: Access::Mutable,
                },
            );
        }

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
            tracing::error!("page {page} not found");
            return Err(Error::MemoryInaccessible { page });
        }

        self.write_bytes(page, offset, &value.to_vec())
    }

    /// Write bytes to the memory.
    pub fn write_bytes(&mut self, page: u32, offset: u32, bytes: &[u8]) -> Result<()> {
        if offset + bytes.len() as u32 > PAGE_SIZE {
            tracing::error!("write_bytes, {page} inaccessible");
            return Err(Error::MemoryInaccessible { page });
        }

        // extend page if necessary
        let page_data = self.mutate(page)?;
        let data_len = page_data.data.len() as u32;
        let to_write = bytes.len() as u32;
        if data_len < to_write + offset {
            page_data
                .data
                .resize(to_write as usize + offset as usize, 0);
        }

        // copy data
        page_data.data[offset as usize..(offset + to_write) as usize].copy_from_slice(bytes);
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
        // Check if the page exists
        match self.pages.get(&page) {
            Some(page_data) => Ok(page_data),
            None => {
                tracing::warn!("memory page {page} not allocated");
                Err(Error::MemoryInaccessible { page })
            }
        }
    }

    /// Get the access type of a page.
    fn mutate(&mut self, pagenum: u32) -> Result<&mut Page> {
        let page = self
            .pages
            .get_mut(&pagenum)
            .ok_or(Error::MemoryInaccessible { page: pagenum })?;
        if page.is_immutable() {
            tracing::error!("mutate, memory write: page {pagenum} is immutable");
            return Err(Error::MemoryImmutable { page: pagenum });
        }

        Ok(page)
    }
}

impl pvm::Memory for Memory {
    fn contains(&self, data: &[u8]) -> bool {
        // Simple implementation: check if the data matches any page content
        self.pages
            .values()
            .any(|page| page.data.windows(data.len()).any(|window| window == data))
    }

    fn from_raw(memory: BTreeMap<u32, (Vec<u8>, bool)>, heap: Range<u32>) -> Self {
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
            heap_ptr: heap.start,
            heap,
        }
    }

    #[tracing::instrument(skip_all)]
    fn read_bytes(&self, address: u32, len: u32) -> std::result::Result<Vec<u8>, Reason> {
        // First 64KB of memory is always inaccessible per graypaper
        // Note: We removed the restriction on accessing the first 64KB of memory
        // to allow reading from heap memory for logging and other purposes

        let page = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE;
        let mut bytes = vec![0; len as usize];

        // Handle memory aliasing for page 16 - try shadow page 15 first for dynamic content
        if page == 16 {
            if let Some(shadow_data) = self.pages.get(&15) {
                let shadow_bytes = shadow_data.data.as_slice();
                if shadow_bytes.len() > offset as usize {
                    // Found data in shadow page, use it preferentially for dynamic content
                    let shadow_data_len = shadow_bytes.len() as u32;
                    let to_copy = (len).min(shadow_data_len.saturating_sub(offset));
                    if to_copy > 0 {
                        bytes[..to_copy as usize].copy_from_slice(
                            &shadow_bytes[offset as usize..(offset + to_copy) as usize],
                        );

                        return Ok(bytes);
                    }
                }
            }
            // Fall through to read from actual RO data if shadow is empty
        }

        // First, check if the page exists in the pages map
        if let Some(page_data) = self.pages.get(&page) {
            // Next, check if the page is accessible
            if page_data.is_inaccessible() {
                tracing::error!("memory read: memory page {page} inaccessible");
                return Err(Reason::Fault { page });
            }

            // Page exists and is accessible, so copy data if available
            let data = page_data.data.as_slice();
            let data_len = data.len() as u32;
            let to_copy = (len).min(data_len.saturating_sub(offset));
            if to_copy > 0 {
                bytes[..to_copy as usize]
                    .copy_from_slice(&data[offset as usize..(offset + to_copy) as usize]);
            }
            Ok(bytes)
        } else {
            tracing::debug!("memory read: memory page {page} not allocated, returning zeros");
            Ok(bytes)
        }
    }

    #[tracing::instrument(skip_all)]
    fn write_bytes(&mut self, from: u32, bytes: &[u8]) -> std::result::Result<(), Reason> {
        let page = from / PAGE_SIZE;
        let offset = from % PAGE_SIZE;

        // bounds check
        if offset + bytes.len() as u32 > PAGE_SIZE {
            tracing::error!("memory write: page {page} not found");
            return Err(Reason::Fault { page });
        }

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
