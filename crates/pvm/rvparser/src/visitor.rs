//! RISC-V instruction visitor

use crate::{
    format::{BType, IType, JType, RType, SType, UType},
    instr::Instruction,
};

include!(concat!(env!("OUT_DIR"), "/visitor.rs"));
