//! Primitives for the refine invocation

use crate::{invocation::General, Argument, Executed};
use score::{Accounts, ServiceId};

/// Refine host call arguments
pub struct Refine<R: Accounts> {
    /// (δ) accounts for historical lookup
    pub accounts: R,

    /// (s) service id
    pub service: ServiceId,

    /// (c) core index
    pub core: u16,

    /// (r) authorizer output
    pub auth_output: Vec<u8>,

    /// (ī) all work items' import segments
    pub all_imports: Vec<Vec<[u8; score::SEGMENT_SIZE as usize]>>,

    /// (ς) export segment offset
    pub export_offset: u16,

    /// (e) exported segments (to be filled during execution)
    pub exports: Vec<[u8; score::SEGMENT_SIZE as usize]>,
}

impl<R: Accounts> Argument<R> for Refine<R> {
    fn as_general(&self) -> crate::Result<General<R>> {
        Ok(super::General::new(
            self.service,
            self.accounts.clone(),
            Vec::new(),
        ))
    }

    // FIXME: find a better way to update the account
    fn update_general(&mut self, mut general: General<R>) -> crate::Result<()> {
        let index = general.index;
        let Some(account) = general.account() else {
            crate::bail!("Account {} not found in context", general.index);
        };

        self.accounts.upsert(index, account.clone());
        Ok(())
    }

    fn as_refine_mut(&mut self) -> crate::Result<&mut Refine<R>> {
        Ok(self)
    }
}

/// The result of refine invocation (ΨR)
pub struct Refined {
    /// The executed result
    pub executed: Executed,

    /// The imports
    pub segments: Vec<[u8; score::SEGMENT_SIZE as usize]>,
}

impl Refined {
    /// Create a new refined result
    pub fn new(executed: Executed, segments: Vec<[u8; score::SEGMENT_SIZE as usize]>) -> Self {
        Self { executed, segments }
    }
}
