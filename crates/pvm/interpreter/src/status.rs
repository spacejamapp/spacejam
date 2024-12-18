//! The status of the PVM.

/// The status of the PVM.
#[derive(Debug, Clone, Default, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Status {
    /// The program has failed to execute.
    Trap,

    /// The program has halted.
    Halt,

    /// The program has successfully executed.
    Success,

    /// The program has run out of gas.
    OutOfGas,

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
        *self == Status::Trap
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("{self:?}").to_lowercase())
    }
}
