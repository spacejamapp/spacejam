//! IO for binary formats

mod reader;
mod writer;

pub use {
    reader::{Reader, read},
    writer::Writer,
};
