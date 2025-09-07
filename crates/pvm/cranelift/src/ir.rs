//! the long waited IR

use cranelift::prelude::{types, AbiParam, Signature};
use cranelift_codegen::isa::CallConv;
use parser::{reader::Offset, Instruction, ProgramBlob};
use std::{collections::BTreeMap, ops::Range};

/// Signature for the function
pub fn sig() -> Signature {
    Signature {
        params: vec![AbiParam::new(types::I64); 16],
        returns: vec![AbiParam::new(types::I64); 2],
        call_conv: CallConv::Fast,
    }
}

/// Polkadot Virtual Machine IR
pub struct IR {
    /// Dispatcher function
    pub main: Function,

    /// Functions in this program
    pub dfuncs: BTreeMap<u64, Function>,
}

impl Default for IR {
    fn default() -> Self {
        Self {
            main: Function::new(0),
            dfuncs: BTreeMap::new(),
        }
    }
}

impl From<&ProgramBlob<'_>> for IR {
    fn from(program: &ProgramBlob<'_>) -> Self {
        let mut ir = Self::default();
        let mut reader = program.reader();

        // read main function
        let mut target = 0;
        while !reader.eof() {
            if let Ok(block) = reader.read_block() {
                ir.main.blocks.insert(target, block);
                target = reader.position as u64;
            }
        }

        ir
    }
}

/// Polkadot Virtual Machine Function
pub struct Function {
    /// Range of this function, e.g. start pc and end pc
    pub offset: Range<u64>,

    /// Blocks in this function
    pub blocks: BTreeMap<u64, Vec<Offset<Instruction>>>,

    /// Signature of this function
    pub signature: Signature,
}

impl Function {
    /// Create a new function
    pub fn new(pc: u64) -> Self {
        Self {
            offset: pc..pc,
            blocks: BTreeMap::new(),
            signature: sig(),
        }
    }
}
