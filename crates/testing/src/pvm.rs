use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PageMap {
    address: u32,
    length: u32,
    #[serde(alias = "is-writable")]
    is_writable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct Memory {
    address: u32,
    contents: Vec<u8>,
}

/// PVM test case
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Test {
    name: String,
    #[serde(alias = "initial-regs")]
    initial_regs: Vec<u32>,
    #[serde(alias = "initial-pc")]
    initial_pc: u32,
    #[serde(alias = "initial-page-map")]
    initial_page_map: Vec<PageMap>,
    #[serde(alias = "initial-memory")]
    initial_memory: Vec<Memory>,
    #[serde(alias = "initial-gas")]
    initial_gas: u32,
    program: Vec<u8>,
    #[serde(alias = "expected-status")]
    expected_status: String,
    #[serde(alias = "expected-regs")]
    expected_regs: Vec<u32>,
    #[serde(alias = "expected-memory")]
    expected_memory: Vec<Memory>,
    #[serde(alias = "expected-gas")]
    expected_gas: u32,
}

impl Test {
    /// Parse a test from a JSON string
    fn from_json(s: &str) -> anyhow::Result<Self> {
        serde_json::from_str(s).map_err(Into::into)
    }

    /// Run the test
    fn run(self) {}
}

include!(concat!(env!("OUT_DIR"), "/pvm.rs"));
