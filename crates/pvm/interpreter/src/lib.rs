//! PVM is a virtual machine for the PVM assembly language.
//!
//! # TODOs:
//!
//! - error handling for index out of bounds.
//! - embed execution result in step outputs.
//! - calculate gas from step outputs.

mod interp;
mod pvmi;
mod result;

pub use parser::{Memory, Register, PAGE_SIZE};
pub use {
    interp::Interpreter,
    result::{Error, Result},
};
