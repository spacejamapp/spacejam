use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Memory {
    pub address: u32,
    pub contents: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestInput {
    pub name: String,
    #[serde(alias = "initial-regs")]
    pub initial_regs: Vec<u32>,
    #[serde(alias = "initial-pc")]
    pub initial_pc: u32,
    #[serde(alias = "initial-memory")]
    pub initial_memory: Vec<Memory>,
    #[serde(alias = "initial-gas")]
    pub initial_gas: u32,
    pub program: Vec<u8>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TestOutput {
    #[serde(alias = "expected-pc")]
    pub expected_pc: usize,
    #[serde(alias = "expected-status")]
    pub expected_status: String,
    #[serde(alias = "expected-regs")]
    pub expected_regs: Vec<u32>,
    #[serde(alias = "expected-memory")]
    pub expected_memory: Vec<Memory>,
    #[serde(alias = "expected-gas")]
    pub expected_gas: u32,
}

include!(concat!(env!("OUT_DIR"), "/pvm.rs"));
