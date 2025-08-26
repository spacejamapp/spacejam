//! PVM Compiler test vectors

use std::borrow::Cow;

use crate::pvmi::{to_test_memory, TestInput, TestOutput};
use anyhow::Result;
use pvmc::Compiler;
use serde::{Deserialize, Serialize};
use specjam::Test;
use tracing_subscriber::EnvFilter;

include!(concat!(env!("OUT_DIR"), "/pvmc.rs"));

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
        let mut initial_registers = [0u64; pvm::REGISTER_COUNT];
        initial_registers.copy_from_slice(&input.initial_regs);

        // Initialize memory from test input
        let mut memory = pvm::Memory::default();
        for page in &input.initial_page_map {
            let index = page.address / pvm::PAGE_SIZE as u32;
            let data = vec![0u8; pvm::PAGE_SIZE as usize];
            memory.memory.insert(index, (data, true));

            // WORKAROUND: adapt to the standard memory layout
            if page.is_writable {
                memory.write.start = page.address;
                memory.write.end = page.address + page.length as u32;
            } else {
                memory.read.start = page.address;
                memory.read.end = page.address + page.length as u32;
            }
        }

        // Then write initial memory data
        for mem in &input.initial_memory {
            memory.write_bytes(mem.address, &mem.contents)?;
        }

        let mut compiler = Compiler::new()?;
        let module = compiler.compile(&pvm::Program {
            code: Cow::Borrowed(&input.program),
            registers: initial_registers,
            memory: memory.clone(),
        })?;

        let result = module.execute(
            &initial_registers,
            input.initial_pc as u64,
            input.initial_gas as u64,
            memory.clone(),
        )?;

        assert_eq!(result.registers.to_vec(), output.expected_regs);
        // assert_eq!(result.pc, output.expected_pc as u64);
        // assert_eq!(result.gas, output.expected_gas as u64);

        // Validate memory state using helper function
        let final_memory_test = to_test_memory(&result.memory);
        assert_eq!(final_memory_test, output.expected_memory);
        Ok(())
    }
}
