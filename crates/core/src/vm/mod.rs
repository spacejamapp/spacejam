//! Virtual machine interfaces

pub use {
    acc::{AccumulateResult, Accumulated, CommitmentMap, Operand},
    context::StateContext,
    transfer::DeferredTransfer,
};

mod acc;
mod context;
mod transfer;
