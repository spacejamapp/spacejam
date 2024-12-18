//! Code generation for the instruction tables.

mod format;
mod instruction;
mod opcode;
mod visitor;

use anyhow::Result;
pub use format::Format;
use heck::ToUpperCamelCase;
use instruction::InstructionEnum;
use proc_macro2::Span;
use std::{env, fs, path::PathBuf};
use syn::Ident;
use {format::Formats, opcode::OpcodeEnum, visitor::VisitorTrait};

const VISITOR_RS: &str = "visitor.rs";
const INSTRUCTION_RS: &str = "instruction.rs";
const OPCODE_RS: &str = "opcode.rs";
const FORMAT_RS: &str = "format.rs";

/// The codegen for the PVM parser.
pub struct Codegen {
    /// The output directory for the generated code.
    pub out_dir: PathBuf,

    /// The opcode enum.
    pub opcode: OpcodeEnum,

    /// The visitor trait.
    pub visitor: VisitorTrait,

    /// The instruction enum.
    pub instruction: InstructionEnum,

    /// The formats.
    pub formats: Formats,
}

impl Codegen {
    /// Processes the codegen.
    pub fn process(mut self, formats: Vec<Format>) -> Result<()> {
        for format in formats.into_iter() {
            let name = format.ident.clone();
            self.formats.emit(&name, &format);

            for opcode in format.opcodes.iter() {
                let opcodei = Ident::new(&opcode.name.to_upper_camel_case(), Span::call_site());
                self.instruction.emit(&format, &opcodei);
                self.opcode.emit(opcode, &opcodei, &format.ident);
                self.visitor.emit(&name, opcode, &opcodei);
            }
        }

        // write the files
        fs::write(self.out_dir.join(OPCODE_RS), self.opcode.to_string())?;
        fs::write(self.out_dir.join(VISITOR_RS), self.visitor.to_string())?;
        fs::write(self.out_dir.join(FORMAT_RS), self.formats.to_string())?;
        fs::write(
            self.out_dir.join(INSTRUCTION_RS),
            self.instruction.to_string(),
        )?;
        Ok(())
    }
}

impl Default for Codegen {
    fn default() -> Self {
        Self {
            out_dir: PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set")),
            opcode: OpcodeEnum::default(),
            visitor: VisitorTrait::default(),
            instruction: InstructionEnum::default(),
            formats: Formats::default(),
        }
    }
}
