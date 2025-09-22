//! Testing library for the PVM

pub use score::service::ServiceAccount as Account;
use score::service::WorkItem;
pub use {account::AccountBuilder, auth::Auth, chain::Chain, extrinsic::Extrinsic};

mod account;
mod auth;
mod builder;
mod chain;
mod exec;
mod extrinsic;
pub mod util;

/// JAM environment
#[derive(Default)]
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
