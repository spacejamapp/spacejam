//! PVM test vectors

use serde::{Deserialize, Serialize};

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

/// Convert parser::Memory to test format for comparison
pub fn to_test_memory(memory: &pvmi::Memory) -> Vec<Memory> {
    let mut result = Vec::new();

    for (&page_num, (page_data, _)) in &memory.memory {
        // Find non-zero segments in this page
        let base_addr = page_num * pvmi::PAGE_SIZE as u32;
        let mut current_start = None;
        let mut current_data = Vec::new();

        for (offset, &byte) in page_data.iter().enumerate() {
            if byte != 0 {
                match current_start {
                    Some(start_offset) => {
                        // Continue current segment
                        if offset == start_offset + current_data.len() {
                            current_data.push(byte);
                        } else {
                            // Gap found, save current segment and start new one
                            if !current_data.is_empty() {
                                result.push(Memory {
                                    address: base_addr + start_offset as u32,
                                    contents: current_data,
                                });
                            }
                            current_start = Some(offset);
                            current_data = vec![byte];
                        }
                    }
                    None => {
                        // Start new segment
                        current_start = Some(offset);
                        current_data = vec![byte];
                    }
                }
            } else if current_start.is_some() && !current_data.is_empty() {
                // End current segment on zero byte
                result.push(Memory {
                    address: base_addr + current_start.unwrap() as u32,
                    contents: current_data,
                });
                current_start = None;
                current_data = Vec::new();
            }
        }

        // Don't forget the last segment
        if let Some(start_offset) = current_start {
            if !current_data.is_empty() {
                result.push(Memory {
                    address: base_addr + start_offset as u32,
                    contents: current_data,
                });
            }
        }
    }

    // Sort by address for consistent comparison
    result.sort_by_key(|m| m.address);
    result
}

include!(concat!(env!("OUT_DIR"), "/pvm.rs"));
