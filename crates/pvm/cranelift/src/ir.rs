//! the long waited IR

use cranelift::prelude::{types, AbiParam, Signature};
use cranelift_codegen::{ir::ArgumentPurpose, isa::CallConv};
use parser::{reader::Offset, Instruction, ProgramBlob};
use std::{collections::BTreeMap, ops::Range};

/// Signature for the function
pub fn sig(main: bool) -> Signature {
    // [gas, registers]
    let returns = vec![AbiParam::new(types::I64); 2];
    if main {
        Signature {
            params: [
                vec![
                    AbiParam::special(types::I64, ArgumentPurpose::VMContext),
                    AbiParam::new(types::I8),
                    AbiParam::new(types::I64),
                ],
                vec![AbiParam::new(types::I64); 13],
            ]
            .concat(),
            returns,
            call_conv: CallConv::Fast,
        }
    } else {
        let mut sig = Signature {
            params: vec![AbiParam::new(types::I64); 15],
            returns,
            call_conv: CallConv::Fast,
        };
        sig.params[13] = AbiParam::special(types::I64, ArgumentPurpose::VMContext);
        sig
    }
}

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
            main: Function::new(0, true),
            functions: BTreeMap::new(),
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
                if program.jump_table.contains(&(reader.position as u64)) {
                    ir.main.offset.end = reader.position as u64;
                    break;
                }
            }
        }

        // read other functions
        for entry in &program.jump_table {
            let mut function = Function::new(*entry, false);
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

                function.blocks.insert(pc as u64, block);
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
    pub blocks: BTreeMap<u64, Vec<Offset<Instruction>>>,

    /// Signature of this function
    pub signature: Signature,
}

impl Function {
    /// Create a new function
    pub fn new(pc: u64, main: bool) -> Self {
        Self {
            offset: pc..pc,
            blocks: BTreeMap::new(),
            signature: sig(main),
        }
    }
}
