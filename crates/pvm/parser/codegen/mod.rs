//! Code generation for the instruction tables.

mod context;
mod format;
mod opcode;

use anyhow::Result;
pub use format::Format;
use opcode::OpcodeEnum;
use std::{collections::HashMap, env, fs, path::PathBuf};
use syn::Ident;

const VISITOR_RS: &str = "visitor.rs";
const INSTRUCTION_RS: &str = "instruction.rs";
const OPCODE_RS: &str = "opcode.rs";
const FORMAT_RS: &str = "format.rs";

/// The codegen for the PVM parser.
pub struct Codegen {
    /// The root directory of the PVM parser.
    pub root: PathBuf,

    /// The output directory for the generated code.
    pub out_dir: PathBuf,

    /// The opcode enum.
    pub opcode: OpcodeEnum,
}

impl Codegen {
    /// Creates a new codegen instance.
    pub fn new() -> Result<Self> {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let out_dir = PathBuf::from(env::var("OUT_DIR")?);

        Ok(Self {
            root,
            out_dir,
            opcode: OpcodeEnum::default(),
        })
    }

    /// Processes the codegen.
    pub fn process(mut self, formats: HashMap<Ident, Format>) -> Result<()> {
        for (_, format) in formats.into_iter() {
            for opcode in format.opcodes.iter() {
                self.opcode.emit(opcode);
            }
        }

        fs::write(self.out_dir.join(OPCODE_RS), self.opcode.to_string()).map_err(Into::into)
    }
}
