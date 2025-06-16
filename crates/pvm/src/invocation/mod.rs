//! PVM invocation interface

use score::Accounts;
pub use {
    accumulate::Accumulate,
    api::Invocation,
    general::General,
    refine::Refine,
    state::{Executed, Received, State, Stepped},
};

pub mod accumulate;
mod api;
mod general;
pub mod refine;
mod state;
pub mod transfer;

/// Dynamic arguments for host calls
pub trait Argument<R: Accounts> {
    /// returns some if the input data is general
    fn as_general(&self) -> crate::Result<General<R>> {
        crate::bail!("not a general")
    }

    /// update the general argument
    fn update_general(&mut self, _general: General<R>) -> crate::Result<()> {
        crate::bail!("not a general")
    }

    /// returns some if the input data is accumulate
    fn as_accumulate_mut(&mut self) -> crate::Result<&mut Accumulate<R>> {
        crate::bail!("not an accumulate")
    }

    /// returns some if the input data is refine
    fn as_refine_mut(&mut self) -> crate::Result<&mut Refine> {
        crate::bail!("not a refine")
    }
}
