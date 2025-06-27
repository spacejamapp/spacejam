//! Ancestry of the best head

use score::{block::Head, OpaqueHash};

/// Ancestry of the best head
pub struct Ancestry {
    /// The selected best head.
    pub best: Head,

    /// The ancestors of the best head.
    pub ancestors: Vec<OpaqueHash>,

    /// The finalized head.
    pub finalized: Head,
}
