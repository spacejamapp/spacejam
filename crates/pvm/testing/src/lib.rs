//! Testing library for the PVM

use score::service::WorkItem;
pub use score::{service::ServiceAccount as Account, Account as AccountExt};
use tracing_subscriber::EnvFilter;
pub use {account::AccountBuilder, auth::Auth, chain::Chain, extrinsic::Extrinsic};

mod account;
mod auth;
mod builder;
mod chain;
mod exec;
mod extrinsic;

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

/// Initialize the logger
pub fn init_logger() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .try_init();
}
