//! JSON utilities
//!
//! Now using hex as the default encoding.
use anyhow::Result;
pub use result::ResultJson;
use serde::{Serialize, de::DeserializeOwned};
pub use spacejson_derive::Json;

mod array;
mod bytes;
mod map;
mod option;
mod primitive;
mod result;
mod tuple;

/// A trait for types that can be encoded and decoded to and from JSON.
pub trait Json<Target: Serialize + DeserializeOwned>: Sized + std::fmt::Debug {
    /// Converts the value to its JSON representation.
    fn to_json(self) -> Target;

    /// Converts the value from its JSON representation.
    fn from_json(json: Target) -> Result<Self>;
}
