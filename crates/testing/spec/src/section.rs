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
            "traces/safrole" => Ok(Section::Trace(Trace::Safrole)),
            "reports-l0" | "traces/reports-l0" => Ok(Section::Trace(Trace::ReportsL0)),
            "reports-l1" | "traces/reports-l1" => Ok(Section::Trace(Trace::ReportsL1)),
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
            Section::Erasure => "erasure-coding",
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
                Trace::Safrole => "traces/safrole",
                Trace::ReportsL0 => "traces/reports-l0",
                Trace::ReportsL1 => "traces/reports-l1",
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
    /// The safrole traces
    Safrole,
    /// The reports traces
    ReportsL0,
    /// The reports traces
    ReportsL1,
}

impl AsRef<str> for Trace {
    fn as_ref(&self) -> &str {
        match self {
            Trace::Fallback => "fallback",
            Trace::Safrole => "safrole",
            Trace::ReportsL0 => "reports-l0",
            Trace::ReportsL1 => "reports-l1",
        }
    }
}
