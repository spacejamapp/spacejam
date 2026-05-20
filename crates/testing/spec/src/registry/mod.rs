//! test vector registry

use crate::{section::Trace, Scale, Section};
use anyhow::Result;
pub use entry::Entry;
use std::path::PathBuf;

mod entry;

/// The test vector registry
pub struct Registry {
    /// The root directory of the test vectors
    root: PathBuf,
    /// The default scale for scaled sections
    scale: Scale,
}

impl Registry {
    /// Create a new registry from the given jam-test-vectors directory
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self::with_scale(root, Scale::Tiny)
    }

    /// Create a new registry with the given scale
    pub fn with_scale(root: impl Into<PathBuf>, scale: Scale) -> Self {
        let root = root.into();
        if !root.exists() {
            panic!(
                "jam-test-vectors directory does not exist: {}",
                root.display()
            );
        }
        Self { root, scale }
    }

    /// Get an entry from the registry
    pub fn entry(&self, section: &str) -> Result<Entry> {
        let section = section.parse::<Section>()?;
        match section {
            Section::Codec => self.codec(self.scale),
            Section::Pvm => self.pvm(),
            Section::Shuffle => self.shuffle(),
            Section::Trie => self.trie(),
            Section::Erasure => self.erasure(self.scale),
            Section::Accumulate => self.accumulate(self.scale),
            Section::Assurances => self.assurances(self.scale),
            Section::Authorizations => self.authorizations(self.scale),
            Section::Disputes => self.disputes(self.scale),
            Section::History => self.history(self.scale),
            Section::Preimages => self.preimages(self.scale),
            Section::Reports => self.reports(self.scale),
            Section::Safrole => self.safrole(self.scale),
            Section::Statistics => self.statistics(self.scale),
            Section::Trace(trace) => self.trace(trace),
        }
    }

    /// Get the accumulate test vectors
    pub fn accumulate(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Accumulate, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the assurances test vectors
    pub fn assurances(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Assurances, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the authorizations test vectors
    pub fn authorizations(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Authorizations, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the codec test vectors
    pub fn codec(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Codec, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the disputes test vectors
    pub fn disputes(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Disputes, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the erasure test vectors
    pub fn erasure(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Erasure, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the history test vectors
    pub fn history(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::History, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the preimages test vectors
    pub fn preimages(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Preimages, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the pvm test vectors
    pub fn pvm(&self) -> Result<Entry> {
        let entry = Entry::new(Section::Pvm, None, &self.root)?;
        Ok(entry)
    }

    /// Get the reports test vectors
    pub fn reports(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Reports, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the safrole test vectors
    pub fn safrole(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Safrole, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the statistics test vectors
    pub fn statistics(&self, scale: Scale) -> Result<Entry> {
        let entry = Entry::new(Section::Statistics, Some(scale), &self.root)?;
        Ok(entry)
    }

    /// Get the shuffle test vectors
    pub fn shuffle(&self) -> Result<Entry> {
        let entry = Entry::new(Section::Shuffle, None, &self.root)?;
        Ok(entry)
    }

    /// Get the trace test vectors
    pub fn trace(&self, trace: Trace) -> Result<Entry> {
        let entry = Entry::new(Section::Trace(trace), None, &self.root)?;
        Ok(entry)
    }

    /// Get the trie test vectors
    pub fn trie(&self) -> Result<Entry> {
        let entry = Entry::new(Section::Trie, None, &self.root)?;
        Ok(entry)
    }
}
