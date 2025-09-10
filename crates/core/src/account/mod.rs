//! JAM account abstraction

pub use {
    account::{Account, AccountInnerKey},
    registry::Accounts,
};

#[allow(clippy::module_inception)]
mod account;
mod registry;
