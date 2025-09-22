//! Primitives for the transfer invocation

use score::{service::ServiceAccount, Gas};

/// The result of transfer invocation (ΨT)
#[derive(Default)]
pub struct Transferred {
    /// The account
    pub account: ServiceAccount,

    /// The gas used
    pub gas: Gas,
}
