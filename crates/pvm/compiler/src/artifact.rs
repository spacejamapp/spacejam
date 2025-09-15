//! Cache for the compiled modules

use anyhow::Result;
use cranelift::codegen::incremental_cache::CacheKvStore;
use std::{
    borrow::Cow,
    fs,
    path::PathBuf,
    sync::{LazyLock, Mutex},
};

/// Cache directory for the compiled modules
pub static SPACEJAM_CACHE_DIR: LazyLock<Mutex<PathBuf>> =
    LazyLock::new(|| Mutex::new(dirs::data_dir().unwrap_or_default().join("spacejam")));

/// Artifact for the compiled modules
pub struct Artifact;

impl Artifact {
    /// Save the artifact to the cache
    pub fn save(folder: &str, fname: &str, value: &[u8]) -> Result<()> {
        let base = SPACEJAM_CACHE_DIR
            .try_lock()
            .map_err(|e| anyhow::anyhow!("failed to lock cache directory: {e:?}"))?
            .clone();

        let parent = base.join(folder);
        if !parent.exists() {
            fs::create_dir_all(&parent).map_err(|e| {
                anyhow::anyhow!("failed to create cache directory at {parent:?}: {e:?}")
            })?;
        }

        let target = parent.join(fname);
        fs::write(&target, value)
            .map_err(|e| anyhow::anyhow!("failed to save artifact to {target:?}: {e:?}"))
    }

    /// Load the artifact from the cache
    pub fn load(folder: &str, fname: &str) -> Option<Vec<u8>> {
        let base = SPACEJAM_CACHE_DIR.lock().ok()?.clone();
        let parent = base.join(folder);
        if !parent.exists() {
            return None;
        }

        let target = parent.join(fname);
        fs::read(&target).ok()
    }
}

impl CacheKvStore for Artifact {
    fn get(&self, key: &[u8]) -> Option<Cow<'_, [u8]>> {
        let key = hex::encode(key);
        Self::load("artifacts", &key).map(Cow::Owned)
    }

    fn insert(&mut self, key: &[u8], value: Vec<u8>) {
        let key = hex::encode(key);
        Self::save("artifacts", &key, &value).ok();
    }
}
