//! PVM instruction visitor.

use crate::format::*;
use crate::Instruction;

pub mod polkavm;

include!(concat!(env!("OUT_DIR"), "/visitor.rs"));

/// get the register name.
pub fn fmt(reg: u8) -> &'static str {
    match reg {
        0 => "ra",
        1 => "sp",
        5 => "s0",
        6 => "s1",
        7 => "a0",
        8 => "a1",
        9 => "a2",
        10 => "a3",
        11 => "a4",
        _ => "unknown",
    }
}
