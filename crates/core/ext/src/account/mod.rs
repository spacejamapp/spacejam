//! Account extensions

pub use {account::Account, registry::Accounts};

#[allow(clippy::module_inception)]
mod account;
mod registry;
