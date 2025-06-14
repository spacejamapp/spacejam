//! Virtual machine interfaces

pub use {
    accumulate::{
        AccumulateParams, Accumulated, Accumulation, CommitmentMap, Operand, StateContext,
    },
    transfer::DeferredTransfer,
};

mod accumulate;
mod transfer;
