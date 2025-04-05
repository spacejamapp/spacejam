//! PVM is a virtual machine for the PVM assembly language.
//!
//! # TODOs:
//!
//! - error handling for index out of bounds.
//! - embed execution result in step outputs.
//! - calculate gas from step outputs.

mod interp;
pub mod memory;
mod pvmi;
mod result;

pub use parser::Register;
pub use {
    interp::Interpreter,
    memory::{Access, Memory, Page, PAGE_SIZE},
    result::{Error, Result},
};
