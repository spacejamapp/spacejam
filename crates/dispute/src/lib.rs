use codec::Json;
use error::Result;
use score::{
    dispute::{DisputesExtrinsic, DisputesRecords, DisputesRecordsJson},
    misc::{
        AvailabilityAssignment, AvailabilityAssignmentJson, Ed25519Public, TimeSlot,
        ValidatorDataJson, ValidatorsData,
    },
};
use serde::{Deserialize, Serialize};

pub mod error;

#[derive(Debug, PartialEq, Eq, Json, Serialize, Deserialize, Clone)]
pub struct State {
    #[json(nested)]
    psi: DisputesRecords,
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    rho: Vec<Option<AvailabilityAssignment>>,
    tau: TimeSlot,
    #[json(Vec<ValidatorDataJson>)]
    kappa: ValidatorsData,
    #[json(Vec<ValidatorDataJson>)]
    lambda: ValidatorsData,
}

/// Disputes handler
pub struct Disputes {
    pub state: State,
    pub next_state: State,
}

#[derive(Json, Serialize, Deserialize, Debug)]
pub struct OffendersMark {
    #[json(Vec<String>)]
    offenders_mark: Vec<Ed25519Public>,
}

impl Disputes {
    /// Handle an extrinsic
    pub fn handle(&mut self, _extrinsic: DisputesExtrinsic) -> Result<OffendersMark> {
        Ok(OffendersMark {
            offenders_mark: vec![],
        })
    }
}

impl From<State> for Disputes {
    fn from(state: State) -> Self {
        Self {
            next_state: state.clone(),
            state,
        }
    }
}
