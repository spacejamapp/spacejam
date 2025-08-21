//! Primitives for the general invocation

use crate::{invocation::Accumulate, Accumulated, Gas, Reason};
use score::{service::WorkExecResult, Accounts};

/// The execution state of programs.
#[derive(Default, Clone)]
pub struct State {
    /// (ı') The program counter.
    pub pc: u64,

    /// (ϱ') The gas left.
    pub gas: i64,

    /// (ω') The registers.
    pub registers: [u64; 13],

    /// (µ') The memory.
    pub memory: parser::Memory,
}

impl State {
    /// Create a new stepped result
    pub fn stepped(self, reason: Reason) -> Stepped<()> {
        Stepped {
            reason,
            state: self,
            data: (),
        }
    }
}

/// The result of step invocation (Ψ1)
pub struct Stepped<X> {
    /// (ε) the reason for exiting
    pub reason: Reason,

    /// (U) The newly updated state
    pub state: State,

    /// (X) the data
    pub data: X,
}

impl Stepped<()> {
    /// Create a new stepped result with the given reason
    pub fn new(reason: Reason, state: State) -> Self {
        Self {
            reason,
            state,
            data: (),
        }
    }

    /// Create a new stepped result with the given data
    pub fn with<X>(self, data: X) -> Stepped<X> {
        Stepped {
            reason: self.reason,
            state: self.state,
            data,
        }
    }
}

impl<X> Stepped<X> {
    /// Convert the stepped result to a received result
    pub fn received(self, gas: Gas, output: Vec<u8>) -> Received<X> {
        Received {
            gas,
            output,
            reason: self.reason,
            data: self.data,
        }
    }
}

/// The received data from (ΨM)
pub struct Received<X> {
    /// (u) The gas we used
    pub gas: Gas,

    /// (o) The output
    pub output: Vec<u8>,

    /// (e) program exit-reason
    pub reason: Reason,

    /// (m??) The data we got
    pub data: X,
}

impl<X> Received<X> {
    /// Create a new received result with a panic reason
    pub fn panic(message: impl ToString, data: X) -> Self {
        Self {
            gas: 0,
            output: Vec::new(),
            reason: Reason::Panic(message.to_string()),
            data,
        }
    }

    /// Convert the received result to a work exec result
    pub fn result(self) -> WorkExecResult {
        match self.reason {
            Reason::Halt => WorkExecResult::Ok(self.output.clone()),
            Reason::OOG => WorkExecResult::OutOfGas,
            Reason::Panic(_) => WorkExecResult::Panic,
            _ => WorkExecResult::BadCode,
        }
    }
}

impl<R: Accounts> Received<Accumulate<R>> {
    /// Convert the received result to an accumulate result
    pub fn to_result(self) -> Accumulated<R> {
        // Treat Continue and Halt as successful completion
        // Only Panic, OOG, and Fault should use Y context (exceptional dimension)
        match self.reason {
            Reason::Continue | Reason::Halt => {
                let mut result = self.data.x.to_result(self.gas, self.reason);
                if self.output.len() == 32 {
                    let mut hash = [0; 32];
                    hash.copy_from_slice(&self.output);
                    result.hash = Some(hash);
                }
                result
            }
            _ => self.data.y.to_result(self.gas, self.reason),
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

    /// Check if the execution is successful
    pub fn is_ok(&self) -> bool {
        matches!(self.exec, WorkExecResult::Ok(_))
    }
}
