//! Primitives for the refine invocation

use crate::Executed;

/// Reine host call arguments
pub struct Refine {}

/// The result of refine invocation (ΨR)
pub struct Refined {
    /// The executed result
    pub executed: Executed,

    /// The imports
    pub segments: Vec<[u8; score::SEGMENT_SIZE as usize]>,
}

impl Refined {
    /// Create a new refined result
    pub fn new(executed: Executed, segments: Vec<[u8; score::SEGMENT_SIZE as usize]>) -> Self {
        Self { executed, segments }
    }
}
