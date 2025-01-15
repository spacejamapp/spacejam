//! This module contains the implementation of the `Runner` struct, which is used to run the tests.

use anyhow::Result;
use score::block::BlocksHistory;
use specjam::{Section, Test};
use statistic::Stats;

/// The `Runner` struct which is used to run the tests.
pub struct Runner;

impl Runner {
    /// Step a test.
    pub fn step(test: &Test) -> Result<()> {
        match test.section {
            Section::Assurances => {
                use crate::assurances;

                let input = assurances::TestInput::from_json(test.input)?;
                let output = assurances::TestOutput::from_json(test.output)?;
                let mut handler = assurance::Handler::from(input.pre_state);
                let result = handler.handle(input.input);
                assert_eq!(result, output.output);
                assert_eq!(handler.post_state, output.post_state);
            }
            Section::Authorizations => {
                use crate::authorizations;

                let input = authorizations::TestInput::from_json(test.input)?;
                let output = authorizations::TestOutput::from_json(test.output)?;
                let state = authorizations::TestState::from(input.pre_state);
                let result = guarantee::auth::handle(
                    state.into(),
                    input.input.slot,
                    input.input.auths.into_iter().map(|a| a.into()).collect(),
                )?;
                assert_eq!(result, output.post_state.into());
            }
            Section::Disputes => {
                use crate::disputes;

                let input = disputes::TestInput::from_json(test.input)?;
                let output = disputes::TestOutput::from_json(test.output)?;
                let mut handler = dispute::DisputesHandler::from(input.pre_state);
                let result = handler.handle(input.input.disputes);
                assert_eq!(result, output.output);
                assert_eq!(handler.next_state, output.post_state);
            }
            Section::History => {
                use crate::history;

                let input = history::TestInput::from_json(test.input)?;
                let output = history::TestOutput::from_json(test.output)?;
                let mut history = BlocksHistory {
                    blocks: input.pre_state.beta,
                };
                history.import(
                    input.input.header_hash,
                    input.input.parent_state_root,
                    input.input.accumulate_root,
                    input.input.work_packages.clone(),
                );
                assert_eq!(history.blocks, output.post_state.beta);
            }
            Section::Preimages => {
                use crate::preimage;

                let input = preimage::TestInput::from_json(test.input)?;
                let output = preimage::TestOutput::from_json(test.output)?;
                let pre = preimage::to_state(input.pre_state.accounts);
                let post = preimage::to_state(output.post_state.accounts);

                let result =
                    ::preimage::handle(pre.clone(), input.input.slot, input.input.preimages)
                        .unwrap_or(pre);
                assert_eq!(result, post);
            }
            Section::Reports => {
                use crate::reports;

                let input = reports::TestInput::from_json(test.input)?;
                let output = reports::TestOutput::from_json(test.output)?;

                let mut handler = guarantee::Handler::from(input.pre_state);
                let result = handler.handle(input.input);
                assert_eq!(result, output.output);
                assert_eq!(handler.next, output.post_state);
            }
            Section::Safrole => {
                use crate::safrole;

                let mut input = safrole::TestInput::from_json(test.input)?;
                let output = safrole::TestOutput::from_json(test.output)?;

                let result = input
                    .pre_state
                    .enact(
                        input.input.slot,
                        input.input.entropy,
                        input.input.extrinsic.clone(),
                    )
                    .expect("could not enact epoch change");
                assert_eq!(result, output.output);
                assert_eq!(input.pre_state, output.post_state);
            }
            Section::Statistics => {
                use crate::statistics;

                let input = statistics::TestInput::from_json(test.input)?;
                let output = statistics::TestOutput::from_json(test.output)?;

                let mut stats = Stats::from(input.pre_state);
                stats = stats.update(
                    input.input.slot,
                    input.input.author_index,
                    input.input.extrinsic,
                );

                assert_eq!(stats.next_state, output.post_state);
            }
            Section::Pvm => {
                use crate::pvm;

                let input: pvm::TestInput = serde_json::from_str(&test.input)?;
                let output: pvm::TestOutput = serde_json::from_str(&test.output)?;
                let mut registers = [0; 13];
                registers.copy_from_slice(&input.initial_regs);

                // Initialize memory
                let mut memory = pvmi::Memory::default();
                for mem in input.initial_memory {
                    memory.slots.insert(mem.address, mem.contents.clone());
                }

                // Initialize interpreter
                let mut interpreter = pvmi::Interpreter::default()
                    .gas(input.initial_gas)
                    .registers(registers)
                    .memory(memory);

                interpreter
                    .interp(&input.program)
                    .expect("failed to run program");

                let expected_memory = interpreter
                    .memory
                    .slots
                    .iter()
                    .map(|(k, v)| pvm::Memory {
                        address: *k,
                        contents: v.to_vec(),
                    })
                    .collect::<Vec<_>>();

                assert_eq!(interpreter.pc, output.expected_pc);
                assert_eq!(interpreter.status.to_string(), output.expected_status);
                assert_eq!(interpreter.registers.to_vec(), output.expected_regs);
                assert_eq!(interpreter.gas, output.expected_gas);
                assert_eq!(expected_memory, output.expected_memory);
            }
            _ => {}
        }

        Ok(())
    }
}
