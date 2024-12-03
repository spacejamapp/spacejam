//! Spacejam's SAFRole prototype

pub mod error;
pub mod state;

pub use {
    error::Error,
    state::{OutputData, OutputDataJson, State, StateJson},
};
