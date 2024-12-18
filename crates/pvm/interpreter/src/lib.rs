//! PVM is a virtual machine for the PVM assembly language.
//!
//! # TODOs:
//!
//! - error handling for index out of bounds.
//! - embed execution result in step outputs.
//! - calculate gas from step outputs.

mod interp;
pub mod memory;
mod status;

pub use {interp::Interpreter, memory::Memory, status::Status};
