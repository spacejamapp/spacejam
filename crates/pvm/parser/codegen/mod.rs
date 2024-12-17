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
use quote::{quote, ToTokens};
use std::{env, fs, path::PathBuf};
use syn::{parse_quote, Ident, ItemStruct};
use {opcode::OpcodeEnum, visitor::VisitorTrait};

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
    pub formats: Vec<ItemStruct>,
}

impl Codegen {
    /// Processes the codegen.
    pub fn process(mut self, formats: Vec<Format>) -> Result<()> {
        for format in formats.into_iter() {
            let name = format.ident.clone();
            self.format(&name, &format);

            for opcode in format.opcodes.iter() {
                let opcodei = Ident::new(&opcode.name.to_upper_camel_case(), Span::call_site());
                self.instruction.emit(&format, &opcodei);
                self.opcode.emit(opcode, &opcodei);
                self.visitor.emit(&name, opcode);
            }
        }

        // write the files
        fs::write(self.out_dir.join(OPCODE_RS), self.opcode.to_string())?;
        fs::write(self.out_dir.join(VISITOR_RS), self.visitor.to_string())?;
        fs::write(self.out_dir.join(FORMAT_RS), self.formats())?;
        fs::write(
            self.out_dir.join(INSTRUCTION_RS),
            self.instruction.to_string(),
        )?;
        Ok(())
    }

    fn format(&mut self, name: &Option<Ident>, format: &Format) {
        let Some(name) = name else {
            return;
        };

        let description = &format.description;
        let item = parse_quote! {
            #[doc = #description]
            pub struct #name;
        };

        self.formats.push(item);
    }

    /// Formats the formats.
    fn formats(&self) -> String {
        let formats = self.formats.clone();

        quote! {
            #(#formats)*
        }
        .to_token_stream()
        .to_string()
    }
}

impl Default for Codegen {
    fn default() -> Self {
        Self {
            out_dir: PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR is not set")),
            opcode: OpcodeEnum::default(),
            visitor: VisitorTrait::default(),
            instruction: InstructionEnum::default(),
            formats: vec![],
        }
    }
}
