//! Report testing types

use runtime::tx::{
    self,
    guarantee::error::{Error, Result},
};
use score::{
    block::{History, HistoryJson},
    extrinsic::{GuaranteesExtrinsic, ReportGuaranteeJson},
    service::{ReportedWorkPackage, ReportedWorkPackageJson},
    Block, Ed25519Public, OpaqueHash, TimeSlot,
};
use serde::{Deserialize, Serialize};
use spacejson::{Json, ResultJson};
pub use types::*;

include!(concat!(env!("OUT_DIR"), "/reports.rs"));

/// Run the reports test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let TestInput { input, pre_state } = TestInput::from_json(&test.input)?;
    let TestOutput { output, post_state } = TestOutput::from_json(&test.output)?;

    assert_eq!(pre_state.curr_validators, post_state.curr_validators);
    assert_eq!(pre_state.prev_validators, post_state.prev_validators);
    assert_eq!(pre_state.entropy, post_state.entropy);
    assert_eq!(pre_state.offenders, post_state.offenders);
    assert_eq!(pre_state.auth_pools, post_state.auth_pools);
    assert_eq!(pre_state.services, post_state.services);

    // Validate the output
    let state: score::State = pre_state.clone().into();
    let result =
        tx::guarantee::reports(input.slot, &pre_state.avail_assignments, &input.guarantees)
            .and_then(|assignments| {
                tx::guarantee::report(&state, input.slot, &state.accounts, &input.guarantees)
                    .map(|(reported, reporters)| (reported, reporters, assignments))
            });

    assert_eq!(
        result.clone().map(|(reported, reporters, _)| Output {
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
    Ok(())
}

/// Test input.
#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct TestInput {
    #[json(nested)]
    pub input: Input,
    #[json(nested)]
    pub pre_state: State,
}

/// Test output.
#[derive(Debug, Serialize, Deserialize, Json, Clone)]
pub struct TestOutput {
    #[json(ResultJson<OutputJson, Error>)]
    pub output: Result<Output>,
    #[json(nested)]
    pub post_state: State,
}

/// Input of the reporting module.
#[derive(Debug, Clone, Serialize, Deserialize, Json)]
pub struct Input {
    pub slot: TimeSlot,
    #[json(Vec<ReportGuaranteeJson>)]
    pub guarantees: GuaranteesExtrinsic,
    #[json(Vec<String>)]
    pub known_packages: Vec<OpaqueHash>,
}

impl From<Input> for Block {
    fn from(value: Input) -> Self {
        let mut block = Block::default();
        block.header.slot = value.slot;
        block.extrinsic.guarantees = value.guarantees;
        block
    }
}

/// Output of the reporting module.
#[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
pub struct Output {
    #[json(nested)]
    pub reported: Vec<ReportedWorkPackage>,
    #[json(Vec<String>)]
    pub reporters: Vec<Ed25519Public>,
}

mod types {
    use score::{
        block::{History, HistoryJson},
        safrole::{ValidatorDataJson, ValidatorsData},
        service::{
            AvailabilityAssignmentJson, AvailabilityAssignments, ServiceAccount, ServiceInfo,
            ServiceInfoJson,
        },
        AccountInnerKey, Ed25519Public, EntropyBuffer, OpaqueHash, ServiceId, CORES_COUNT,
    };
    use serde::{Deserialize, Serialize};
    use spacejson::Json;
    use std::collections::BTreeMap;

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
        #[json(nested)]
        pub recent_blocks: History,

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
            state.pools = self.auth_pools;

            for ServiceItem { id, data } in self.services.into_iter() {
                state.accounts.entry(id).or_default().info = data.service;
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
                auth_pools: value.pools,
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

    /// Represents a service item.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct ServiceItem {
        /// The id of the service item
        pub id: ServiceId,

        /// The info of the service item
        #[json(nested)]
        pub data: ServiceAccountData,
    }

    /// Represents the service account data.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct ServiceAccountData {
        /// The service account state
        #[json(nested)]
        pub service: ServiceInfo,

        /// (a_p) The preimages
        #[serde(default)]
        #[json(nested)]
        pub preimages: Vec<ServicePreimage>,

        /// The storage
        #[serde(default)]
        #[json(nested)]
        pub storage: Vec<ServiceStorage>,
    }

    impl From<&ServiceAccount> for ServiceAccountData {
        fn from(account: &ServiceAccount) -> Self {
            ServiceAccountData {
                service: account.state(),
                preimages: account
                    .preimage
                    .iter()
                    .map(|(k, v)| ServicePreimage {
                        hash: k.hash(),
                        // TODO: find a better solution for doing this.
                        blob: v.to_vec(),
                    })
                    .collect(),
                storage: account
                    .storage
                    .iter()
                    .map(|(k, v)| ServiceStorage {
                        key: k.storage(),
                        value: v.clone(),
                    })
                    .collect(),
            }
        }
    }

    impl From<ServiceItem> for ServiceAccount {
        fn from(item: ServiceItem) -> Self {
            let data = item.data;
            let mut lookup = BTreeMap::new();
            for preimage in &data.preimages {
                lookup.insert(
                    AccountInnerKey::Lookup(item.id, preimage.hash, preimage.blob.len() as u32),
                    Default::default(),
                );
            }

            ServiceAccount {
                index: item.id,
                storage: data
                    .storage
                    .into_iter()
                    .map(|s| (AccountInnerKey::Storage(item.id, s.key), s.value))
                    .collect(),
                preimage: data
                    .preimages
                    .into_iter()
                    .map(|p| (AccountInnerKey::Preimage(item.id, p.hash), p.blob))
                    .collect(),
                lookup,
                info: data.service,
            }
        }
    }

    /// Represents a service preimage.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct ServicePreimage {
        /// The hash of the preimage
        #[json(hex)]
        pub hash: OpaqueHash,

        /// The blob of the preimage
        #[json(hex)]
        pub blob: Vec<u8>,
    }

    /// Represents a service storage.
    #[derive(Debug, Clone, Serialize, Deserialize, Json, PartialEq, Eq)]
    pub struct ServiceStorage {
        /// The key of the storage
        #[json(hex)]
        pub key: Vec<u8>,

        /// The value of the storage
        #[json(hex)]
        pub value: Vec<u8>,
    }
}
