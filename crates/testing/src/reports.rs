//! Report testing types

use runtime::tx::{
    self,
    guarantee::error::{Error, Result},
};
use score::{
    Block, Ed25519Public, OpaqueHash, TimeSlot, extrinsic::GuaranteesExtrinsic,
    service::ReportedWorkPackage,
};
use serde::{Deserialize, Serialize};
pub use types::*;

include!(concat!(env!("OUT_DIR"), "/reports.rs"));

/// Run the reports test
pub fn run(test: &specjam::Test) -> anyhow::Result<()> {
    let (input, pre, output, post) =
        codec::decode::<(Input, RawState, Result<Output>, RawState)>(test.input.expect_bin()?)?;
    let pre_state: State = pre.into();
    let post_state: State = post.into();

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
                let (reported, reporters, triples) =
                    tx::guarantee::report(&state, input.slot, &state.accounts, &input.guarantees)?;
                crypto::ed25519::SigItem::batch_verify(&triples)
                    .map_err(|_| Error::BadSignature)?;
                Ok((reported, reporters, assignments))
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestInput {
    pub input: Input,
    pub pre_state: State,
}

/// Test output.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TestOutput {
    pub output: Result<Output>,
    pub post_state: State,
}

/// Input of the reporting module.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Input {
    pub guarantees: GuaranteesExtrinsic,
    pub slot: TimeSlot,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Output {
    pub reported: Vec<ReportedWorkPackage>,
    pub reporters: Vec<Ed25519Public>,
}

mod types {
    use score::{
        Ed25519Public, EntropyBuffer, OpaqueHash, ServiceId,
        block::History,
        safrole::ValidatorsData,
        service::{AvailabilityAssignments, ServiceAccount, ServiceInfo},
    };
    use serde::{Deserialize, Serialize};
    use std::collections::BTreeMap;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct State {
        /// (ρ‡) Intermediate pending reports after that any work report judged as
        /// uncertain or invalid has been removed from it (ϱ†), and the availability
        /// assurances are processed. Mutated to ϱ'.
        pub avail_assignments: AvailabilityAssignments,

        /// (κ') Posterior active validators.
        pub curr_validators: ValidatorsData,

        /// (λ') Posterior previous validators.
        pub prev_validators: ValidatorsData,

        /// (η') Posterior entropy buffer.
        pub entropy: EntropyBuffer,

        /// (ψ'_o) Posterior offenders.
        pub offenders: Vec<Ed25519Public>,

        /// (β) Recent blocks.
        pub recent_blocks: History,

        /// (α') Authorization pools.
        pub auth_pools: score::AuthorizationPools,

        /// (δ) Encoded services dictionary. Refer to T(σ) in Appendix D.
        #[serde(alias = "accounts")]
        pub services: Vec<ServiceItem>,
    }

    /// The reports STF `State` raw layout (reports.asn): the eight modelled
    /// fields, then a minimal `(id, ServiceInfo)` accounts list, then the
    /// `cores-statistics` and `services-statistics` records.
    pub type RawState = (
        AvailabilityAssignments,
        ValidatorsData,
        ValidatorsData,
        EntropyBuffer,
        Vec<Ed25519Public>,
        History,
        score::AuthorizationPools,
        Vec<(ServiceId, ServiceInfo)>,
        score::statistic::CoreStats,
        Vec<(ServiceId, score::statistic::ServiceActivityRecord)>,
    );

    /// Build from the raw tuple, dropping the statistics records the reports
    /// test doesn't assert on.
    impl From<RawState> for State {
        fn from(w: RawState) -> Self {
            let (
                avail_assignments,
                curr_validators,
                prev_validators,
                entropy,
                offenders,
                recent_blocks,
                auth_pools,
                accounts,
                _cores,
                _services,
            ) = w;
            State {
                avail_assignments,
                curr_validators,
                prev_validators,
                entropy,
                offenders,
                recent_blocks,
                auth_pools,
                services: accounts
                    .into_iter()
                    .map(|(id, service)| ServiceItem {
                        id,
                        data: ServiceAccountData {
                            service,
                            storage: vec![],
                            preimages: vec![],
                            preimage_requests: vec![],
                        },
                    })
                    .collect(),
            }
        }
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
                            preimage_requests: vec![],
                            storage: vec![],
                        },
                    })
                    .collect(),
            }
        }
    }

    /// Represents a service item.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ServiceItem {
        /// The id of the service item
        pub id: ServiceId,

        /// The info of the service item
        pub data: ServiceAccountData,
    }

    /// Represents the service account data.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ServiceAccountData {
        /// The service account state
        pub service: ServiceInfo,

        /// The storage
        #[serde(default)]
        pub storage: Vec<ServiceStorage>,

        /// (a_p) The preimages
        #[serde(default)]
        #[serde(alias = "preimage_blobs")]
        pub preimages: Vec<ServicePreimage>,

        /// The preimage status
        #[serde(default)]
        #[serde(alias = "preimage_requests")]
        pub preimage_requests: Vec<ServicePreimageRequest>,
    }

    impl From<&ServiceAccount> for ServiceAccountData {
        fn from(account: &ServiceAccount) -> Self {
            ServiceAccountData {
                service: account.state(),
                preimages: account
                    .preimage
                    .iter()
                    .map(|(k, v)| ServicePreimage {
                        hash: *k,
                        // TODO: find a better solution for doing this.
                        blob: v.to_vec(),
                    })
                    .collect(),
                preimage_requests: account
                    .lookup
                    .iter()
                    .map(|(k, v)| ServicePreimageRequest {
                        key: ServicePreimageRequestKey {
                            hash: k.0,
                            length: k.1,
                        },
                        value: if v.is_empty() { vec![0] } else { v.to_vec() },
                    })
                    .collect(),
                storage: account
                    .storage
                    .iter()
                    .map(|(k, v)| ServiceStorage {
                        key: k.to_vec(),
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
                let mut slots = Default::default();
                if let Some(status) = data.preimage_requests.iter().find(|s| {
                    s.key.hash == preimage.hash && s.key.length == preimage.blob.len() as u32
                }) {
                    slots = status.value.clone();
                }
                lookup.insert((preimage.hash, preimage.blob.len() as u32), slots);
            }

            ServiceAccount {
                index: item.id,
                storage: data.storage.into_iter().map(|s| (s.key, s.value)).collect(),
                preimage: data
                    .preimages
                    .into_iter()
                    .map(|p| (p.hash, p.blob))
                    .collect(),
                lookup,
                info: data.service,
            }
        }
    }

    /// Represents a service preimage.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ServicePreimage {
        /// The hash of the preimage
        pub hash: OpaqueHash,

        /// The blob of the preimage
        pub blob: Vec<u8>,
    }

    /// Represents a service preimage.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ServicePreimageRequest {
        /// The key of the preimage
        pub key: ServicePreimageRequestKey,

        /// The status of the preimage
        pub value: Vec<u32>,
    }

    /// Represents a service preimage.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ServicePreimageRequestKey {
        /// The hash of the preimage
        pub hash: OpaqueHash,

        /// The length of the preimage
        pub length: u32,
    }

    /// Represents a service storage.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ServiceStorage {
        /// The key of the storage
        pub key: Vec<u8>,

        /// The value of the storage
        pub value: Vec<u8>,
    }
}
