//! This module contains the implementation of the `Runner` struct, which is used to run the tests.

use anyhow::Result;
use score::{block::History, runtime::tx};
use specjam::{Section, Test};
use tracing_subscriber::EnvFilter;

/// The `Runner` struct which is used to run the tests.
pub struct Runner;

impl Runner {
    /// Step a test.
    pub fn step(test: &Test) -> Result<()> {
        tracing_subscriber::fmt::Subscriber::builder()
            .with_env_filter(EnvFilter::from_default_env())
            .init();

        match test.section {
            Section::Assurances => {
                use crate::assurances;

                let mut input = assurances::TestInput::from_json(test.input)?;
                let assurances::TestOutput { output, post_state } =
                    assurances::TestOutput::from_json(test.output)?;

                assert_eq!(input.pre_state.curr_validators, post_state.curr_validators);

                // validate output
                let result = tx::assurance::available(
                    &input.pre_state.avail_assignments,
                    &input.pre_state.curr_validators,
                    input.input.slot,
                    input.input.parent,
                    &input.input.assurances,
                );
                assert_eq!(result, output.map(|s| s.reported));

                // validate post state
                if let Ok(available) = result {
                    let mut assignments =
                        tx::assurance::reports(input.input.slot, input.pre_state.avail_assignments);

                    // remove the available work reports from the assignments
                    // to get the mark for testing.
                    for work in available {
                        assignments[work.core_index as usize] = None;
                    }
                    input.pre_state.avail_assignments = assignments;
                }

                assert_eq!(
                    input.pre_state.avail_assignments,
                    post_state.avail_assignments,
                );
            }
            Section::Authorizations => {
                use crate::authorizations;

                let input = authorizations::TestInput::from_json(test.input)?;
                let output = authorizations::TestOutput::from_json(test.output)?;
                let state: score::State = input.pre_state.clone().into();
                let post: score::State = output.post_state.clone().into();

                // Validate post state
                let result = tx::guarantee::pools(
                    input.input.slot,
                    &state.pools,
                    &state.authorization,
                    &input.input.auths.into_iter().map(|a| a.into()).collect(),
                );

                assert_eq!(result, post.pools);
                assert_eq!(state.authorization, post.authorization);
            }
            Section::Disputes => {
                use crate::disputes;

                let mut input = disputes::TestInput::from_json(test.input)?;
                let output = disputes::TestOutput::from_json(test.output)?;
                let result = tx::dispute::disputes(
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
                    input.pre_state.rho = tx::dispute::reports(&records, &input.pre_state.rho);
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

                // Validate post state
                let accounts = preimage::to_accounts(input.pre_state.accounts.clone());
                let result =
                    tx::preimage::accounts(input.input.slot, &input.input.preimages, &accounts);
                if let Ok(accounts) = result {
                    assert_eq!(accounts, preimage::to_accounts(output.post_state.accounts));
                } else {
                    assert_eq!(input.pre_state, output.post_state);
                }
            }
            Section::Reports => {
                use crate::reports;

                let reports::TestInput { input, pre_state } =
                    reports::TestInput::from_json(test.input)?;
                let reports::TestOutput { output, post_state } =
                    reports::TestOutput::from_json(test.output)?;

                assert_eq!(pre_state.curr_validators, post_state.curr_validators);
                assert_eq!(pre_state.prev_validators, post_state.prev_validators);
                assert_eq!(pre_state.entropy, post_state.entropy);
                assert_eq!(pre_state.offenders, post_state.offenders);
                assert_eq!(pre_state.auth_pools, post_state.auth_pools);
                assert_eq!(pre_state.services, post_state.services);

                // Validate the output
                let state = pre_state.clone().into();
                let result = tx::guarantee::reports(
                    input.slot,
                    &pre_state.avail_assignments,
                    &input.guarantees,
                )
                .and_then(|assignments| {
                    tx::guarantee::report(&state, input.slot, &input.guarantees)
                        .map(|(reported, reporters)| (reported, reporters, assignments))
                });

                assert_eq!(
                    result
                        .clone()
                        .map(|(reported, reporters, _)| reports::Output {
                            reported,
                            reporters,
                        }),
                    output
                );

                if let Ok((_, _, assignments)) = result {
                    assert_eq!(assignments, post_state.avail_assignments);
                } else {
                    assert_eq!(pre_state, post_state);
                }
            }
            Section::Safrole => {
                use crate::safrole;

                let mut input = safrole::TestInput::from_json(test.input)?;
                let output = safrole::TestOutput::from_json(test.output)?;
                let result = input.pre_state.enact(&input.input);

                assert_eq!(result, output.output);
                assert_eq!(output.post_state, input.pre_state);
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
                for page in &input.initial_page_map {
                    memory.pages.insert(
                        page.address / ::pvmi::PAGE_SIZE,
                        ::pvmi::Page {
                            data: Default::default(),
                            access: ::pvmi::Access::Mutable,
                        },
                    );
                }

                for mem in input.initial_memory {
                    memory.write_bytes(
                        mem.address,
                        mem.address % ::pvmi::PAGE_SIZE,
                        mem.contents.as_slice(),
                    )?;
                }

                for tpage in input.initial_page_map {
                    let page = memory.pages.get_mut(&(tpage.address / ::pvmi::PAGE_SIZE));
                    if let Some(page) = page {
                        page.access = if tpage.is_writable {
                            ::pvmi::Access::Mutable
                        } else {
                            ::pvmi::Access::Immutable
                        };
                    }
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
                    .to_data_maps()
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
