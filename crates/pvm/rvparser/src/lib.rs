//! RISC-V machine code parser

pub mod format;
pub mod instr;
pub mod parser;
pub mod visitor;

pub use parser::parse;
