//! Spacejam's SAFRole prototype

pub mod error;
pub mod state;

pub use {
    error::{Error, ErrorJson},
    state::{OutputData, OutputDataJson, State, StateJson},
};
