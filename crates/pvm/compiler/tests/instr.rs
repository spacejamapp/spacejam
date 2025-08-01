//! PVM Compiler instruction tests
//!
//! Tests the PVM compiler (JIT) against the official JAM test vectors.

use anyhow::Result;
use pvmc::JitCompiler;
use serde::{Deserialize, Serialize};
use specjam::Test;

/// Test runner for PVM compiler tests
pub struct Runner;

impl Runner {
    /// Step a compiler test against the test vector
    pub fn step(test: &Test) -> Result<()> {
        let input: TestInput = serde_json::from_str(&test.input)?;
        let output: TestOutput = serde_json::from_str(&test.output)?;

        let mut compiler = JitCompiler::new()?;
        let mut initial_registers = [0u64; 13];
        initial_registers.copy_from_slice(&input.initial_regs);

        let module = compiler.compile(&input.program)?;
        let result = module.execute(&initial_registers, input.initial_pc as u64)?;
        assert_eq!(result.registers.len(), 13);
        assert_eq!(result.registers.to_vec(), output.expected_regs);
        assert_eq!(result.pc, output.expected_pc as u64);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Memory {
    /// The address of the memory slot.
    pub address: u32,
    /// The contents of the memory slot.
    pub contents: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestInput {
    pub name: String,
    #[serde(alias = "initial-regs")]
    pub initial_regs: Vec<u64>,
    #[serde(alias = "initial-pc")]
    pub initial_pc: u32,
    #[serde(alias = "initial-memory")]
    pub initial_memory: Vec<Memory>,
    #[serde(alias = "initial-gas")]
    pub initial_gas: u64,
    #[serde(alias = "initial-page-map")]
    pub initial_page_map: Vec<Page>,
    /// The program to run.
    pub program: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestOutput {
    #[serde(alias = "expected-pc")]
    pub expected_pc: usize,
    #[serde(alias = "expected-status")]
    pub expected_status: String,
    #[serde(alias = "expected-regs")]
    pub expected_regs: Vec<u64>,
    #[serde(alias = "expected-memory")]
    pub expected_memory: Vec<Memory>,
    #[serde(alias = "expected-gas")]
    pub expected_gas: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    /// The address of the page.
    pub address: u32,
    /// The length of the page.
    pub length: u16,
    /// Whether the page is writable.
    #[serde(alias = "is-writable")]
    pub is_writable: bool,
}

// Include the generated tests
include!(concat!(env!("OUT_DIR"), "/pvm_compiler_tests.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runner_basic() {
        // Test that the runner can be instantiated
        let _runner = Runner;
    }

    #[test]
    fn test_compiler_creation() {
        // Test that we can create a JIT compiler
        let compiler = JitCompiler::new();
        assert!(compiler.is_ok());
    }
}
