//! PVM is a virtual machine for the PVM assembly language.

mod interp;
mod result;
mod status;

pub use {interp::Interpreter, result::Result, status::Status};
