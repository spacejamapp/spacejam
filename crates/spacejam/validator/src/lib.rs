//! Extrinsic extensions in SpaceJam

use score::{block::header::Header, state::State};
use std::sync::Arc;
pub use {
    extrinsic::{ExtrinsicInMem, ExtrinsicInPool},
    result::{Error, Result, ValidationError},
};

mod extrinsic;
mod result;
pub mod validate;

/// Read-only context for validation
#[derive(Debug, Clone)]
pub struct Context {
    /// Block Header
    pub header: Arc<Header>,
    /// Safrole
    pub safrole: Arc<State>,
}

/// An interface for patch
pub trait Patch<T> {
    /// Patch to the state
    fn patch(self, target: &mut T);
}
