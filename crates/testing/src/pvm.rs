use serde::{Deserialize, Serialize};

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
            let mut contents = [0; 4];
            contents[..mem.contents.len()].copy_from_slice(&mem.contents);
            memory.slots.insert(mem.address, contents);
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
            .map(|(k, v)| {
                let mut contents = v.to_vec();
                while let Some(0) = contents.last() {
                    contents.pop();
                }

                Memory {
                    address: *k,
                    contents,
                }
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
