//! Cache for the compiled modules

use anyhow::Result;
use cranelift::codegen::incremental_cache::CacheKvStore;
use std::{borrow::Cow, fs, path::PathBuf, sync::OnceLock};

/// Cache directory for the compiled modules
pub static SPACEVM_CACHE_DIR: OnceLock<Option<PathBuf>> = OnceLock::new();

/// Artifact for the compiled modules
pub struct Artifact {
    /// The base directory for the artifact
    dir: PathBuf,
}

impl Artifact {
    /// Create new artifact
    pub fn new() -> Result<Self> {
        let dir = SPACEVM_CACHE_DIR.get_or_init(|| {
            let dir = dirs::data_dir()
                .unwrap_or_default()
                .join("spacejam")
                .join("spacevm");
            Some(dir)
        });

        let dir = dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("cache dir not found"))?
            .clone();

        fs::create_dir_all(dir.join("artifacts")).map_err(|e| {
            anyhow::anyhow!("failed to create artifact directory at {dir:?}: {e:?}")
        })?;
        Ok(Self { dir })
    }
}

impl CacheKvStore for Artifact {
    fn get(&self, key: &[u8]) -> Option<Cow<'_, [u8]>> {
        let key = hex::encode(key);
        let path = self.dir.join("artifacts").join(key);
        if !path.exists() {
            return None;
        }

        let serialized = fs::read(path).ok()?;
        Some(Cow::Owned(serialized))
    }

    fn insert(&mut self, key: &[u8], value: Vec<u8>) {
        let key = hex::encode(key);
        let path = self.dir.join("artifacts").join(key);
        if path.exists() {
            return;
        }

        let _ = fs::write(path, value);
    }
}
