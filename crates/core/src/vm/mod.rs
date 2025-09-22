//! Virtual machine interfaces

pub use {
    accumulate::{
        AccumulateParams, AccumulateState, Accumulated, Accumulation, CommitmentMap, IndexSalt,
        Operand,
    },
    refine::RefineParams,
    transfer::DeferredTransfer,
};

mod accumulate;
mod refine;
mod transfer;
