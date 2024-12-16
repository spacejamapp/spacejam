use serde::{Deserialize, Serialize};

/// PVM test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Test {
    name: String,
    initial_regs: Vec<u8>,
    initial_pc: u32,
    initial_page_map: Vec<u8>,
    initial_memory: Vec<u8>,
    initial_gas: u32,
    program: Vec<u8>,
    expected_status: String,
    expected_regs: Vec<u8>,
    expected_memory: Vec<u8>,
    expected_gas: u32,
}
