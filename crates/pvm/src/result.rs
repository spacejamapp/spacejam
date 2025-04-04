//! PVM execution result

use core::fmt;
use score::{
    service::{ServiceAccount, WorkExecResult},
    Gas,
};
use std::fmt::Display;

/// The program exit reason.
///
/// As defined per the graypaper (A.2)
#[derive(Debug, Default)]
pub enum Reason {
    /// The program has halted.
    Halt,

    /// The program has panicked.
    Panic(String),

    /// The program has run out of gas.
    OOG,

    /// The invocation completed with a page fault.
    Fault(u32),

    /// The status is unknown.
    HostCall(u32),

    /// The program is still running.
    #[default]
    Continue,
}

impl Reason {
    /// Check if the reason is a trap.
    pub fn is_continue(&self) -> bool {
        matches!(self, Reason::Continue)
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
                Reason::Fault(_) => "page-fault".to_string(),
                Reason::HostCall(addr) => format!("host-call({addr})"),
            }
        )
    }
}

/// The execution state of programs.
#[derive(Default)]
pub struct State<Memory: Default> {
    /// (ı') The program counter.
    pub pc: u64,

    /// (ϱ') The gas left.
    pub gas: i64,

    /// (ω') The registers.
    pub registers: [u64; 13],

    /// (µ') The memory.
    pub memory: Memory,
}

/// The result of step invocation (Ψ1)
pub struct Stepped<Memory: Default, X> {
    /// (ε) the reason for exiting
    pub reason: Reason,

    /// (U) The newly updated state
    pub state: State<Memory>,

    /// (X) the data
    pub data: X,
}

impl<Memory: Default, X: Default> Stepped<Memory, X> {
    /// Create a new stepped result
    pub fn new(reason: Reason, state: State<Memory>) -> Self {
        Self {
            reason,
            state,
            data: X::default(),
        }
    }

    /// Create a new stepped result with the given data
    pub fn with(self, data: X) -> Self {
        Self {
            reason: self.reason,
            state: self.state,
            data,
        }
    }
}

/// The received data from (ΨM)
pub struct Received<X: Default> {
    /// The gas we used
    pub gas: Gas,

    /// The output
    pub output: Vec<u8>,

    /// program exit-reason
    pub reason: Reason,

    /// The data we got
    pub data: X,
}

impl<X: Default> Received<X> {
    /// Create a new received result
    pub fn new(gas: Gas, output: Vec<u8>, reason: Reason) -> Self {
        Self {
            gas,
            output,
            reason,
            data: X::default(),
        }
    }

    /// Create a new extracted result with the given data
    pub fn with(self, data: X) -> Self {
        Self {
            gas: self.gas,
            output: self.output,
            reason: self.reason,
            data,
        }
    }
}

/// The result of is-authorized invocation (ΨI)
pub struct Executed {
    /// The output
    pub data: Vec<u8>,

    /// The reason
    pub exec: WorkExecResult,

    /// The gas used
    pub gas: Gas,
}

impl Executed {
    /// Create a new executed result
    pub fn new(data: Vec<u8>, exec: WorkExecResult, gas: Gas) -> Self {
        Self { data, exec, gas }
    }
}

/// The result of refine invocation (ΨR)
pub struct Refined {
    /// The executed result
    pub executed: Executed,

    /// The imports
    pub segments: Vec<[u8; score::SEGMENT_SIZE]>,
}

impl Refined {
    /// Create a new refined result
    pub fn new(executed: Executed, segments: Vec<[u8; score::SEGMENT_SIZE]>) -> Self {
        Self { executed, segments }
    }
}

/// The result of transfer invocation (ΨT)
#[derive(Default)]
pub struct Transferred {
    /// The account
    pub account: ServiceAccount,

    /// The gas used
    pub gas: Gas,
}
