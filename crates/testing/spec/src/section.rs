//! The section of the test vectors

use std::{fmt::Display, str::FromStr};

/// A section of the test vectors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// The accumulate section
    Accumulate,
    /// The assurances section
    Assurances,
    /// The authorizations section
    Authorizations,
    /// The codec section
    Codec,
    /// The disputes section
    Disputes,
    /// The erasure coding section
    Erasure,
    /// The history section
    History,
    /// The preimages section
    Preimages,
    /// The pvm section
    Pvm,
    /// The reports section
    Reports,
    /// The safrole section
    Safrole,
    /// The statistics section
    Statistics,
    /// The shuffle section
    Shuffle,
    /// State trace section
    Trace(Trace),
    /// The trie section
    Trie,
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "codec" => Ok(Section::Codec),
            "pvm/programs" => Ok(Section::Pvm),
            "shuffle" => Ok(Section::Shuffle),
            "trie" => Ok(Section::Trie),
            "erasure" | "erasure-coding" => Ok(Section::Erasure),
            "accumulate" | "stf/accumulate" => Ok(Section::Accumulate),
            "assurances" | "stf/assurances" => Ok(Section::Assurances),
            "safrole" | "stf/safrole" => Ok(Section::Safrole),
            "statistics" | "stf/statistics" => Ok(Section::Statistics),
            "authorizations" | "stf/authorizations" => Ok(Section::Authorizations),
            "disputes" | "stf/disputes" => Ok(Section::Disputes),
            "history" | "stf/history" => Ok(Section::History),
            "preimages" | "stf/preimages" => Ok(Section::Preimages),
            "reports" | "stf/reports" => Ok(Section::Reports),
            "fallback" | "traces/fallback" => Ok(Section::Trace(Trace::Fallback)),
            "fuzzy" | "traces/fuzzy" => Ok(Section::Trace(Trace::Fuzzy)),
            "traces/safrole" => Ok(Section::Trace(Trace::Safrole)),
            "traces/preimages" => Ok(Section::Trace(Trace::Preimages)),
            "traces/preimages_light" => Ok(Section::Trace(Trace::PreimagesLight)),
            "traces/storage" => Ok(Section::Trace(Trace::Storage)),
            "traces/storage_light" => Ok(Section::Trace(Trace::StorageLight)),
            _ => Err(anyhow::anyhow!("Invalid section {s}")),
        }
    }
}

impl AsRef<str> for Section {
    fn as_ref(&self) -> &str {
        match self {
            Section::Codec => "codec",
            Section::Pvm => "pvm/programs",
            Section::Shuffle => "shuffle",
            Section::Trie => "trie",
            Section::Erasure => "erasure",
            Section::Accumulate => "stf/accumulate",
            Section::Assurances => "stf/assurances",
            Section::Safrole => "stf/safrole",
            Section::Statistics => "stf/statistics",
            Section::Authorizations => "stf/authorizations",
            Section::Disputes => "stf/disputes",
            Section::History => "stf/history",
            Section::Preimages => "stf/preimages",
            Section::Reports => "stf/reports",
            Section::Trace(trace) => match trace {
                Trace::Fallback => "traces/fallback",
                Trace::Fuzzy => "traces/fuzzy",
                Trace::Preimages => "traces/preimages",
                Trace::PreimagesLight => "traces/preimages_light",
                Trace::Safrole => "traces/safrole",
                Trace::Storage => "traces/storage",
                Trace::StorageLight => "traces/storage_light",
                Trace::Any => ".",
            },
        }
    }
}

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_ref())
    }
}

/// The traces section
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Trace {
    /// The fallback traces
    Fallback,
    /// The fuzzy traces
    Fuzzy,
    /// The preimages traces
    Preimages,
    /// The preimages traces light
    PreimagesLight,
    /// The safrole traces
    Safrole,
    /// The storage traces
    Storage,
    /// The storage traces light
    StorageLight,
    /// Any trace
    Any,
}
