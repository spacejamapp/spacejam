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
}
