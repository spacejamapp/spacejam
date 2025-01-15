//! This module contains the implementation of the `Runner` struct, which is used to run the tests.

use anyhow::Result;
use score::block::BlocksHistory;
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
            _ => {}
        }

        Ok(())
    }
}
