//! Host functions

use crate::{result::Reason, State};

/// The host function type
pub type HostCall<X, Memory> = fn(u32, X) -> (Reason, State<Memory>, X);
