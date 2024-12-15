//! RISC-V instruction format

mod b;
mod i;
mod j;
mod r;
mod s;
mod u;

pub use b::BType;
pub use i::IType;
pub use j::JType;
pub use r::RType;
pub use s::SType;
pub use u::UType;

/// RISC-V instruction format
pub trait Format {
    const OPCODE: u8;
}

/// Extract bits from a u32 value
#[inline]
fn extract_bits(value: u32, msb: u32, lsb: u32) -> u32 {
    let mask = (1 << (msb - lsb + 1)) - 1;
    (value >> lsb) & mask
}
