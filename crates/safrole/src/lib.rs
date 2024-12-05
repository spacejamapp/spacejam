//! Spacejam's SAFRole prototype

pub mod error;
pub mod state;

pub use {
    error::{Error, Result},
    state::{Markers, MarkersJson, State, StateJson},
};
