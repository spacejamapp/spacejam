//! PVM Compiler instruction tests
//!
//! Tests the PVM compiler (JIT) against the official JAM test vectors.

use anyhow::Result;
use pvmc::Compiler;
use serde::{Deserialize, Serialize};
use specjam::Test;
use tracing_subscriber::EnvFilter;

/// Test runner for PVM compiler tests
pub struct Runner;

impl Runner {
    /// Step a compiler test against the test vector
    pub fn step(test: &Test) -> Result<()> {
        let _ = tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .without_time()
            .with_ansi(false)
            .with_thread_names(false)
            .with_file(false)
            // .with_level(false)
            .with_target(false)
            .try_init();

        let input: TestInput = serde_json::from_str(&test.input)?;
        let output: TestOutput = serde_json::from_str(&test.output)?;
        let mut initial_registers = [0u64; translator::PVM_REGISTER_COUNT];
        initial_registers.copy_from_slice(&input.initial_regs);

        // Initialize memory from test input
        let mut initial_memory = pvm::Memory::default();

        // First, allocate pages with proper permissions
        for page_info in &input.initial_page_map {
            let page_num = page_info.address / pvm::PAGE_SIZE as u32;
            let page_data = vec![0u8; pvm::PAGE_SIZE as usize];
            // Initially set all pages as writable for data initialization
            initial_memory.memory.insert(page_num, (page_data, true));
        }

        // Then write initial memory data
        for mem in &input.initial_memory {
            initial_memory.write_bytes(mem.address, &mem.contents)?;
        }

        // Finally, set correct page permissions
        for page_info in &input.initial_page_map {
            let page_num = page_info.address / pvm::PAGE_SIZE as u32;
            if let Some((_page_data, writable)) = initial_memory.memory.get_mut(&page_num) {
                *writable = page_info.is_writable;
            }
        }

        let mut compiler = Compiler::new()?;
        let module = compiler.compile(&input.program)?;
        let result = module.execute(&initial_registers, input.initial_pc as u64, initial_memory)?;

        assert_eq!(result.registers.len(), translator::PVM_REGISTER_COUNT);
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
fn to_test_memory(memory: &pvm::Memory) -> Vec<Memory> {
    let mut result = Vec::new();

    for (&page_num, (page_data, _)) in &memory.memory {
        let base_address = page_num * pvm::PAGE_SIZE as u32;
        let mut current_addr = None;
        let mut data = Vec::new();

        // Find non-zero data segments in the page
        for (offset, &byte) in page_data.iter().enumerate() {
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
