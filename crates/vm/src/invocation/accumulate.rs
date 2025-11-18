//! PolkaVM environment

use crate::{invocation::Argument, Reason, Result};
use account::{Account, Accounts};
use score::{
    safrole::{ValidatorData, ValidatorsData},
    service::Privileges,
    vm::{AccumulateItem, DeferredTransfer},
    EntropyBuffer, Gas, OpaqueHash, ServiceId, TimeSlot,
};
use serde::{Deserialize, Serialize};

/// Data used in accumulate related host calls
pub struct Accumulate<R: Accounts> {
    /// The regular dimension
    pub x: AccumulateContext<R>,

    /// The exceptional dimension
    pub y: AccumulateContext<R>,

    /// The read-only state of the accumulation
    pub state: AccumulateState<R>,

    /// The timeslot
    pub timeslot: TimeSlot,

    /// (η′0) The entropy
    pub entropy: [u8; 32],

    /// (i) The accumulate items
    pub items: Vec<AccumulateItem>,
}

impl<R: Accounts> Accumulate<R> {
    /// Get the account
    pub fn account(&mut self) -> Result<&mut (impl Account + '_)> {
        self.x
            .context
            .accounts
            .get(self.x.service)
            .ok_or(Reason::Panic("Could not find account".into()))
    }
}

impl<R: Accounts> Argument for Accumulate<R> {
    const SUPPORTED_CALLS: &[u32] = &[
        0, 1, 2, 3, 4, 5, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 100,
    ];

    const INITIAL_PC: u64 = 5;

    fn account(&mut self, id: u64) -> anyhow::Result<&mut impl Account> {
        self.x
            .context
            .accounts
            .get(id as u32)
            .ok_or(anyhow::anyhow!("Could not find account {id}"))
    }

    fn check(&mut self, index: ServiceId) -> ServiceId {
        self.x.context.accounts.check(index)
    }

    fn checkpoint(&mut self) {
        self.y = self.x.clone();
    }

    fn entropy(&self) -> OpaqueHash {
        self.entropy
    }

    fn index(&self) -> ServiceId {
        self.x.index
    }

    fn items(&self) -> &[AccumulateItem] {
        &self.items
    }

    fn output(&mut self, hash: OpaqueHash) {
        self.x.output = Some(hash);
    }

    fn privileges(&self) -> Privileges {
        self.x.context.privileges.clone()
    }

    fn remove(&mut self, service: ServiceId) {
        self.x.context.accounts.remove(service);
    }

    fn service(&self) -> ServiceId {
        self.x.service
    }

    fn set_index(&mut self, index: ServiceId) {
        self.x.index = index;
    }

    fn set_privileges(&mut self, privileges: Privileges) {
        self.x.context.privileges = privileges;
    }

    fn set_validators(&mut self, validators: [ValidatorData; score::VALIDATORS_COUNT as usize]) {
        self.x.context.validators = validators;
    }

    fn this(&mut self) -> anyhow::Result<&mut impl Account> {
        self.x
            .context
            .accounts
            .get(self.x.service)
            .ok_or(anyhow::anyhow!("Could not find account {}", self.x.service))
    }

    fn timeslot(&self) -> TimeSlot {
        self.timeslot
    }

    fn transfer(&mut self, transfer: DeferredTransfer) {
        self.x.transfer.push(transfer);
    }

    fn upsert(&mut self, id: ServiceId, account: impl Account) {
        self.x.context.accounts.upsert(id, account);
    }
}

/// Context for the accumulate host calls
#[derive(Clone)]
pub struct AccumulateContext<R: Accounts> {
    /// (s) The service id
    pub service: ServiceId,

    /// (e) the accumulate state
    pub context: AccumulateState<R>,

    /// (i) empty index for a new account
    pub index: ServiceId,

    /// (t) The deferred transfer
    pub transfer: Vec<DeferredTransfer>,

    /// (y) The output hash of the accumulation
    pub output: Option<OpaqueHash>,
}

impl<R: Accounts> AccumulateContext<R> {
    /// Create a new accumulate context
    pub fn new(mut context: AccumulateState<R>, service: ServiceId, timeslot: TimeSlot) -> Self {
        Self {
            service,
            index: context.index(service, timeslot),
            context,
            transfer: Vec::new(),
            output: None,
        }
    }

    /// Get the account for the accumulation
    pub fn account(&mut self) -> Option<&mut (impl Account + '_)> {
        self.context.accounts.get(self.service)
    }

    /// Convert the accumulate context to an accumulate
    pub fn accumulate(self, timeslot: TimeSlot, items: Vec<AccumulateItem>) -> Accumulate<R> {
        let entropy = self.context.entropy[0];
        Accumulate {
            state: self.context.clone(),
            y: self.clone(),
            x: self,
            timeslot,
            entropy,
            items,
        }
    }

    /// Convert the accumulate context to an accumulate result
    pub fn to_result(self, gas: Gas, reason: Reason) -> Accumulated<R> {
        Accumulated {
            context: self.context,
            transfers: self.transfer,
            hash: self.output,
            gas,
            reason,
        }
    }
}

/// The state context used in pvm accumulation
#[derive(Clone)]
pub struct AccumulateState<R> {
    /// d (δ) The accounts
    pub accounts: R,

    /// i (ι) The upcoming validators
    pub validators: ValidatorsData,

    /// p (φ) The authorization queue
    pub authorization: [Vec<OpaqueHash>; score::CORES_COUNT],

    /// a (χ) The privileged service indices
    pub privileges: Privileges,

    /// (η) The entropy
    pub entropy: EntropyBuffer,

    /// (τ) The timeslot for the current accumulation
    pub timeslot: TimeSlot,
}

impl<R: Accounts> AccumulateState<R> {
    /// (I) Generate a new index from provided environment (B.10)
    pub fn index(&mut self, service: ServiceId, timeslot: TimeSlot) -> ServiceId {
        let encoded = codec::encode(&IndexSalt {
            service,
            entropy: self.entropy[0],
            timeslot,
        })
        .expect("failed to encode");
        let hash = crypto::blake2b(&encoded);
        let base = u32::from_le_bytes([hash[0], hash[1], hash[2], hash[3]]);
        let index = (base % score::CHECK_SALT) + score::MINIMUM_SERVICE_ID;
        self.accounts.check(index)
    }
}

/// The accumulate result of (ΨA)
pub struct Accumulated<R: Accounts> {
    /// (o) The state context
    pub context: AccumulateState<R>,

    /// (t) The timeslot for the current accumulation
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The output hash of the accumulation
    pub hash: Option<OpaqueHash>,

    /// (u) The gas used
    pub gas: Gas,

    /// (_e) The reason for the accumulation
    pub reason: Reason,
}

impl<R: Accounts> Accumulated<R> {
    /// Create a new accumulate result
    pub fn new(context: AccumulateState<R>) -> Self {
        Self {
            context,
            transfers: Vec::new(),
            hash: None,
            gas: 0,
            reason: Reason::Continue,
        }
    }
}

/// The salt for the index function
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct IndexSalt {
    /// (N_s)  the service id of the caller
    #[serde(with = "codec::compact")]
    pub service: u32,

    /// (η) The entropy
    pub entropy: [u8; 32],

    /// (N_t)  timeslot for the current accumulation
    #[serde(with = "codec::compact")]
    pub timeslot: u32,
}
