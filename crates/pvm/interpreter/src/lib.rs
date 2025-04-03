//! PVM is a virtual machine for the PVM assembly language.
//!
//! # TODOs:
//!
//! - error handling for index out of bounds.
//! - embed execution result in step outputs.
//! - calculate gas from step outputs.

mod interp;
pub mod memory;
mod result;
mod pvmi;

pub use pvm_parser::Register;
pub use {
    interp::Interpreter,
    memory::{Access, Memory, Page, PAGE_SIZE},
    result::{Error, Result},
};
