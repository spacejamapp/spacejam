//! PVM interface implementation

use crate::Interpreter;
use lru::LruCache;
use parser::{program, reader::Offset, Instruction};
use pvm::{
    score::{Gas, OpaqueHash},
    Argument, Invocation, Invoked, Pvm,
};
use std::{
    num::NonZeroUsize,
    sync::{Arc, LazyLock, Mutex},
};

/// The maximum number of cached parsed programs.
const MAX_CACHED_PROGRAMS: usize = 16;

/// Cached parsed programs (LRU).
pub static CACHED_PROGRAMS: LazyLock<Mutex<LruCache<OpaqueHash, Arc<ParsedProgram>>>> =
    LazyLock::new(|| {
        Mutex::new(LruCache::new(
            NonZeroUsize::new(MAX_CACHED_PROGRAMS).expect("MAX_CACHED_PROGRAMS must be non-zero"),
        ))
    });

/// Set the parsed program.
pub fn set(hash: OpaqueHash, program: ParsedProgram) {
    if let Ok(mut cache) = CACHED_PROGRAMS.try_lock() {
        cache.put(hash, Arc::new(program));
    }
}

/// Get the parsed program.
pub fn get(hash: OpaqueHash) -> Option<Arc<ParsedProgram>> {
    if let Ok(mut cache) = CACHED_PROGRAMS.try_lock() {
        return cache.get(&hash).cloned();
    }
    None
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
