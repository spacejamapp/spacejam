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

impl Section {
    /// The all sections
    pub fn all() -> [Section; 16] {
        [
            Section::Accumulate,
            Section::Assurances,
            Section::Authorizations,
            Section::Codec,
            Section::Pvm,
            Section::Safrole,
            Section::Statistics,
            Section::Disputes,
            Section::History,
            Section::Preimages,
            Section::Reports,
            Section::Shuffle,
            Section::Trie,
            Section::Trace(Trace::Fallback),
            Section::Trace(Trace::Safrole),
            Section::Trace(Trace::ReportsL0),
        ]
    }
}

impl FromStr for Section {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "codec" => Ok(Section::Codec),
            "pvm" => Ok(Section::Pvm),
            "shuffle" => Ok(Section::Shuffle),
            "trie" => Ok(Section::Trie),
            "accumulate" => Ok(Section::Accumulate),
            "assurances" => Ok(Section::Assurances),
            "safrole" => Ok(Section::Safrole),
            "statistics" => Ok(Section::Statistics),
            "authorizations" => Ok(Section::Authorizations),
            "disputes" => Ok(Section::Disputes),
            "history" => Ok(Section::History),
            "preimages" => Ok(Section::Preimages),
            "reports" => Ok(Section::Reports),
            "traces/fallback" => Ok(Section::Trace(Trace::Fallback)),
            "traces/safrole" => Ok(Section::Trace(Trace::Safrole)),
            "traces/reports-l0" => Ok(Section::Trace(Trace::ReportsL0)),
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
            Section::Accumulate => "stf/accumulate",
            Section::Assurances => "stf/assurances",
            Section::Safrole => "stf/safrole",
            Section::Statistics => "stf/statistics",
            Section::Authorizations => "stf/authorizations",
            Section::Disputes => "stf/disputes",
            Section::History => "stf/history/data",
            Section::Preimages => "stf/preimages/data",
            Section::Reports => "stf/reports",
            Section::Trace(trace) => match trace {
                Trace::Fallback => "traces/fallback",
                Trace::Safrole => "traces/safrole",
                Trace::ReportsL0 => "traces/reports-l0",
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
}

impl AsRef<str> for Trace {
    fn as_ref(&self) -> &str {
        match self {
            Trace::Fallback => "fallback",
            Trace::Safrole => "safrole",
            Trace::ReportsL0 => "reports-l0",
        }
    }
}
