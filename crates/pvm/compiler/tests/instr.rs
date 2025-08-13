//! PVM Compiler instruction tests
//!
//! Tests the PVM compiler (JIT) against the official JAM test vectors.

use anyhow::Result;
use pvmc::module::memory::Page as CompilerPage;
use pvmc::{Compiler, Memory as CompilerMemory};
use serde::{Deserialize, Serialize};
use specjam::Test;

/// Test runner for PVM compiler tests
pub struct Runner;

impl Runner {
    /// Step a compiler test against the test vector
    pub fn step(test: &Test) -> Result<()> {
        let input: TestInput = serde_json::from_str(&test.input)?;
        let output: TestOutput = serde_json::from_str(&test.output)?;

        let mut compiler = Compiler::new()?;
        let mut initial_registers = [0u64; 13];
        initial_registers.copy_from_slice(&input.initial_regs);

        // Initialize memory from test input
        let mut initial_memory = CompilerMemory::new();

        // First, allocate pages as mutable (to allow initial data writes)
        for page_info in &input.initial_page_map {
            let page_num = page_info.address / 4096; // PAGE_SIZE
            initial_memory.pages.insert(page_num, CompilerPage::new(0)); // 0=Mutable for initial setup
        }

        // Then write initial memory data
        for mem in &input.initial_memory {
            initial_memory.write_bytes(mem.address, &mem.contents)?;
        }

        // Finally, set correct page permissions
        for page_info in &input.initial_page_map {
            let page_num = page_info.address / 4096; // PAGE_SIZE
            let access = if page_info.is_writable { 0 } else { 1 }; // 0=Mutable, 1=Immutable
            if let Some(page) = initial_memory.pages.get_mut(&page_num) {
                page.access = access;
            }
        }

        let module = compiler.compile(&input.program)?;
        let result = module.execute(&initial_registers, input.initial_pc as u64, initial_memory)?;

        assert_eq!(result.registers.len(), 13);
        assert_eq!(result.registers.to_vec(), output.expected_regs);
        assert_eq!(result.pc, output.expected_pc as u64);

        // Validate memory state using helper function
        let final_memory_test = to_test_memory(&result.memory);
        assert_eq!(final_memory_test, output.expected_memory);
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

// Convert from compiler memory to test vector memory format
fn to_test_memory(compiler_memory: &CompilerMemory) -> Vec<Memory> {
    let mut result = Vec::new();

    for (&page_num, page) in &compiler_memory.pages {
        let base_address = page_num * 4096; // PAGE_SIZE
        let mut current_addr = None;
        let mut data = Vec::new();

        // Find non-zero data segments in the page
        for (offset, &byte) in page.data.iter().enumerate() {
            if byte == 0 {
                if !data.is_empty() {
                    if let Some(addr) = current_addr {
                        result.push(Memory {
                            address: addr,
                            contents: data,
                        });
                    }
                    data = Vec::new();
                    current_addr = None;
                }
            } else {
                if current_addr.is_none() {
                    current_addr = Some(base_address + offset as u32);
                }
                data.push(byte);
            }
        }

        // Handle remaining data at end of page
        if !data.is_empty() {
            if let Some(addr) = current_addr {
                result.push(Memory {
                    address: addr,
                    contents: data,
                });
            }
        }
    }

    result
}

// Include the generated tests
include!(concat!(env!("OUT_DIR"), "/pvm_compiler_tests.rs"));
