//! Memory management for the interpreter

use crate::{Error, Result, Value};
use std::collections::BTreeMap;

/// The size of a page in the memory.
pub const PAGE_SIZE: u64 = 4096;

/// The memory of the interpreter.
#[derive(Default, Debug, PartialEq, Eq, Clone)]
pub struct Memory {
    /// The pages of the memory.
    pub pages: BTreeMap<u64, Page>,

    /// The slots of the memory.
    ///
    /// TODO: this should be a BTreeMap<u32, Vec<u8>>
    pub slots: BTreeMap<u64, Vec<u8>>,
}

impl Memory {
    /// Read a value from the memory.
    pub fn read<V: Value>(&self, address: u64) -> Result<V> {
        self.read_offset(address, 0)
    }

    /// Write a value to the memory.
    pub fn write<V: Value>(&mut self, address: u64, value: V) -> Result<()> {
        if !self.access(address, 0, V::SIZE)?.is_mutable() {
            return Err(Error::MemoryImmutable);
        }

        let bytes = value.to_vec();
        self.slots.insert(address, bytes);
        Ok(())
    }

    /// Read a value from the memory at an offset.
    pub fn read_offset<V: Value>(&self, address: u64, offset: u64) -> Result<V> {
        let bytes = self.read_bytes(address)?;
        let offset = offset as usize;
        V::from_bytes(&bytes[offset..offset + V::SIZE]).ok_or(Error::MemoryInaccessible)
    }

    /// Read bytes from the memory.
    pub fn read_bytes(&self, address: u64) -> Result<Vec<u8>> {
        self.access(address, 0, PAGE_SIZE as usize)?;

        Ok(self
            .slots
            .get(&address)
            .ok_or(Error::MemoryInaccessible)?
            .clone())
    }

    /// Get the access type of a memory slot.
    pub fn access(&self, address: u64, offset: u64, size: usize) -> Result<&Access> {
        let address = address + offset;
        let page = self
            .pages
            .get(&(address / PAGE_SIZE))
            .ok_or(Error::MemoryInaccessible)?;

        tracing::debug!("reading page {:?}", page);
        if size > PAGE_SIZE as usize {
            return Err(Error::MemoryInaccessible);
        }

        Ok(&page.access)
    }
}

/// A memory page.
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Page {
    /// The length of the page.
    pub length: u32,
    /// The access type of the page.
    pub access: Access,
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

impl Access {
    /// Whether the access is mutable.
    pub fn is_mutable(&self) -> bool {
        matches!(self, Access::Mutable)
    }

    /// Whether the access is immutable.
    pub fn is_immutable(&self) -> bool {
        matches!(self, Access::Immutable)
    }

    /// Whether the access is inaccessible.
    pub fn is_inaccessible(&self) -> bool {
        matches!(self, Access::Inaccessible)
    }
}
