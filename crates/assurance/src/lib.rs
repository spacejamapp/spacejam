//! Assurance is the process of ensuring that the results of a work-package are available to a super-majority of validators.

use {
    error::Result,
    state::{Input, Output, State},
};

pub mod error;
pub mod state;

pub struct Handler {
    pub prev_state: State,
    pub post_state: State,
}

impl Handler {
    pub fn handle(&self, _input: Input) -> Result<Output> {
        Ok(Output { reported: vec![] })
    }
}

impl Handler {
    pub fn from(state: State) -> Self {
        Self {
            prev_state: state.clone(),
            post_state: state,
        }
    }
}
