#[cfg(feature = "std")]
pub use std::{borrow::Cow, format, string::String, vec, vec::Vec};

#[cfg(not(feature = "std"))]
pub use alloc::{borrow::Cow, format, string::String, vec, vec::Vec};
