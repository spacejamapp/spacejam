//! Virtual machine interfaces

pub use {
    accumulate::{
        AccumulateParams, AccumulateState, Accumulated, Accumulation, CommitmentMap, Operand,
    },
    transfer::DeferredTransfer,
};

mod accumulate;
mod transfer;
