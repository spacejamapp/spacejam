//! PVM is a virtual machine for the PVM assembly language.

mod interp;
pub mod mem;
mod status;

pub use {interp::Interpreter, mem::Memory, status::Status};
