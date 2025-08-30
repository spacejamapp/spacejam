//! PVM Compiler test vectors

use crate::pvmi::{to_test_memory, TestInput, TestOutput};
use anyhow::Result;
use pvmc::Compiler;
use serde::{Deserialize, Serialize};
use specjam::Test;
use std::borrow::Cow;
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
            let start = page.address / pvm::PAGE_SIZE as u32;
            let count = (page.length as u32).div_ceil(pvm::PAGE_SIZE as u32);
            memory.allocate(start, count)?;

            // WORKAROUND: adapt to the standard memory layout
            if page.is_writable {
                memory.info.write.start = page.address;
                memory.info.write.end = page.address + page.length as u32;
            } else {
                memory.info.read.start = page.address;
                memory.info.read.end = page.address + page.length as u32;
            }
        }

        // Then write initial memory data
        for mem in &input.initial_memory {
            memory.write_bytes(mem.address, &mem.contents)?;
        }

        // restore original page permissions
        for page in &input.initial_page_map {
            let start = page.address / pvm::PAGE_SIZE as u32;
            let count = (page.length as u32).div_ceil(pvm::PAGE_SIZE as u32);
            for page_idx in start..(start + count) {
                if let Some((data, _)) = memory.memory.get(&page_idx).cloned() {
                    memory.memory.insert(page_idx, (data, page.is_writable));
                }
            }
        }

        let mut compiler = Compiler::new()?;
        let module = compiler.compile(&pvm::Program {
            code: input.program.to_vec(),
            registers: initial_registers,
            memory: memory.clone(),
        })?;

        let result = module.invoke(
            &initial_registers,
            input.initial_pc as u64,
            input.initial_gas as u64,
            memory.clone(),
        )?;

        // assert_eq!(result.pc, output.expected_pc as u64);
        let final_memory = to_test_memory(&result.memory);
        assert_eq!(result.reason.to_string(), output.expected_status);
        assert_eq!(result.registers.to_vec(), output.expected_regs);
        assert_eq!(result.gas, output.expected_gas as u64);
        assert_eq!(final_memory, output.expected_memory);
        Ok(())
    }
}
