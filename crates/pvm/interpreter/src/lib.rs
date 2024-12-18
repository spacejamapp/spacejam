//! PVM is a virtual machine for the PVM assembly language.

mod interp;
mod status;

pub use {interp::Interpreter, status::Status};
