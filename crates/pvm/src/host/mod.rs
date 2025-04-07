//! Host functions

use crate::{Reason, State, Stepped};
use accumulate::Accumulate;
pub use general::General;
use refine::Refine;

mod accumulate;
mod general;
mod refine;

/// Call the host function
pub fn call<X: Argument, Memory: crate::Memory>(
    call: u32,
    state: State<Memory>,
    data: X,
) -> Stepped<Memory, X> {
    let mut state = state;
    let mut data = data;
    let reason = match call {
        0..5 => general::call(call, &mut state, Default::default(), &mut data),
        5..17 => accumulate::call(call, &mut state, &mut data),
        17..27 => refine::call(call, &mut state, &mut data),
        _ => Err(Reason::Panic(format!("unknown host call: {call}"))),
    };

    match reason {
        Ok(exit) => {
            state.registers[7] = exit;
            Stepped::new(Reason::Continue, state)
        }
        Err(reason) => Stepped::new(reason, state),
    }
}

/// Dynamic arguments for host calls
pub trait Argument: Default {
    /// returns some if the input data is general
    fn as_general() -> crate::Result<General> {
        crate::bail!("not a general")
    }

    /// returns some if the input data is general
    fn as_general_mut(&mut self) -> crate::Result<&mut General> {
        crate::bail!("not a general")
    }

    /// returns some if the input data is accumulate
    fn as_accumulate() -> crate::Result<Accumulate> {
        crate::bail!("not an accumulate")
    }

    /// returns some if the input data is accumulate
    fn as_accumulate_mut(&mut self) -> crate::Result<&mut Accumulate> {
        crate::bail!("not an accumulate")
    }

    /// returns some if the input data is refine
    fn as_refine() -> crate::Result<Refine> {
        crate::bail!("not a refine")
    }

    /// returns some if the input data is refine
    fn as_refine_mut(&mut self) -> crate::Result<&mut Refine> {
        crate::bail!("not a refine")
    }
}

/// Host call results
#[repr(u64)]
pub enum Result {
    /// The return value indicating an item does not exist.
    None = u64::MAX,
    /// Name unknown.
    What = u64::MAX - 1,
    /// The inner PVM memory index provided for reading/writing is not accessible.
    OOB = u64::MAX - 2,
    /// Index unknown
    Who = u64::MAX - 3,
    /// Storage full
    Full = u64::MAX - 4,
    /// Core index unknown
    Core = u64::MAX - 5,
    /// Insufficient funds
    Cash = u64::MAX - 6,
    /// Gas limit too low
    Low = u64::MAX - 7,
    /// The item is already solicited or cannot be forgotten.
    Huh = u64::MAX - 8,
    /// The return value indicating general success.
    Ok = 0,
}

/// The result type of host calls
pub type Exit = Result;

/// The exit code type
pub type ExitCode = u64;
