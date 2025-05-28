//! The state of the reporting portion of the protocol.

use score::{
    CORES_COUNT, Ed25519Public, EntropyBuffer, OpaqueHash,
    block::{BlockInfo, BlockInfoJson},
    safrole::{ValidatorDataJson, ValidatorsData},
    service::{
        AvailabilityAssignmentJson, AvailabilityAssignments, ServiceAccountData, ServiceItem,
        ServiceItemJson,
    },
};
use serde::{Deserialize, Serialize};
use spacejson::Json;

#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct State {
    /// (ρ‡) Intermediate pending reports after that any work report judged as
    /// uncertain or invalid has been removed from it (ϱ†), and the availability
    /// assurances are processed. Mutated to ϱ'.
    #[json(Vec<Option<AvailabilityAssignmentJson>>)]
    pub avail_assignments: AvailabilityAssignments,

    /// (κ') Posterior active validators.
    #[json(Vec<ValidatorDataJson>)]
    pub curr_validators: ValidatorsData,

    /// (λ') Posterior previous validators.
    #[json(Vec<ValidatorDataJson>)]
    pub prev_validators: ValidatorsData,

    /// (η') Posterior entropy buffer.
    #[json(Vec<String>)]
    pub entropy: EntropyBuffer,

    /// (ψ'_o) Posterior offenders.
    #[json(Vec<String>)]
    pub offenders: Vec<Ed25519Public>,

    /// (β) Recent blocks.
    #[json(Vec<BlockInfoJson>)]
    pub recent_blocks: Vec<BlockInfo>,

    /// (α') Authorization pools.
    #[json(Vec<Vec<String>>)]
    pub auth_pools: [Vec<OpaqueHash>; CORES_COUNT],

    /// (δ) Encoded services dictionary. Refer to T(σ) in Appendix D.
    #[json(nested)]
    #[serde(alias = "accounts")]
    pub services: Vec<ServiceItem>,
}

impl State {
    /// Apply the state to the score state
    fn apply(self, state: &mut score::State) {
        state.reports = self.avail_assignments;
        state.validators.current = self.curr_validators;
        state.validators.previous = self.prev_validators;
        state.entropy = self.entropy;
        state.disputes.offenders = self.offenders;
        state.recent_blocks = self.recent_blocks;
        state.authorization = self.auth_pools;

        for ServiceItem { id, data } in self.services.into_iter() {
            state.accounts.entry(id).or_default().code = data.service.code;
            state.accounts.entry(id).and_modify(|account| {
                account.balance = data.service.balance;
                account.gas = data.service.gas;
            });
        }
    }
}

impl From<State> for score::State {
    fn from(value: State) -> Self {
        let mut state = score::State::default();
        value.apply(&mut state);
        state
    }
}

impl From<score::State> for State {
    fn from(value: score::State) -> Self {
        Self {
            avail_assignments: value.reports,
            curr_validators: value.validators.current,
            prev_validators: value.validators.previous,
            entropy: value.entropy,
            offenders: value.disputes.offenders,
            recent_blocks: value.recent_blocks,
            auth_pools: value.authorization,
            services: value
                .accounts
                .into_iter()
                .map(|(id, service)| ServiceItem {
                    id,
                    data: ServiceAccountData {
                        service: service.state(),
                        preimages: vec![],
                        storage: vec![],
                    },
                })
                .collect(),
        }
    }
}
