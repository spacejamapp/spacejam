//! Testing library for the PVM

use score::service::WorkItem;
pub use {auth::Auth, chain::Chain, extrinsic::Extrinsic};

mod auth;
mod builder;
mod chain;
mod extrinsic;

/// JAM environment
pub struct Jam {
    /// Chain environment
    chain: Chain,

    /// authorization token
    auth: Auth,

    /// work items
    items: Vec<WorkItem>,

    /// extrinsics
    _extrinsic: Vec<Extrinsic>,
}
