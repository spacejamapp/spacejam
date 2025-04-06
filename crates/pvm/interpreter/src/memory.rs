//! Memory management for the interpreter

use crate::{Error, Result};
use pvm::{Reason, Value};
use smallvec::SmallVec;
use std::collections::BTreeMap;

/// The size of a page in the memory.
pub const PAGE_SIZE: u32 = 4096;

/// The memory of the interpreter.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Memory {
    /// The pages of the memory.
    pub pages: BTreeMap<u32, Page>,
}

impl Memory {
    /// Read a value from the memory.
    pub fn read<V: Value>(&self, address: u32) -> Result<V> {
        self.read_offset(address, 0)
    }

    /// Read a value from the memory at an offset.
    pub fn read_offset<V: Value>(&self, address: u32, offset: u32) -> Result<V> {
        let start = address + offset;
        let page = start / PAGE_SIZE;
        let offset = start % PAGE_SIZE;

        // read bytes
        let bytes = self.read_bytes(page, offset, V::SIZE as u32)?;
        V::from_bytes(&bytes).ok_or(Error::MemoryInaccessible(page))
    }

    /// Read bytes from the memory.
    pub fn read_bytes(&self, page: u32, offset: u32, len: u32) -> Result<Vec<u8>> {
        if offset + len > PAGE_SIZE {
            return Err(Error::MemoryInaccessible(page));
        }

        let page = self.access(page)?;
        let data = page.data.as_slice();
        let data_len = data.len() as u32;

        // fill with 0s if necessary
        let mut bytes = vec![0; len as usize];
        let to_copy = (len).min(data_len.saturating_sub(offset));
        bytes[..to_copy as usize]
            .copy_from_slice(&data[offset as usize..(offset + to_copy) as usize]);
        Ok(bytes)
    }

    /// Write a value to the memory.
    pub fn write<V: Value>(&mut self, address: u32, value: V) -> Result<()> {
        self.write_bytes(address / PAGE_SIZE, address % PAGE_SIZE, &value.to_vec())
    }

    /// Write a value to the memory at an offset.
    pub fn write_offset<V: Value>(&mut self, address: u32, offset: u32, value: V) -> Result<()> {
        let start = address + offset;
        let page = start / PAGE_SIZE;
        let offset = start % PAGE_SIZE;
        if offset + V::SIZE as u32 > PAGE_SIZE {
            return Err(Error::MemoryInaccessible(page));
        }

        self.write_bytes(page, offset, &value.to_vec())
    }

    /// Write bytes to the memory.
    pub fn write_bytes(&mut self, page: u32, offset: u32, bytes: &[u8]) -> Result<()> {
        let page = self.mutate(page)?;

        // extend page if necessary
        let data_len = page.data.len() as u32;
        let to_write = bytes.len() as u32;
        if data_len < to_write + offset {
            page.data.resize(to_write as usize + offset as usize, 0);
        }

        // copy data
        page.data[offset as usize..(offset + to_write) as usize]
            .copy_from_slice(&bytes[..to_write as usize]);
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
        self.pages.get(&page).ok_or(Error::MemoryInaccessible(page))
    }

    /// Get the access type of a page.
    fn mutate(&mut self, pagenum: u32) -> Result<&mut Page> {
        let page = self
            .pages
            .get_mut(&pagenum)
            .ok_or(Error::MemoryInaccessible(pagenum))?;
        if page.is_immutable() {
            return Err(Error::MemoryImmutable(pagenum));
        }

        Ok(page)
    }
}

impl pvm::Memory for Memory {
    // TODO: optimize this without using windows
    fn contains(&self, data: &[u8]) -> bool {
        let len = data.len();
        self.pages
            .values()
            .any(|page| page.data.windows(len).any(|window| window == data))
    }

    fn from_raw(memory: BTreeMap<u32, (Vec<u8>, bool)>) -> Self {
        let mut pages = BTreeMap::new();
        for (addr, (data, is_immutable)) in memory {
            pages.insert(
                addr,
                Page {
                    data: data.into(),
                    access: if is_immutable {
                        Access::Immutable
                    } else {
                        Access::Mutable
                    },
                },
            );
        }

        Self { pages }
    }

    fn read_bytes(&self, page: u32, offset: u32, len: u32) -> std::result::Result<Vec<u8>, Reason> {
        self.read_bytes(page, offset, len).map_err(Into::into)
    }

    fn write_bytes(
        &mut self,
        page: u32,
        offset: u32,
        bytes: &[u8],
    ) -> std::result::Result<(), Reason> {
        self.write_bytes(page, offset, bytes).map_err(Into::into)
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
