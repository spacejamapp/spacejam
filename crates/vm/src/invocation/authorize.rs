//! IsAuthorized invocation context

use crate::Argument;
use score::service::ServiceAccount;
use score_ext::Account;

/// IsAuthorized invocation context
#[derive(Debug, Clone)]
pub struct IsAuthorized {
    /// The work package being authorized
    pub package: score::service::WorkPackage,
    /// The core index
    pub core_idx: u16,
}

impl IsAuthorized {
    /// Create a new IsAuthorized context
    pub fn new(package: score::service::WorkPackage, core_idx: u16) -> Self {
        Self { package, core_idx }
    }
}

impl Argument for IsAuthorized {
    const SUPPORTED_CALLS: &[u32] = &[14, 15, 16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26];

    const INITIAL_PC: u64 = 0;

    fn account(&mut self, _id: u64) -> anyhow::Result<&mut impl Account> {
        anyhow::Result::<&mut ServiceAccount>::Err(anyhow::anyhow!("not implemented"))
    }

    fn this(&mut self) -> anyhow::Result<&mut impl Account> {
        anyhow::Result::<&mut ServiceAccount>::Err(anyhow::anyhow!("not implemented"))
    }
}
