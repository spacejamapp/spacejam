//! Memory abstraction

use std::collections::BTreeMap;

use crate::Reason;

/// The memory trait.
pub trait Memory: Default + Clone {
    /// Create a new mwemory from a raw memory.
    fn from_raw(memory: BTreeMap<u32, (Vec<u8>, bool)>) -> Self;

    /// Check if the memory contains the given data.
    fn contains(&self, data: &[u8]) -> bool;

    /// read bytes from the memory
    fn read_bytes(&self, _page: u32, _offset: u32, _len: u32) -> Result<Vec<u8>, Reason>;

    /// write bytes to the memory
    fn write_bytes(&mut self, _page: u32, _offset: u32, _bytes: &[u8]) -> Result<(), Reason>;
}

impl Memory for () {
    fn from_raw(_memory: BTreeMap<u32, (Vec<u8>, bool)>) -> Self {}

    fn contains(&self, _data: &[u8]) -> bool {
        false
    }

    fn read_bytes(&self, _page: u32, _offset: u32, _len: u32) -> Result<Vec<u8>, Reason> {
        Err(Reason::Panic("read memory not supported".into()))
    }

    fn write_bytes(&mut self, _page: u32, _offset: u32, _bytes: &[u8]) -> Result<(), Reason> {
        Err(Reason::Panic("read memory not supported".into()))
    }
}
