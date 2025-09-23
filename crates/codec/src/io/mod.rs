//! IO for binary formats

mod reader;
mod writer;

pub use {
    reader::{read, Reader},
    writer::Writer,
};
