//! PVM test vectors

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Memory {
    /// The address of the memory slot.
    pub address: u64,
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
    pub initial_gas: u32,
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
    pub expected_gas: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Page {
    /// The address of the page.
    pub address: u64,
    /// The length of the page.
    pub length: u16,
    /// Whether the page is writable.
    #[serde(alias = "is-writable")]
    pub is_writable: bool,
}

include!(concat!(env!("OUT_DIR"), "/pvm.rs"));
