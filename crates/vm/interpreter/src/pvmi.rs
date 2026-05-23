//! PVM interface implementation

use crate::Interpreter;
use parser::{program, reader::Offset, Instruction};
use pvm::{
    score::{Gas, OpaqueHash},
    Argument, Cache, Invocation, Invoked, Pvm,
};
use std::sync::{Arc, LazyLock};

/// Cached parsed programs.
pub static CACHED_PROGRAMS: LazyLock<Cache<ParsedProgram>> = LazyLock::new(Default::default);

/// Set the parsed program.
pub fn set(hash: OpaqueHash, program: ParsedProgram) {
    CACHED_PROGRAMS.put(hash, Arc::new(program));
}

/// Get the parsed program.
pub fn get(hash: OpaqueHash) -> Option<Arc<ParsedProgram>> {
    CACHED_PROGRAMS.get(&hash)
}

/// The parsed program.
#[derive(Clone)]
pub struct ParsedProgram {
    /// The parsed program.
    pub program: Vec<Option<Offset<Instruction>>>,
    /// The jump table of the program.
    pub table: Vec<u64>,
}

impl Invocation for Interpreter {
    fn invoke2<X: Argument>(
        ctx: X,
        hash: OpaqueHash,
        code: Vec<u8>,
        args: Vec<u8>,
        gas: Gas,
        pc: usize,
    ) -> Invoked<X> {
        let program = program::preimage(code, &args).expect("failed to preimage");
        if let Some(parsed) = self::get(hash) {
            return Self::invoke_parsed(&parsed, program, ctx, gas, pc).expect("fix me later");
        }

        Self::invoke(program, hash, ctx, gas, pc).expect("fix me later")
    }
}

impl Pvm for Interpreter {}
