//! Spacejam work package builder
//! Work package computation

use runtime::Config;
use std::sync::Arc;
pub use {bundle::Bundle, context::Context, worker::Worker};

mod bundle;
mod context;
mod worker;

/// Builder for work package computation
pub struct Builder<C: Config> {
    /// The context of the builder
    pub context: Arc<Context<C>>,
}

impl<C: Config> Builder<C> {
    /// Create a new builder
    pub fn new(context: Arc<Context<C>>) -> Self {
        Self { context }
    }
}
