//! Ancestry of the best head

use score::{block::Head, OpaqueHash};

/// Ancestry of the best head
#[derive(Clone)]
pub struct Ancestry {
    /// The selected best head.
    pub best: Head,

    /// The ancestors of the best head.
    ///
    /// [best -> ancestors -> finalized]
    pub ancestors: Vec<OpaqueHash>,

    /// The finalized head.
    pub finalized: Head,
}

impl Ancestry {
    /// Update the ancestry with the existing head.
    pub fn advance(&mut self, head: &Head) -> anyhow::Result<()> {
        if head.hash == self.finalized.hash {
            return Ok(());
        }

        if !self.ancestors.contains(&head.hash) {
            anyhow::bail!(
                "current best head#{} is not in the ancestors, FIXME: forked chain",
                head.slot
            );
        }

        self.ancestors = self
            .ancestors
            .iter()
            .cloned()
            .take_while(|h| *h != head.hash)
            .collect();

        self.finalized = head.clone();
        Ok(())
    }
}
