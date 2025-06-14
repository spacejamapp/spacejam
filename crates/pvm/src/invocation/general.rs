//! Primitives for the general invocation

use crate::{Gas, Reason};

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
