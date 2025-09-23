//! JSON utilities
//!
//! Now using hex as the default encoding.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(feature = "std")]
pub(crate) use std::{collections, format, string::String, vec::Vec};

#[cfg(not(feature = "std"))]
pub(crate) use alloc::{collections, format, string::String, vec::Vec};

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
pub trait Json<Target: Serialize + DeserializeOwned>: Sized + core::fmt::Debug {
    /// Converts the value to its JSON representation.
    fn to_json(self) -> Target;

    /// Converts the value from its JSON representation.
    fn from_json(json: Target) -> Result<Self>;
}
