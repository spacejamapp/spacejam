//! PVM interface implementation

use crate::Interpreter;
use parser::{program, reader::Offset, Instruction, Memory};
use pvm::{
    score::{Gas, OpaqueHash},
    Argument, Invocation, Invoked,
};
use std::{
    collections::BTreeMap,
    sync::{LazyLock, RwLock},
};

/// The cached programs.
pub static CACHED_PROGRAMS: LazyLock<RwLock<BTreeMap<OpaqueHash, ParsedProgram>>> =
    LazyLock::new(|| RwLock::new(BTreeMap::new()));

/// Set the parsed program.
pub fn set(hash: OpaqueHash, program: ParsedProgram) {
    if let Ok(mut cached_programs) = CACHED_PROGRAMS.try_write() {
        cached_programs.insert(hash, program);
        if cached_programs.len() > 20 {
            cached_programs.pop_first();
        }
    }
}

/// Get the parsed program.
pub fn get(hash: OpaqueHash) -> Option<ParsedProgram> {
    if let Ok(cached_programs) = CACHED_PROGRAMS.try_read() {
        return cached_programs.get(&hash).cloned();
    }
    None
}

/// The parsed program.
#[derive(Clone)]
pub struct ParsedProgram {
    /// The parsed program.
    pub program: Vec<Option<Offset<Instruction>>>,
    /// The registers of the program.
    pub registers: [u64; 13],
    /// The memory of the program.
    pub memory: Memory,
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
        let is_polkavm = match std::env::var("POLKAVM") {
            Ok(v) => v.to_lowercase() == "true",
            Err(_) => false,
        };

        let now = std::time::Instant::now();
        if is_polkavm {
            let res = crate::polkavmi::invoke(ctx, &code, &args, gas, pc).expect("fix me later");
            println!(
                "Polka VM TIME: {:?}, gas: {}, output: {}",
                now.elapsed(),
                res.gas,
                res.output.len()
            );
            res
        } else {
            let program = program::preimage(code, &args).expect("failed to preimage");
            if let Some(parsed) = self::get(hash) {
                return Self::invoke_parsed(parsed, ctx, gas, pc).expect("fix me later");
            }

            let res = Self::invoke(&program, hash, ctx, gas, pc).expect("fix me later");
            println!(
                "Space VM TIME: {:?}, gas: {}, output: {}",
                now.elapsed(),
                res.gas,
                res.output.len()
            );
            res
        }
    }
}
