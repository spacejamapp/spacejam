//! Testing library for the PVM

use score::service::WorkItem;
pub use score::{Account as AccountExt, service::ServiceAccount as Account};
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
