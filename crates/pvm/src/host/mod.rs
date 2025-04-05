//! Host functions

use crate::{Reason, State, Stepped};

mod accumulate;
mod general;
mod refine;

/// Call the host function
pub fn call<X: Default, Memory: parser::Memory>(
    call: u32,
    state: State<Memory>,
    data: X,
) -> Stepped<Memory, X> {
    let mut state = state;
    let mut data = data;
    match call {
        0..5 => general::call(call, &mut state, Default::default(), &mut data),
        5..17 => accumulate::call(call, &mut state, &mut data),
        17..27 => refine::call(call, &mut state, &mut data),
        _ => return Stepped::new(Reason::Panic(format!("unknown host call: {call}")), state),
    };

    Stepped::new(Reason::Halt, state)
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
    /// Storage full
    Full = u64::MAX - 3,
    /// Core index unknown
    Core = u64::MAX - 4,
    /// Insufficient funds
    Cash = u64::MAX - 5,
    /// Gas limit too low
    Low = u64::MAX - 6,
    /// The item is already solicited or cannot be forgotten.
    Huh = u64::MAX - 7,
    /// The return value indicating general success.
    Ok = 0,
}
