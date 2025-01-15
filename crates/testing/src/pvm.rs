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

/// PVM test case
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Test {
    name: String,
    #[serde(alias = "initial-regs")]
    initial_regs: Vec<u32>,
    #[serde(alias = "initial-pc")]
    initial_pc: u32,
    #[serde(alias = "initial-memory")]
    initial_memory: Vec<Memory>,
    #[serde(alias = "initial-gas")]
    initial_gas: u32,
    program: Vec<u8>,
    #[serde(alias = "expected-pc")]
    expected_pc: usize,
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
    fn run(self) {
        let mut registers = [0; 13];
        registers.copy_from_slice(&self.initial_regs);

        // Initialize memory
        let mut memory = pvmi::Memory::default();
        for mem in self.initial_memory {
            memory.slots.insert(mem.address, mem.contents.clone());
        }

        // Initialize interpreter
        let mut interpreter = pvmi::Interpreter::default()
            .gas(self.initial_gas)
            .registers(registers)
            .memory(memory);

        interpreter
            .interp(&self.program)
            .expect("failed to run program");

        let expected_memory = interpreter
            .memory
            .slots
            .iter()
            .map(|(k, v)| Memory {
                address: *k,
                contents: v.to_vec(),
            })
            .collect::<Vec<_>>();

        assert_eq!(interpreter.pc, self.expected_pc);
        assert_eq!(interpreter.status.to_string(), self.expected_status);
        assert_eq!(interpreter.registers.to_vec(), self.expected_regs);
        assert_eq!(interpreter.gas, self.expected_gas);
        assert_eq!(expected_memory, self.expected_memory);
    }
}

include!(concat!(env!("OUT_DIR"), "/pvm.rs"));
