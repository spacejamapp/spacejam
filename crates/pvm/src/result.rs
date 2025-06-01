//! PVM execution result

use crate::host::Accumulate;
use core::fmt;
use score::{
    service::{ServiceAccount, WorkExecResult},
    vm::{DeferredTransfer, StateContext},
    Gas, OpaqueHash,
};
use std::fmt::Display;

/// The result type of PVM
pub type Result<T> = core::result::Result<T, Reason>;

/// The program exit reason.
///
/// As defined per the graypaper (A.2)
#[derive(Debug, Default, PartialEq, Eq)]
pub enum Reason {
    /// The program has halted.
    Halt,

    /// The program has panicked.
    Panic(String),

    /// The program has run out of gas.
    OOG,

    /// The invocation completed with a page fault.
    Fault { page: u32 },

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

/// The execution state of programs.
#[derive(Default, Clone)]
pub struct State<Memory: crate::Memory> {
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
pub struct Stepped<Memory: crate::Memory, X> {
    /// (ε) the reason for exiting
    pub reason: Reason,

    /// (U) The newly updated state
    pub state: State<Memory>,

    /// (X) the data
    pub data: X,
}

impl<Memory: crate::Memory, X: Default> Stepped<Memory, X> {
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
    pub fn new(gas: Gas, reason: Reason, data: X) -> Self {
        Self {
            gas,
            reason,
            output: Vec::new(),
            data,
        }
    }

    /// Create a new extracted result with the given data
    pub fn with(self, output: Vec<u8>) -> Self {
        Self {
            gas: self.gas,
            output,
            reason: self.reason,
            data: self.data,
        }
    }
}

impl Received<Accumulate> {
    /// Convert the received result to an accumulate result
    pub fn to_result(self, gas: Gas) -> AccumulateResult {
        // Treat Continue and Halt as successful completion
        // Only Panic, OOG, and Fault should use Y context (exceptional dimension)
        match self.reason {
            Reason::Continue | Reason::Halt => {
                let mut result = self.data.x.to_result(gas);
                if self.output.len() == 32 {
                    let mut hash = [0; 32];
                    hash.copy_from_slice(&self.output);
                    result.hash = Some(hash);
                }
                result
            }
            _ => self.data.y.to_result(gas),
        }
    }
}

/// The accumulate result of (ΨA)
#[derive(Default)]
pub struct AccumulateResult {
    /// (o) The state context
    pub context: StateContext,

    /// (t) The timeslot for the current accumulation
    pub transfers: Vec<DeferredTransfer>,

    /// (b) The output hash of the accumulation
    pub hash: Option<OpaqueHash>,

    /// (u) The gas used
    pub gas: Gas,
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
