//! PVM Compiler test vectors

use std::borrow::Cow;

use crate::pvmi::{to_test_memory, TestInput, TestOutput};
use anyhow::Result;
use pvmc::Compiler;
use serde::{Deserialize, Serialize};
use specjam::Test;
use tracing_subscriber::EnvFilter;

include!(concat!(env!("OUT_DIR"), "/pvm.rs"));

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
        let mut initial_memory = pvm::Memory::default();

        // First, allocate pages with proper permissions
        for page_info in &input.initial_page_map {
            let page_num = page_info.address / pvm::PAGE_SIZE as u32;
            let page_data = vec![0u8; pvm::PAGE_SIZE as usize];
            // Initially set all pages as writable for data initialization
            initial_memory.memory.insert(page_num, (page_data, true));
        }

        // Then write initial memory data
        for mem in &input.initial_memory {
            initial_memory.write_bytes(mem.address, &mem.contents)?;
        }

        // Finally, set correct page permissions
        for page_info in &input.initial_page_map {
            let page_num = page_info.address / pvm::PAGE_SIZE as u32;
            if let Some((_page_data, writable)) = initial_memory.memory.get_mut(&page_num) {
                *writable = page_info.is_writable;
            }
        }

        let mut compiler = Compiler::new()?;
        let module = compiler.compile(&pvm::Program {
            code: Cow::Borrowed(&input.program),
            registers: initial_registers,
            memory: initial_memory.clone(),
        })?;

        let result = module.execute(
            &initial_registers,
            input.initial_pc as u64,
            initial_memory.clone(),
        )?;

        assert_eq!(result.registers.len(), pvm::REGISTER_COUNT);
        assert_eq!(result.registers.to_vec(), output.expected_regs);
        assert_eq!(result.pc, output.expected_pc as u64);

        // Validate memory state using helper function
        let final_memory_test = to_test_memory(&result.memory);
        assert_eq!(final_memory_test, output.expected_memory);
        Ok(())
    }
}
