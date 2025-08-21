//! PVM test vectors

use pvm::Invocation;
use serde::{Deserialize, Serialize};

include!(concat!(env!("OUT_DIR"), "/pvm.rs"));

/// Run the PVM test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let input: TestInput = serde_json::from_str(&test.input)?;
    let output: TestOutput = serde_json::from_str(&test.output)?;
    let mut registers = [0; 13];
    registers.copy_from_slice(&input.initial_regs);

    // Initialize memory
    let mut memory = pvmi::Memory::default();
    for page in &input.initial_page_map {
        memory.pages.insert(
            page.address / ::pvmi::PAGE_SIZE as u32,
            ::pvmi::Page {
                data: [0; ::pvmi::PAGE_SIZE as usize],
                access: ::pvmi::Access::Mutable,
            },
        );
    }

    for mem in input.initial_memory {
        memory.write_bytes(
            mem.address / ::pvmi::PAGE_SIZE as u32,
            mem.address % ::pvmi::PAGE_SIZE as u32,
            mem.contents.as_slice(),
        )?;
    }

    for tpage in input.initial_page_map {
        let page = memory
            .pages
            .get_mut(&(tpage.address / ::pvmi::PAGE_SIZE as u32));
        if let Some(page) = page {
            page.access = if tpage.is_writable {
                ::pvmi::Access::Mutable
            } else {
                ::pvmi::Access::Immutable
            };
        }
    }

    // test with the new interface
    let result = <pvmi::Interpreter as Invocation>::invoke(
        &input.program,
        input.initial_pc as u64,
        input.initial_gas,
        registers,
        memory.clone(),
    );

    assert_eq!(result.reason.to_string(), output.expected_status);
    assert_eq!(result.state.pc, output.expected_pc as u64);
    assert_eq!(result.state.registers.to_vec(), output.expected_regs);
    assert_eq!(result.state.gas as u64, output.expected_gas);
    assert_eq!(
        result
            .state
            .memory
            .to_data_maps()
            .iter()
            .map(|(k, v)| Memory {
                address: *k,
                contents: v.to_vec(),
            })
            .collect::<Vec<_>>(),
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
