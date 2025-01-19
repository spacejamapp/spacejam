//! This module contains the implementation of the `Runner` struct, which is used to run the tests.

use anyhow::Result;
use score::block::History;
use specjam::{Section, Test};

/// The `Runner` struct which is used to run the tests.
pub struct Runner;

impl Runner {
    /// Step a test.
    pub fn step(test: &Test) -> Result<()> {
        match test.section {
            Section::Assurances => {
                use crate::assurances;

                let input = assurances::TestInput::from_json(test.input)?;
                let assurances::TestOutput { output, post_state } =
                    assurances::TestOutput::from_json(test.output)?;

                // validate output
                let mut context = input.pre_state.clone().into();
                let result = sync::assurance::validate(&mut context, &input.input.into());
                assert_eq!(result, output.map(|s| s.reported));

                // validate post state
                assert_eq!(post_state, context.into());
            }
            Section::Authorizations => {
                use crate::authorizations;

                let input = authorizations::TestInput::from_json(test.input)?;
                let output = authorizations::TestOutput::from_json(test.output)?;
                let state = authorizations::TestState::from(input.pre_state);
                let mut context = state.into();
                let mut block = score::Block::default();
                block.header.slot = input.input.slot;
                block.extrinsic.guarantees =
                    input.input.auths.into_iter().map(|a| a.into()).collect();

                // Validate post state
                sync::guarantee::auth::validate(&mut context, &block)?;
                assert_eq!(context, output.post_state.into());
            }
            Section::Disputes => {
                use crate::disputes;

                let input = disputes::TestInput::from_json(test.input)?;
                let output = disputes::TestOutput::from_json(test.output)?;
                let mut state = input.pre_state.clone().into();
                let mut block = score::Block::default();
                block.extrinsic.disputes = input.input.disputes.clone();
                let result = sync::dispute::transit(&block, &mut state);

                assert_eq!(result, output.output.map(|v| { v.offenders_mark }));
                assert_eq!(
                    if result.is_err() {
                        input.pre_state
                    } else {
                        state.into()
                    },
                    output.post_state
                );
            }
            Section::History => {
                use crate::history;

                let input = history::TestInput::from_json(test.input)?;
                let output = history::TestOutput::from_json(test.output)?;
                let mut history = input.pre_state.beta.clone();
                history.import(
                    input.input.header_hash,
                    input.input.parent_state_root,
                    input.input.accumulate_root,
                    input.input.work_packages.clone(),
                );
                assert_eq!(output.post_state.beta, history);
            }
            Section::Preimages => {
                use crate::preimage;

                let input = preimage::TestInput::from_json(test.input)?;
                let output = preimage::TestOutput::from_json(test.output)?;
                let pre = preimage::to_state(input.pre_state.accounts);
                let post = preimage::to_state(output.post_state.accounts);

                let mut context = pre.clone();
                let mut block = score::Block::default();
                block.header.slot = input.input.slot;
                block.extrinsic.preimages = input.input.preimages.clone();

                // Validate post state
                if sync::preimage::validate(&mut context, &block).is_ok() {
                    assert_eq!(context, post);
                } else {
                    assert_eq!(pre, post);
                }
            }
            Section::Reports => {
                use crate::reports;

                let reports::TestInput { input, pre_state } =
                    reports::TestInput::from_json(test.input)?;
                let reports::TestOutput { output, post_state } =
                    reports::TestOutput::from_json(test.output)?;
                let mut context = pre_state.clone().into();

                // Validate the output
                let result = sync::guarantee::validate(&mut context, &input.into());
                assert_eq!(
                    result.map(|(reported, reporters)| reports::Output {
                        reported,
                        reporters,
                    }),
                    output
                );

                // validate the post state
                let mut state: sync::guarantee::State = context.into();
                state.services = pre_state.services;
                assert_eq!(post_state, state);
            }
            Section::Safrole => {
                use crate::safrole;

                let input = safrole::TestInput::from_json(test.input)?;
                let output = safrole::TestOutput::from_json(test.output)?;

                let mut block = score::Block::default();
                block.header.slot = input.input.slot;
                block.extrinsic.tickets = input.input.extrinsic.clone();

                let mut context = input.pre_state.into();
                let result = sync::ticket::validate(&mut context, &block, input.input.entropy).map(
                    |(epoch_mark, tickets_mark)| crate::safrole::Markers {
                        epoch_mark,
                        tickets_mark,
                    },
                );

                assert_eq!(result, output.output);
                assert_eq!(context, output.post_state.into());
            }
            Section::Statistics => {
                use crate::statistics;

                let input = statistics::TestInput::from_json(test.input)?;
                let output = statistics::TestOutput::from_json(test.output)?;

                // construct inputs
                let mut block = score::Block::default();
                block.header.slot = input.input.slot;
                block.header.author_index = input.input.author_index;
                block.extrinsic = input.input.extrinsic.clone();
                let mut context = input.pre_state.into();

                // validate
                sync::statistic::validate(&mut context, &block);
                assert_eq!(context, output.post_state.into());
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
