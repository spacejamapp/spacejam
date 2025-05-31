//! Memory abstraction

use crate::Reason;
use std::{borrow::Cow, collections::BTreeMap};

/// The memory trait.
pub trait Memory: Default + Clone {
    /// Create a new mwemory from a raw memory.
    fn from_raw(memory: BTreeMap<u32, (Cow<'_, [u8]>, bool)>, initial_heap: u64) -> Self;

    /// Check if the memory contains the given data.
    fn contains(&self, data: &[u8]) -> bool;

    /// read bytes from the memory
    fn read_bytes(&self, _from: u32, _len: u32) -> Result<Vec<u8>, Reason>;

    /// read a hash from the memory
    fn read_hash(&self, from: u32) -> Result<[u8; 32], Reason> {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&self.read_bytes(from, 32)?);
        Ok(hash)
    }

    /// write bytes to the memory
    fn write_bytes(&mut self, _from: u32, _bytes: &[u8]) -> Result<(), Reason>;

    /// Allocate a memory page if it doesn't exist
    fn allocate_page(&mut self, _page_num: u32) -> Result<(), Reason> {
        Err(Reason::Panic("page allocation not supported".into()))
    }

    /// Get the initial heap pointer
    fn initial_heap(&self) -> u32 {
        0
    }

    /// Get current heap pointer (for sbrk implementation)
    fn get_heap_pointer(&self) -> Option<u32> {
        None
    }

    /// Set heap pointer (for sbrk implementation)
    fn set_heap_pointer(&mut self, _heap_ptr: u32) {}
}

impl Memory for () {
    fn from_raw(_memory: BTreeMap<u32, (Cow<'_, [u8]>, bool)>, _initial_heap: u64) -> Self {}

    fn contains(&self, _data: &[u8]) -> bool {
        false
    }

    fn read_bytes(&self, _from: u32, _len: u32) -> Result<Vec<u8>, Reason> {
        Err(Reason::Panic("read memory not supported".into()))
    }

    fn write_bytes(&mut self, _from: u32, _bytes: &[u8]) -> Result<(), Reason> {
        Err(Reason::Panic("read memory not supported".into()))
    }
}
