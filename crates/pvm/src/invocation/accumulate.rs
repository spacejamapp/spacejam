//! PolkaVM environment

use crate::{
    invocation::{Argument, General},
    Reason, Result,
};
use score::{
    vm::{AccumulateState, DeferredTransfer, Operand},
    Account, Accounts, Gas, OpaqueHash, ServiceId, TimeSlot,
};

/// Data used in accumulate related host calls
pub struct Accumulate<R: Accounts> {
    /// The regular dimension
    pub x: AccumulateContext<R>,

    /// The exceptional dimension
    pub y: AccumulateContext<R>,

    /// The timeslot
    pub timeslot: TimeSlot,

    /// (η′0) The entropy
    pub entropy: [u8; 32],

    /// (o) The operands
    pub operands: Vec<Operand>,
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

impl<R: Accounts> Argument<R> for Accumulate<R> {
    fn as_general(&self) -> crate::Result<General<R>> {
        Ok(General::new(
            self.x.service,
            self.x.context.accounts.clone(),
            self.operands.clone(),
            self.entropy,
        ))
    }

    // FIXME: find a better way to update the account
    fn update_general(&mut self, mut general: General<R>) -> crate::Result<()> {
        let index = general.index;
        let Some(account) = general.account() else {
            crate::bail!("Account {} not found in context", general.index);
        };

        // Update the account metadata - set the update field to current timeslot
        let mut updated_account = account.clone();
        updated_account.set_update(self.timeslot);
        self.x.context.accounts.upsert(index, updated_account);
        Ok(())
    }

    fn as_accumulate_mut(&mut self) -> crate::Result<&mut Accumulate<R>> {
        Ok(self)
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
    pub fn accumulate(self, timeslot: TimeSlot, operands: Vec<Operand>) -> Accumulate<R> {
        let entropy = self.context.entropy[0];
        Accumulate {
            y: self.clone(),
            x: self,
            timeslot,
            entropy,
            operands,
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
