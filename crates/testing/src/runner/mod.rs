//! This module contains the implementation of the `Runner` struct, which is used to run the tests.

use anyhow::Result;
use score::{block::History, safrole::Safrole, validator::Validators};
use specjam::{Section, Test};

/// The `Runner` struct which is used to run the tests.
pub struct Runner;

impl Runner {
    /// Step a test.
    pub fn step(test: &Test) -> Result<()> {
        match test.section {
            Section::Assurances => {
                use crate::assurances;

                let mut input = assurances::TestInput::from_json(test.input)?;
                let assurances::TestOutput { output, post_state } =
                    assurances::TestOutput::from_json(test.output)?;

                // validate output
                let result = sync::assurance::available(
                    &input.pre_state.avail_assignments,
                    &input.pre_state.curr_validators,
                    input.input.slot,
                    input.input.parent,
                    &input.input.assurances,
                );
                assert_eq!(result.clone().map(|(_, a)| a), output.map(|s| s.reported));

                // validate post state
                if let Ok((assignments, _)) = result {
                    input.pre_state.avail_assignments = assignments;
                }

                assert_eq!(post_state, input.pre_state);
            }
            Section::Authorizations => {
                use crate::authorizations;

                let input = authorizations::TestInput::from_json(test.input)?;
                let output = authorizations::TestOutput::from_json(test.output)?;
                let state = input.pre_state;
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

                let mut input = disputes::TestInput::from_json(test.input)?;
                let output = disputes::TestOutput::from_json(test.output)?;
                let result = sync::dispute::disputes(
                    input.pre_state.tau,
                    &input.pre_state.kappa,
                    &input.pre_state.lambda,
                    &input.pre_state.psi,
                    &input.input.disputes,
                );

                // check offenders mark
                assert_eq!(
                    result.clone().map(|(_, mark)| mark.offenders),
                    output.output.map(|v| { v.offenders_mark })
                );

                if let Ok((psi, records)) = result {
                    input.pre_state.psi = psi;
                    input.pre_state.rho = sync::dispute::reports(&records, &input.pre_state.rho);
                }

                // check post state
                assert_eq!(input.pre_state, output.post_state);
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

                let mut state = input.pre_state.clone();
                let new_epoch = input.input.slot / score::EPOCH_LENGTH
                    > input.pre_state.tau / score::EPOCH_LENGTH;

                let safrole = Safrole {
                    validators: state.gamma_k.clone(),
                    series: state.gamma_s.clone(),
                    ring_commitment: state.gamma_z.clone(),
                    accumulator: state.gamma_a.clone(),
                };

                let mut validators = Validators {
                    current: state.kappa.clone(),
                    next: state.iota.clone(),
                    previous: state.lambda.clone(),
                };

                validators = sync::ticket::validators(new_epoch, &safrole.validators, &validators);
                state.eta = sync::ticket::eta(new_epoch, &input.pre_state.eta, input.input.entropy);
                let result = sync::ticket::safrole(
                    state.tau,
                    input.input.slot,
                    state.eta,
                    &state.post_offenders,
                    &safrole,
                    &validators,
                    &input.input.extrinsic,
                );

                assert_eq!(
                    result.clone().map(|safrole| safrole::Markers {
                        epoch_mark: safrole.epoch_mark(new_epoch, &state.eta),
                        tickets_mark: safrole.tickets_mark(state.tau, input.input.slot),
                    }),
                    output.output
                );
                assert_eq!(
                    output.post_state,
                    if result.is_ok() {
                        let safrole = result?;
                        state.gamma_a = safrole.accumulator;
                        state.gamma_k = safrole.validators;
                        state.gamma_s = safrole.series;
                        state.gamma_z = safrole.ring_commitment;
                        state.kappa = validators.current;
                        state.lambda = validators.previous;
                        state.tau = input.input.slot;
                        state
                    } else {
                        input.pre_state
                    }
                );
            }
            Section::Statistics => {
                use crate::statistics;

                let input = statistics::TestInput::from_json(test.input)?;
                let output = statistics::TestOutput::from_json(test.output)?;

                // validate
                let state = input.pre_state.pi.update(
                    input.pre_state.tau,
                    input.input.slot,
                    input.input.author_index,
                    &input.input.extrinsic,
                );
                assert_eq!(state, output.post_state.pi);
            }
            Section::Pvm => {
                use crate::pvm;

                let input: pvm::TestInput = serde_json::from_str(test.input)?;
                let output: pvm::TestOutput = serde_json::from_str(test.output)?;
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
