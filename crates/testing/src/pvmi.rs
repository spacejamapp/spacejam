//! PVM test vectors

use std::borrow::Cow;

use pvm::{Invocation, Program};
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/pvm.rs"));

/// Run the PVM test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let input: TestInput = serde_json::from_str(&test.input)?;
    let output: TestOutput = serde_json::from_str(&test.output)?;
    let mut registers = [0; 13];
    registers.copy_from_slice(&input.initial_regs);

    // Initialize memory using the new unified parser::Memory
    let mut memory = pvmi::Memory::default();
    for page in &input.initial_page_map {
        let page_num = page.address / ::pvmi::PAGE_SIZE as u32;
        memory
            .memory
            .insert(page_num, (vec![0u8; ::pvmi::PAGE_SIZE as usize], true));
    }

    // write initial memory data
    for mem in &input.initial_memory {
        let page_num = mem.address / ::pvmi::PAGE_SIZE as u32;
        if let Some((data, _)) = memory.memory.get(&page_num).cloned() {
            memory.memory.insert(page_num, (data, true));
        }
        memory.write_bytes(mem.address, &mem.contents)?;
    }

    // restore original page permissions
    for page in &input.initial_page_map {
        let page_num = page.address / ::pvmi::PAGE_SIZE as u32;
        if let Some((data, _)) = memory.memory.get(&page_num).cloned() {
            memory.memory.insert(page_num, (data, page.is_writable));
        }
    }

    // run the program
    let program = Program {
        code: input.program.to_vec(),
        memory: memory.clone(),
        registers,
    };
    let result = <pvmi::Interpreter as Invocation>::invoke2(
        &program,
        (),
        input.initial_gas,
        input.initial_pc as usize,
    );

    assert_eq!(result.reason.to_string(), output.expected_status);
    assert_eq!(result.state.pc, output.expected_pc);
    assert_eq!(result.state.registers.to_vec(), output.expected_regs);
    assert_eq!(result.state.gas as u64, output.expected_gas);
    assert_eq!(
        crate::pvmi::to_test_memory(&result.state.memory),
        output.expected_memory
    );
    Ok(())
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
pub fn to_test_memory(memory: &pvm::Memory) -> Vec<Memory> {
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
