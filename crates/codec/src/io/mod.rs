//! IO for binary formats

mod reader;
mod writer;

pub use {
    reader::{read_cow, Reader},
    writer::Writer,
};
