//! Cache for the compiled modules

use anyhow::Result;
use cranelift_codegen::{incremental_cache::CacheKvStore, ir::Function};
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
                .join("jastime");
            Some(dir)
        });

        let dir = dir
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("cache dir not found"))?
            .clone();

        fs::create_dir_all(dir.join("artifacts"))?;
        fs::create_dir_all(dir.join("clif"))?;
        Ok(Self { dir })
    }

    /// Check if the cache hits
    pub fn hits(&self, key: [u8; 32]) -> bool {
        if let Some((_, confirmed)) = self.clif(key) {
            return confirmed;
        }

        false
    }

    /// Get the path to the CLIF artifacts
    pub fn clif(&self, key: [u8; 32]) -> Option<(Function, bool)> {
        let path = self.dir.join("clif").join(hex::encode(key));
        if !path.exists() {
            return None;
        }

        let serialized = fs::read(path).ok()?;
        postcard::from_bytes(&serialized).ok()
    }

    /// Put the CLIF artifact
    pub fn put(&self, key: [u8; 32], function: &Function, confirmed: bool) -> Result<()> {
        let key = hex::encode(key);
        let path = self.dir.join("clif").join(&key);
        let serialized = postcard::to_allocvec(&(function, confirmed))?;
        fs::write(path, serialized)?;
        Ok(())
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
