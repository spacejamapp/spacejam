//! Build context for the PVM parser.

use syn::{Expr, Ident};

/// The build context for the PVM parser.
pub struct Context {
    /// The name of the instruction.
    instr: Ident,
    /// The name of the instruction function.
    instr_fn: Ident,
    /// The opcode of the instruction.
    opcode: Expr,
}
