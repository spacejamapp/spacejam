//! Memory interfaces for the PVM.

/// The memory of the PVM.
#[derive(Debug, Clone)]
pub struct Memory {
    /// The value of the memory
    pub value: [u8; score::PVM_MEMORY_SIZE],

    /// The access type of the memory
    pub access: [Option<Access>; score::PAGE_LENGTH],
}

impl Default for Memory {
    fn default() -> Self {
        Self {
            value: [0; score::PVM_MEMORY_SIZE],
            access: [None; score::PAGE_LENGTH],
        }
    }
}

/// The access type of the memory.
#[derive(Debug, Clone, Copy)]
pub enum Access {
    /// The memory is mutable.
    Mutable,

    /// The memory is immutable.
    Immutable,
}
