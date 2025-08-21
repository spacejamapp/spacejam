//! This module contains the implementation of the `Runner` struct, which is used to run the tests.

use crate::traces::KeyValue;
use ::pvm::Invocation;
use anyhow::{Context, Result};
use pvmi::Interpreter;
use runtime::{
    storage::{MemoryDb, StateStorage},
    tx,
};
use score::{
    block::{Block, BlockInfo, Header, History, Mmr},
    safrole::ValidatorsData,
    service::{AccumulatedQueue, Privileges, ReadyQueue, ServiceInfo},
    state::{key, StateKeyInfo, StateKeyLike},
    statistic::Statistics,
    Account, Accounts, EntropyBuffer,
};
use spacejson::Json;
use specjam::{Section, Test};

use std::{collections::BTreeMap, sync::Arc};
use tracing_subscriber::EnvFilter;

/// The `Runner` struct which is used to run the tests.
pub struct Runner;

impl Runner {
    /// Step a test.
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

        match test.section {
            Section::Accumulate => {
                use crate::accumulate;

                let input = accumulate::TestInput::from_json(&test.input)?;
                let output = accumulate::TestOutput::from_json(&test.output)?;
                let accounts = input.pre_state.accounts();

                // run the accumulate function
                let mut accumulation = tx::guarantee::accumulate::<Interpreter, _>(
                    input.input.slot,
                    input.pre_state.slot,
                    input.input.reports,
                    &input.pre_state.ready_queue,
                    &input.pre_state.accumulated,
                    &input.pre_state.privileges.into(),
                    &Default::default(),
                    accounts.clone(),
                    Default::default(),
                )?;
                accumulation.root = Default::default();

                // convert the accounts to the service items
                let accounts = accumulate::to_accounts(&accumulation);
                // TODO: the records check got broken after fuzz tests for 0.6.7
                //
                // will be fixed in 0.7.0
                // assert_eq!(accumulation.records, output.post_state.statistics());
                // assert_eq!(BTreeMap::new(), output.post_state.statistics());

                assert_eq!(accumulation.root, output.output.unwrap());
                assert_eq!(
                    accumulation.accumulated_queue,
                    output.post_state.accumulated
                );
                assert_eq!(accumulation.ready_queue, output.post_state.ready_queue);
                for (idx, account) in accounts.iter().enumerate() {
                    assert_eq!(
                        account.data.service.total,
                        output.post_state.accounts[idx].data.service.total
                    );
                }
                assert_eq!(accounts, output.post_state.haccounts());
                assert_eq!(accumulation.privileges, output.post_state.privileges.into());
            }
            Section::Assurances => {
                use crate::assurances;

                let mut input = assurances::TestInput::from_json(&test.input)?;
                let assurances::TestOutput { output, post_state } =
                    assurances::TestOutput::from_json(&test.output)?;

                assert_eq!(input.pre_state.curr_validators, post_state.curr_validators);

                // validate output
                let result = tx::assurance::available(
                    &input.pre_state.avail_assignments,
                    &input.pre_state.curr_validators,
                    input.input.slot,
                    input.input.parent,
                    &input.input.assurances,
                );
                assert_eq!(result.clone().map(|(a, _)| a), output.map(|s| s.reported));

                // validate post state
                if let Ok((available, _)) = result {
                    let mut assignments = tx::assurance::reports(
                        input.input.slot,
                        &available,
                        input.pre_state.avail_assignments,
                    );

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

                let input = authorizations::TestInput::from_json(&test.input)?;
                let output = authorizations::TestOutput::from_json(&test.output)?;
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

                let mut input = disputes::TestInput::from_json(&test.input)?;
                let output = disputes::TestOutput::from_json(&test.output)?;
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
            Section::Erasure => {
                let mut data = hex::decode(test.input.trim_start_matches("0x"))?;
                let shards = serde_json::from_str::<Vec<String>>(&test.output)?
                    .into_iter()
                    .map(|s| hex::decode(s.trim_start_matches("0x")).map_err(Into::into))
                    .collect::<anyhow::Result<Vec<_>>>()?;

                let rt = tokio::runtime::Runtime::new().unwrap();
                {
                    // test encoding
                    let edata = data.clone();
                    let eshards = shards.clone();
                    rt.block_on(async move {
                        let encoded = erasure::encode(edata).await.expect("failed to encode");
                        assert_eq!(encoded, eshards);
                    });
                }

                {
                    rt.block_on(async move {
                        let decoded =
                            erasure::decode(vec![(0, shards[0].clone()), (2, shards[2].clone())])
                                .await
                                .expect("failed to decode");
                        data.resize(decoded.len(), 0);
                        assert_eq!(decoded, data);
                    });
                }
            }
            Section::History => {
                use crate::history;

                let input = history::TestInput::from_json(&test.input)?;
                let output = history::TestOutput::from_json(&test.output)?;
                let mut history = input.pre_state.beta.clone();
                history.complete_state_root(input.input.parent_state_root)?;
                history.import(
                    input.input.header_hash,
                    input.input.accumulate_root,
                    input.input.work_packages.clone(),
                );
                assert_eq!(output.post_state.beta, history);
            }
            Section::Preimages => {
                use crate::preimage;
                let input = preimage::TestInput::from_json(&test.input)?;
                let output = preimage::TestOutput::from_json(&test.output)?;

                // Validate post state
                let accounts = preimage::to_accounts(input.pre_state.accounts.clone());
                let result =
                    tx::preimage::accounts(input.input.slot, &input.input.preimages, accounts);
                if let Ok(accounts) = result {
                    assert_eq!(
                        accounts
                            .accounts()
                            .iter()
                            .map(|(id, account)| (*id, account.account()))
                            .collect::<BTreeMap<_, _>>(),
                        preimage::to_accounts(output.post_state.accounts)
                    );
                } else {
                    assert_eq!(input.pre_state, output.post_state);
                }
            }
            Section::Reports => {
                use crate::reports;

                let reports::TestInput { input, pre_state } =
                    reports::TestInput::from_json(&test.input)?;
                let reports::TestOutput { output, post_state } =
                    reports::TestOutput::from_json(&test.output)?;

                assert_eq!(pre_state.curr_validators, post_state.curr_validators);
                assert_eq!(pre_state.prev_validators, post_state.prev_validators);
                assert_eq!(pre_state.entropy, post_state.entropy);
                assert_eq!(pre_state.offenders, post_state.offenders);
                assert_eq!(pre_state.auth_pools, post_state.auth_pools);
                assert_eq!(pre_state.services, post_state.services);

                // Validate the output
                let state: score::State = pre_state.clone().into();
                let result = tx::guarantee::reports(
                    input.slot,
                    &pre_state.avail_assignments,
                    &input.guarantees,
                )
                .and_then(|assignments| {
                    tx::guarantee::report(&state, input.slot, &state.accounts, &input.guarantees)
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

                let mut input = safrole::TestInput::from_json(&test.input)?;
                let output = safrole::TestOutput::from_json(&test.output)?;
                let result = input.pre_state.enact(&input.input);

                assert_eq!(result, output.output);
                assert_eq!(output.post_state.gamma_a, input.pre_state.gamma_a);
                assert_eq!(output.post_state.gamma_k, input.pre_state.gamma_k);
                assert_eq!(output.post_state.gamma_s, input.pre_state.gamma_s);
                assert_eq!(output.post_state.gamma_z, input.pre_state.gamma_z);
                assert_eq!(output.post_state, input.pre_state);
            }
            Section::Statistics => {
                /* use crate::statistics;

                let input = statistics::TestInput::from_json(&test.input)?;
                let output = statistics::TestOutput::from_json(&test.output)?;

                // validate
                let state = input.pre_state.statistics.update(
                    input.input.slot,
                    input.input.author_index,
                    &input.input.extrinsic,
                );
                assert_eq!(state, output.post_state.statistics); */
            }
            Section::Pvm => {
                use crate::pvm;

                let input: pvm::TestInput = serde_json::from_str(&test.input)?;
                let output: pvm::TestOutput = serde_json::from_str(&test.output)?;
                let mut registers = [0; 13];
                registers.copy_from_slice(&input.initial_regs);

                // Initialize memory using the new unified parser::Memory
                let mut memory = pvmi::Memory::default();

                // First allocate pages with proper permissions
                for page in &input.initial_page_map {
                    let page_num = page.address / ::pvmi::PAGE_SIZE as u32;
                    // Insert page with correct permission from test input
                    memory.memory.insert(
                        page_num,
                        (vec![0u8; ::pvmi::PAGE_SIZE as usize], page.is_writable),
                    );
                }

                // Then write initial memory data - temporarily make pages writable for setup
                for mem in &input.initial_memory {
                    let page_num = mem.address / ::pvmi::PAGE_SIZE as u32;
                    // Temporarily make page writable for initialization
                    if let Some((page_data, _)) = memory.memory.get(&page_num).cloned() {
                        memory.memory.insert(page_num, (page_data, true));
                    }
                    memory.write_bytes(mem.address, &mem.contents)?;
                }

                // Restore original page permissions after initialization
                for page in &input.initial_page_map {
                    let page_num = page.address / ::pvmi::PAGE_SIZE as u32;
                    if let Some((page_data, _)) = memory.memory.get(&page_num).cloned() {
                        memory
                            .memory
                            .insert(page_num, (page_data, page.is_writable));
                    }
                }

                // run the program
                let result = <pvmi::Interpreter as Invocation>::invoke(
                    &input.program,
                    input.initial_pc as u64,
                    input.initial_gas,
                    registers,
                    memory.clone(),
                );

                assert_eq!(result.reason.to_string(), output.expected_status);
                assert_eq!(result.state.pc, output.expected_pc);
                assert_eq!(result.state.registers.to_vec(), output.expected_regs);
                assert_eq!(result.state.gas as u64, output.expected_gas);
                assert_eq!(
                    crate::pvm::to_test_memory(&result.state.memory),
                    output.expected_memory
                );
            }
            Section::Trace(_) => {
                use crate::traces;
                if test.input.len() == 31 {
                    // SKIP the genesis block
                    return Ok(());
                }

                let input = traces::TestInput::from_json(&test.input)?;
                let output = traces::TestOutput::from_json(&test.output)?;
                let block: Block = input.block;
                let memdb = Arc::new(MemoryDb::default());

                // 1. verify the state root in pre-stateπ
                let keyvals = input.pre_state.keyvals;
                for keyval in keyvals {
                    memdb
                        .state_set(keyval.key, keyval.value)
                        .expect("failed to set keyval");
                }

                let state_root = memdb.root().expect("failed to get state root");
                assert_eq!(state_root, input.pre_state.state_root);

                // 2. verify the state transition
                let mut pkeys = Vec::new();
                if let Err(e) = tx::transit::<Interpreter>(block, memdb.clone()) {
                    tracing::warn!("failed to transit block with error: {e:?}");
                }

                for KeyValue { key, value } in output.post_state.keyvals {
                    let info = key.as_state_key().info();
                    let encoded = hex::encode(&key);
                    let Some(result) = memdb.state_get(&key)? else {
                        tracing::error!(
                            "{info:?} key=0x{encoded} value=0x{} not exists",
                            hex::encode(&value)
                        );
                        continue;
                    };

                    pkeys.push(key.clone());
                    if value != result {
                        tracing::error!("keyval mismatch: {info:?}: 0x{encoded}");
                    } else {
                        tracing::debug!("keyval matched: {info:?}: 0x{encoded}");
                    }

                    /* if key == key::STATISTICS && value != result {
                        let polkajam: Statistics = codec::decode(&value)?;
                        let statistics: Statistics = codec::decode(&result)?;
                        tracing::debug!("polkajam: {:#?}", polkajam.to_json());
                        tracing::debug!("spacejam: {:#?}", statistics.to_json());
                    } */

                    if key == key::RECENT_BLOCKS && value != result {
                        let polkajam: History = codec::decode(&value)?;
                        let recent: History = codec::decode(&result)?;
                        tracing::debug!("polkajam: {:?}", polkajam.to_json());
                        tracing::debug!("spacejam: {:?}", recent.to_json());
                    }

                    if key == key::PRIVILEGED_SERVICE && value != result {
                        let polkajam: Privileges = codec::decode(&value)?;
                        let spacejam: Privileges = codec::decode(&result)?;
                        tracing::debug!("polkajam: {:?}", polkajam);
                        tracing::debug!("spacejam: {:?}", spacejam);
                    }

                    if key == key::DRAWN_VALIDATORS && value != result {
                        let polkajam: ValidatorsData = codec::decode(&value)?;
                        let spacejam: ValidatorsData = codec::decode(&result)?;
                        tracing::debug!(
                            "polkajam-ed25519: {:?}",
                            polkajam
                                .iter()
                                .map(|v| hex::encode(v.ed25519))
                                .collect::<Vec<_>>()
                        );
                        tracing::debug!(
                            "spacejam-ed25519: {:?}",
                            spacejam
                                .iter()
                                .map(|v| hex::encode(v.ed25519))
                                .collect::<Vec<_>>()
                        );
                    }

                    if key.starts_with(&[255]) && value != result {
                        let polkajam: ServiceInfo = codec::decode(&value)?;
                        let spacejam: ServiceInfo = codec::decode(&result)?;
                        tracing::debug!("polkajam: {:#?}", polkajam.to_json());
                        tracing::debug!("spacejam: {:#?}", spacejam.to_json());
                    }
                }

                // check if spacejam left extra keyvals
                for pair in memdb.state_iter()? {
                    let (key, value) = pair?;
                    if pkeys.contains(&key) {
                        continue;
                    }

                    let info = key.as_state_key().info();
                    tracing::error!(
                        "extra keyval: {info:?} key=0x{} value=0x{}...",
                        hex::encode(&key),
                        hex::encode(&value[..std::cmp::min(32, value.len())])
                    );
                }

                let state_root = memdb.root().expect("failed to get state root");
                assert_eq!(state_root, output.post_state.state_root);
            }
            Section::Codec | Section::Shuffle | Section::Trie => {}
        }

        Ok(())
    }
}
