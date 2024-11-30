//! JSON utilities
//!
//! Now using hex as the default encoding.
use anyhow::Result;
use serde::{de::DeserializeOwned, Serialize};

/// A trait for types that can be encoded and decoded to and from JSON.
pub trait Json: Sized {
    type Target: Serialize + DeserializeOwned + From<Self> + TryInto<Self, Error = anyhow::Error>;

    /// Converts the value to its JSON representation.
    fn json(self) -> Self::Target {
        Self::Target::from(self)
    }

    /// Encodes the value to a JSON string.
    fn to_json(self) -> Result<String> {
        let target: Self::Target = self.into();
        serde_json::to_string_pretty(&target).map_err(Into::into)
    }

    /// Decodes the value from a JSON string.
    fn from_json(json: &str) -> Result<Self> {
        let target: Self::Target = serde_json::from_str(json)?;
        Self::Target::try_into(target).map_err(Into::into)
    }
}
