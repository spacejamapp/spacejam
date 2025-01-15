//! This module contains the implementation of the `Runner` struct, which is used to run the tests.

use anyhow::Result;
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
            _ => {}
        }

        Ok(())
    }
}
