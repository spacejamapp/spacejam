//! Reporting is the process of reporting the results of a work-package to the service state singleton.

use error::Result;
use score::{extrinsic::GuaranteesExtrinsic, misc::TimeSlot};
use state::{Output, State};

pub mod error;
pub mod state;

pub struct Handler {
    pub prev: State,
    pub next: State,
}

impl Handler {
    /// Handle a work report.
    pub fn handle(_guarantees: GuaranteesExtrinsic, _slot: TimeSlot) -> Result<Output> {
        Ok(Output {
            reported: vec![],
            reporters: vec![],
        })
    }
}

impl From<State> for Handler {
    fn from(state: State) -> Self {
        Self {
            prev: state.clone(),
            next: state,
        }
    }
}
