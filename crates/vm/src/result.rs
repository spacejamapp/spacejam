//! PVM execution result

use core::fmt;
use std::fmt::Display;

/// The result type of PVM
pub type Result<T> = core::result::Result<T, Reason>;

/// The program exit reason.
///
/// As defined per the graypaper (A.2)
#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum Reason {
    /// The program has halted.
    Halt,

    /// The program has panicked.
    Panic(String),

    /// The invocation completed with a page fault.
    Fault { page: u32 },

    /// The status is unknown.
    HostCall(u32),

    /// The program has run out of gas.
    OOG,

    /// The program is still running.
    #[default]
    Continue,
}

impl Reason {
    /// Check if the reason is a trap.
    pub fn is_continue(&self) -> bool {
        matches!(self, Reason::Continue)
    }

    /// Check if the reason is an error.
    pub fn is_err(&self) -> bool {
        matches!(
            self,
            Reason::Halt | Reason::Panic(_) | Reason::OOG | Reason::Fault { page: _ }
        )
    }
}

impl From<anyhow::Error> for Reason {
    fn from(e: anyhow::Error) -> Self {
        Reason::Panic(e.to_string())
    }
}

impl Display for Reason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Reason::Continue => "continue".to_string(),
                Reason::Halt => "halt".to_string(),
                Reason::Panic(_) => "panic".to_string(),
                Reason::OOG => "OOG".to_string(),
                Reason::Fault { page: _ } => "page-fault".to_string(),
                Reason::HostCall(addr) => format!("host-call({addr})"),
            }
        )
    }
}
