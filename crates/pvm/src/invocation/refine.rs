//! Primitives for the refine invocation

use crate::{Argument, Executed};
use score::{Account, Accounts, ServiceId};

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

impl<R: Accounts> Argument for Refine<R> {
    const SUPPORTED_CALLS: &[u32] = &[14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26];

    fn account(&mut self, id: u64) -> anyhow::Result<&mut impl Account> {
        self.accounts
            .get(id as u32)
            .ok_or(anyhow::anyhow!("Could not find account {id}"))
    }

    fn this(&mut self) -> anyhow::Result<&mut impl Account> {
        self.accounts
            .get(self.service)
            .ok_or(anyhow::anyhow!("Could not find account {}", self.service))
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
