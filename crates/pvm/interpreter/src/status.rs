//! The status of the PVM.

/// The status of the PVM.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    /// The program has halted.
    Halt,

    /// The program has panicked.
    Panic,

    /// The invocation completed with a page fault.
    Fault(u32),

    /// The invocation completed with a host-call fault.
    Host,

    /// The program has run out of gas.
    OOG,

    /// The status is unknown.
    #[default]
    Unknown,
}

impl Status {
    /// Check if the status is unknown.
    pub fn is_unknown(&self) -> bool {
        *self == Status::Unknown
    }

    /// Check if the status is a trap.
    pub fn is_trap(&self) -> bool {
        *self == Status::Panic
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Status::Fault(_) => "page-fault".into(),
            _ => format!("{self:?}").to_lowercase(),
        };

        write!(f, "{s}")
    }
}
