//! Memory abstraction

use crate::Reason;

/// The memory trait.
pub trait Memory: Default + Clone {
    /// Create a new mwemory from a raw memory.
    fn from_raw(memory: parser::Memory) -> Self;

    /// Check if the memory contains the given data.
    fn contains(&self, data: &[u8]) -> bool;

    /// read bytes from the memory
    fn read_bytes(&self, ptr: u32, len: u32) -> Result<Vec<u8>, Reason>;

    /// read a hash from the memory
    fn read_hash(&self, from: u32) -> Result<[u8; 32], Reason> {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&self.read_bytes(from, 32)?);
        Ok(hash)
    }

    /// write bytes to the memory
    fn write_bytes(&mut self, from: u32, bytes: &[u8]) -> Result<(), Reason>;
}

impl Memory for () {
    fn from_raw(_memory: parser::Memory) -> Self {}

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
