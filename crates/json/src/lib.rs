//! JAMCodec based on the parity scale codec

mod decode;
mod encode;
mod json;

pub use json_derive::Json;
pub use {encode::Encode, json::Json};
