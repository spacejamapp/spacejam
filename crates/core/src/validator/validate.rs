use crate::state::State;
use anyhow::Result;

/// A patch to the state
///
/// The state is patched by applying a series of patches. each patch could be
/// a single operation or a series of operations.
pub trait Patch {
    /// Patch to the state
    fn apply(self, target: &mut State) -> Result<()>;
}
