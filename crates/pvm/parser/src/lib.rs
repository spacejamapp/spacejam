//! The PVM parser.

pub use {
    instruction::Instruction,
    memory::Memory,
    opcode::Opcode,
    program::{deblob, preimage, standard, PreimageBlob, ProgramBlob, StandardProgramBlob},
    reader::Reader,
    visitor::Visitor,
};

pub mod format;
pub mod instruction;
mod memory;
pub mod opcode;
pub mod program;
pub mod reader;
pub mod util;
pub mod visitor;

// Note the constants below are same from the score library.

/// (Z_P) The size of a page in octets (2^12)
pub const PAGE_SIZE: u64 = 1 << 12;

/// (Z_Z) The size of a zone in octets (2^16)
pub const ZONE_SIZE: u64 = 1 << 16;

/// (Z_I) The size of the init data in octets (2^24)
pub const PVM_INIT_DATA_SIZE: u64 = 1 << 24;

/// The size of the PVM memory
pub const PVM_MEMORY_SIZE: u64 = 1 << 32;

/// The size of the PVM zone
pub const PVM_ZONE_SIZE: u64 = 1 << 16;

/// The length of pages, p = 2^32 / 2^12
pub const PAGE_LENGTH: u64 = 1 << 20;

/// The type of the registers.
pub type Register = u64;
