//! Memory management for the interpreter

use crate::{Error, Result, Value};
use smallvec::SmallVec;
use std::collections::BTreeMap;

/// The size of a page in the memory.
pub const PAGE_SIZE: u64 = 4096;

/// The memory of the interpreter.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Memory {
    /// The pages of the memory.
    pub pages: BTreeMap<u64, Page>,
}

impl Memory {
    /// Read a value from the memory.
    pub fn read<V: Value>(&self, address: u64) -> Result<V> {
        self.read_offset(address, 0)
    }

    /// Read a value from the memory at an offset.
    pub fn read_offset<V: Value>(&self, address: u64, offset: u64) -> Result<V> {
        let page = address / PAGE_SIZE;
        if offset + (address % PAGE_SIZE) + V::SIZE as u64 > PAGE_SIZE {
            return Err(Error::MemoryInaccessible(page as u32));
        }

        let bytes = self.read_bytes(address, offset, V::SIZE as u64)?;
        V::from_bytes(&bytes).ok_or(Error::MemoryInaccessible(page as u32))
    }

    /// Read bytes from the memory.
    pub fn read_bytes(&self, address: u64, offset: u64, len: u64) -> Result<Vec<u8>> {
        let pagenum = address / PAGE_SIZE;
        let offset = address % PAGE_SIZE + offset;
        let page = self.access(pagenum)?;
        let data = page.data.as_slice();
        let data_len = data.len() as u64;

        // fill with 0s if necessary
        let mut bytes = vec![0; len as usize];
        let to_copy = (len).min(data_len.saturating_sub(offset));
        bytes[..to_copy as usize]
            .copy_from_slice(&data[offset as usize..(offset + to_copy) as usize]);
        Ok(bytes)
    }

    /// Write a value to the memory.
    pub fn write<V: Value>(&mut self, address: u64, value: V) -> Result<()> {
        self.write_bytes(address, 0, &value.to_vec())
    }

    /// Write a value to the memory at an offset.
    pub fn write_offset<V: Value>(&mut self, address: u64, offset: u64, value: V) -> Result<()> {
        let page = address / PAGE_SIZE;
        if offset + (address % PAGE_SIZE) + V::SIZE as u64 > PAGE_SIZE {
            return Err(Error::MemoryInaccessible(page as u32));
        }

        // TODO: note that we hacked (u64).to_vec() here for matching the
        // pvm stf, there could be sth wrong in the test vectors.
        self.write_bytes(address, offset, &value.to_vec())
    }

    /// Write bytes to the memory.
    pub fn write_bytes(&mut self, address: u64, offset: u64, bytes: &[u8]) -> Result<()> {
        let offset = address % PAGE_SIZE + offset;
        let page = self.mutate(address / PAGE_SIZE)?;

        // extend page if necessary
        let data_len = page.data.len() as u64;
        let to_write = bytes.len() as u64;
        if data_len < to_write + offset {
            page.data.resize(to_write as usize + offset as usize, 0);
        }

        // copy data
        page.data[offset as usize..(offset + to_write) as usize]
            .copy_from_slice(&bytes[..to_write as usize]);
        Ok(())
    }

    /// Convert the memory to a data map.
    pub fn to_data_maps(&self) -> BTreeMap<u64, Vec<u8>> {
        self.pages
            .iter()
            .filter_map(|(k, v)| {
                if v.data.is_empty() {
                    return None;
                }

                let offset = v.data.iter().position(|b| *b != 0).unwrap_or_default();
                Some((k * PAGE_SIZE + offset as u64, v.data[offset..].to_vec()))
            })
            .collect()
    }

    /// Get the access type of a memory slot.
    fn access(&self, page: u64) -> Result<&Page> {
        self.pages
            .get(&page)
            .ok_or(Error::MemoryInaccessible(page as u32))
    }

    /// Get the access type of a page.
    fn mutate(&mut self, pagenum: u64) -> Result<&mut Page> {
        let page = self
            .pages
            .get_mut(&pagenum)
            .ok_or(Error::MemoryInaccessible(pagenum as u32))?;
        if page.is_immutable() {
            return Err(Error::MemoryImmutable(pagenum as u32));
        }

        Ok(page)
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
