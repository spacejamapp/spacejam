//! JAMCodec based on the parity scale codec

mod error;
mod json;
mod ser;

pub use codec_derive::Json;
pub use error::Error;
pub use json::Json;
