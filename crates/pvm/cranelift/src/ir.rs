//! the long waited IR

use cranelift::prelude::{types, AbiParam, Signature};
use cranelift_codegen::isa::CallConv;
use parser::{reader::Offset, Instruction, ProgramBlob};
use std::{collections::BTreeMap, ops::Range};

/// Polkadot Virtual Machine IR
pub struct IR {
    /// Dispatcher function
    pub main: Function,

    /// Functions in this program
    pub functions: BTreeMap<u64, Function>,
}

impl Default for IR {
    fn default() -> Self {
        Self {
            main: Function::main(),
            functions: BTreeMap::new(),
        }
    }
}

impl From<&ProgramBlob<'_>> for IR {
    fn from(program: &ProgramBlob<'_>) -> Self {
        let mut ir = Self::default();
        let mut reader = program.reader();

        // read main function
        while !reader.eof() {
            if let Ok(block) = reader.read_block() {
                ir.main.blocks.push(block);
                if program.jump_table.contains(&(reader.position as u64)) {
                    ir.main.offset.end = reader.position as u64;
                    break;
                }
            }
        }

        // read other functions
        for entry in &program.jump_table {
            let mut function = Function::new(*entry);
            reader.set_position(*entry as usize);
            while !reader.eof() {
                let pc = reader.position;
                if pc != *entry as usize && program.jump_table.contains(&(reader.position as u64)) {
                    function.offset.end = reader.position as u64;
                    break;
                }

                let Ok(block) = reader.read_block() else {
                    break;
                };

                function.blocks.push(block);
                function.offset.end = reader.position as u64;
            }

            ir.functions.insert(*entry, function);
        }

        ir
    }
}

/// Polkadot Virtual Machine Function
pub struct Function {
    /// Range of this function, e.g. start pc and end pc
    pub offset: Range<u64>,

    /// Blocks in this function
    pub blocks: Vec<Vec<Offset<Instruction>>>,

    /// Signature of this function
    pub signature: Signature,
}

impl Function {
    /// Create a new function
    pub fn new(pc: u64) -> Self {
        let signature = Signature {
            params: vec![AbiParam::new(types::I64)],
            returns: vec![],
            call_conv: CallConv::SystemV,
        };

        Self {
            offset: pc..pc,
            blocks: vec![],
            signature,
        }
    }

    /// Create a main function
    pub fn main() -> Self {
        let mut fun = Self::new(0);
        fun.signature.returns = vec![AbiParam::new(types::I64)];
        fun
    }
}
