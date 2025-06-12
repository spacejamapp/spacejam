//! Memory management for the interpreter

use crate::{Error, Result};
use pvm::{Reason, Value};
use smallvec::SmallVec;
use std::{collections::BTreeMap, ops::Range};

/// The memory of the interpreter.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Memory {
    /// The pages of the memory.
    pub pages: BTreeMap<u32, Page>,

    /// Current heap pointer for sbrk implementation
    pub heap_ptr: u32,

    /// The heap (read-write) range.
    pub heap: Range<u32>,

    /// The read-only range.
    pub read: Range<u32>,

    /// The stack range.
    pub stack: Range<u32>,

    /// The args range.
    pub args: Range<u32>,
}

impl Memory {
    /// Allocate a range of pages.
    ///
    /// TODO:
    /// - check the range of the heap
    /// - deallocate pages
    pub fn allocate_pages(&mut self, start: u32, count: u32) -> Result<()> {
        for page in start..start + count {
            self.pages.insert(
                page,
                Page {
                    data: SmallVec::from_slice(&vec![0; parser::PAGE_SIZE as usize]),
                    access: Access::Mutable,
                },
            );
        }

        self.heap.end = (start + count) * parser::PAGE_SIZE as u32;
        Ok(())
    }

    /// Read a value from the memory.
    pub fn read<V: Value>(&mut self, address: u32) -> Result<V> {
        self.read_offset(address, 0)
    }

    /// Read a value from the memory at an offset.
    pub fn read_offset<V: Value>(&mut self, address: u32, offset: u32) -> Result<V> {
        let start = address.wrapping_add(offset);
        let page = start / parser::PAGE_SIZE as u32;
        let bytes = self.read_bytes(page, start % parser::PAGE_SIZE as u32, V::SIZE as u32)?;
        V::from_bytes(&bytes).ok_or(Error::MemoryInaccessible { page })
    }

    /// Read bytes from the memory.
    pub fn read_bytes(&self, mut page: u32, mut offset: u32, len: u32) -> Result<Vec<u8>> {
        let mut bytes = vec![0; len as usize];
        let mut read = 0u32;
        while read < len {
            let to_read = (len - read).min(parser::PAGE_SIZE as u32 - offset);
            let data = self.access(page)?;
            if to_read > 0 {
                bytes[read as usize..(read + to_read) as usize]
                    .copy_from_slice(&data.data[offset as usize..(offset + to_read) as usize]);
            }

            read += to_read;
            page += 1;
            offset = 0;
        }

        Ok(bytes)
    }

    /// Write a value to the memory.
    pub fn write<V: Value>(&mut self, address: u32, value: V) -> Result<()> {
        self.write_bytes(
            address / parser::PAGE_SIZE as u32,
            address % parser::PAGE_SIZE as u32,
            &value.to_vec(),
        )
    }

    /// Write a value to the memory at an offset.
    pub fn write_offset<V: Value>(&mut self, address: u32, offset: u32, value: V) -> Result<()> {
        let start = address.wrapping_add(offset);
        let page = start / parser::PAGE_SIZE as u32;
        let offset = start % parser::PAGE_SIZE as u32;
        if offset + V::SIZE as u32 > parser::PAGE_SIZE as u32 {
            tracing::error!("page {page} not found");
            return Err(Error::MemoryInaccessible { page });
        }

        self.write_bytes(page, offset, &value.to_vec())
    }

    /// Write bytes to the memory.
    pub fn write_bytes(&mut self, mut page: u32, mut offset: u32, bytes: &[u8]) -> Result<()> {
        let len = bytes.len() as u32;
        let mut written = 0u32;
        while written < len {
            let to_write = (len - written).min(parser::PAGE_SIZE as u32 - offset);
            let data = self.mutate(page)?;
            data.data[offset as usize..(offset + to_write) as usize]
                .copy_from_slice(&bytes[written as usize..(written + to_write) as usize]);

            written += to_write;
            page += 1;
            offset = 0;
        }

        Ok(())
    }

    /// Convert the memory to a data map.
    pub fn to_data_maps(&self) -> BTreeMap<u32, Vec<u8>> {
        let mut maps = BTreeMap::new();

        for (&page_num, page) in &self.pages {
            if page.data.is_empty() {
                continue;
            }

            let base = page_num * parser::PAGE_SIZE as u32;
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

    fn from_raw(memory: parser::Memory) -> Self {
        let mut pages = BTreeMap::new();
        for (page_num, (data, writable)) in memory.memory {
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
            heap_ptr: memory.heap.end,
            heap: memory.heap,
            read: memory.read,
            stack: memory.stack,
            args: memory.args,
        }
    }

    #[tracing::instrument(skip_all)]
    fn read_bytes(&self, address: u32, len: u32) -> std::result::Result<Vec<u8>, Reason> {
        let page = address / parser::PAGE_SIZE as u32;
        let offset = address % parser::PAGE_SIZE as u32;
        self.read_bytes(page, offset, len).map_err(Reason::from)
    }

    #[tracing::instrument(skip_all)]
    fn write_bytes(&mut self, from: u32, bytes: &[u8]) -> std::result::Result<(), Reason> {
        let page = from / parser::PAGE_SIZE as u32;
        let offset = from % parser::PAGE_SIZE as u32;
        self.write_bytes(page, offset, bytes).map_err(Reason::from)
    }
}

/// A memory page.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Page {
    /// The data of the page.
    pub data: SmallVec<[u8; parser::PAGE_SIZE as usize]>,

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
