//! Block processor

use crate::tx;
use score::State;

/// Block processor
pub struct Processor {
    /// the runtime state
    state: State,
}
