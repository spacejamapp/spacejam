//! JAMCodec based on the parity scale codec

mod de;
mod error;
mod json;
mod ser;

pub use codec_derive::Json;
pub use {
    de::Deserializer,
    error::{Error, Result},
    json::Json,
    ser::Serializer,
};
