use crate::{block::Header, state::State};
use std::sync::Arc;

/// Read-only context for validation
#[derive(Debug, Clone)]
pub struct Context {
    /// Block Header
    pub header: Arc<Header>,
    /// Safrole
    pub state: Arc<State>,
}

/// An interface for patch
pub trait Patch<T> {
    /// Patch to the state
    fn patch(self, target: &mut T);
}
