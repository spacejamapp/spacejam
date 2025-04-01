//! PVM execution result

/// The result type for the PVM invocation.
pub type Result<T> = std::result::Result<T, Reason>;

/// The program exit reason.
///
/// As defined per the graypaper (A.2)
pub enum Reason {
    /// The program is still running.
    Continue,

    /// The program has halted.
    Halt,

    /// The program has panicked.
    Panic(String),

    /// The program has run out of gas.
    OOG,

    /// The invocation completed with a page fault.
    Fault(u64),

    /// The status is unknown.
    HostCall(u64),
}

/// The execution state of programs.
#[derive(Default)]
pub struct State {
    /// (ı') The program counter.
    pub pc: u64,

    /// (ϱ') The gas left.
    pub gas: i64,

    /// (ω') The registers.
    pub registers: [u64; 13],

    /// (µ') The memory.
    pub memory: Vec<u32>,
}
