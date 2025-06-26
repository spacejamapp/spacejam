//! The archive storage per block.

use crate::storage::{sync::SYNC, Commit, KVStorage};
use anyhow::Result;
use score::OpaqueHash;

/// The prefix of the archive storage.
pub const ARCHIVE: &[u8] = b"archive";

/// The archive storage interface.
pub trait ArchiveStorage: KVStorage {
    /// Get the archive storage for a given block.
    ///
    /// TODO: introduce a better solution for archving storage per block
    fn archive(&self, block: OpaqueHash) -> Result<()> {
        let mut iter = self.iter()?;
        let mut commit = Commit::default();
        while let Some(Ok((key, value))) = iter.next() {
            if key.starts_with(ARCHIVE) || key.starts_with(SYNC) || key.len() != 31 {
                continue;
            }

            let mut skey = [0; 31];
            skey.copy_from_slice(key.as_ref());
            commit.set([ARCHIVE, block.as_ref(), &skey].concat(), value);
            if commit.len() > 20 {
                self.commit(commit.clone())?;
                commit = Commit::default();
            }
        }
        Ok(())
    }

    /// Finalize the archive storage for a given block.
    fn finalize(&self, block: OpaqueHash) -> Result<()> {
        let prefix = [ARCHIVE, block.as_ref()].concat();
        let mut iter = self.prefix_iter(prefix)?;
        while let Some(Ok((key, value))) = iter.next() {
            self.set(key[39..].to_vec(), value)?;
        }

        Ok(())
    }
}

impl<T: KVStorage> ArchiveStorage for T {}
