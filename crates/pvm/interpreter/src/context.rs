//! Context of the interpreter

use crate::Interpreter;
use pvm::{
    score::{service::ServiceAccount, Account, Gas},
    Argument,
};

/// Context of the interpreter
#[derive(Default)]
pub struct Context {
    /// The gas of the interpreter.
    pub gas: i64,

    /// The memory of the interpreter.
    pub memory: parser::Memory,

    /// The registers of the interpreter.
    pub registers: [u64; 13],
}

impl Context {
    /// Convert the context to the PVM context.
    pub fn ctx<'ctx, X: Argument>(
        &'ctx mut self,
        ctx: &'ctx mut X,
    ) -> pvm::Context<'ctx, X, &'ctx mut parser::Memory> {
        pvm::Context {
            registers: self.registers,
            gas: self.gas,
            dispatch: [0; pvm::MAX_FUNCTIONS],
            memory: &mut self.memory,
            ctx,
        }
    }
}

impl Argument for Interpreter {
    const SUPPORTED_CALLS: &[u32] = &[];
    const INITIAL_PC: u64 = 0;

    fn burn(&mut self, gas: Gas) {
        self.context.gas -= gas as i64;
    }

    fn read(&self, address: u32, len: u32) -> anyhow::Result<Vec<u8>> {
        self.context.memory.read_bytes(address, len)
    }

    fn write(&mut self, address: u32, data: &[u8]) -> anyhow::Result<()> {
        self.context.memory.write_bytes(address, data)
    }

    fn rget(&self, reg: u8) -> u64 {
        self.context.registers[reg as usize]
    }

    fn rset(&mut self, reg: u8, value: u64) {
        self.context.registers[reg as usize] = value;
    }

    fn heap_ptr(&self) -> u32 {
        self.context.memory.heap_ptr
    }

    fn set_heap_ptr(&mut self, heap_ptr: u32) {
        self.context.memory.heap_ptr = heap_ptr;
    }

    fn allocate(&mut self, start: u32, count: u32) -> anyhow::Result<()> {
        self.context.memory.allocate(start, count)
    }

    fn account(&mut self, _id: u64) -> anyhow::Result<&mut impl Account> {
        anyhow::Result::<&mut ServiceAccount>::Err(anyhow::anyhow!("not implemented"))
    }

    fn this(&mut self) -> anyhow::Result<&mut impl Account> {
        anyhow::Result::<&mut ServiceAccount>::Err(anyhow::anyhow!("not implemented"))
    }
}
