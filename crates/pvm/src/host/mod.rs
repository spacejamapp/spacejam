//! Host functions

use crate::{
    invocation::{Argument, State, Stepped},
    Reason,
};

mod accumulate;
mod general;
mod jip;
mod refine;

/// Call the host function
pub fn call<X: Argument>(call: u32, mut state: State, data: X) -> Stepped<X> {
    let mut data = data;
    tracing::debug!("calling host call {call}");
    let reason = match call {
        0 => general::gas(&mut state),
        1 => general::fetch(&mut data, &mut state),
        2 => general::lookup(&mut data, &mut state),
        3 => general::read(&mut data, &mut state),
        4 => general::write(&mut data, &mut state),
        5 => general::info(&mut data, &mut state),
        6..14 => {
            tracing::error!("refine host call: {}", call);
            Ok(Exit::What as u64)
        }
        14 => accumulate::bless(&mut data, &mut state),
        15 => accumulate::assign(&mut data, &mut state),
        16 => accumulate::designate(&mut data, &mut state),
        17 => accumulate::checkpoint(&mut data, &mut state),
        18 => accumulate::new_(&mut data, &mut state),
        19 => accumulate::upgrade(&mut data, &mut state),
        20 => accumulate::transfer(&mut data, &mut state),
        21 => accumulate::eject(&mut data, &mut state),
        22 => accumulate::query(&mut data, &mut state),
        23 => accumulate::solicit(&mut data, &mut state),
        24 => accumulate::forget(&mut data, &mut state),
        25 => accumulate::yield_(&mut data, &mut state),
        26 => accumulate::provide(&mut data, &mut state),
        100 => jip::log(&mut state),
        _ => {
            tracing::debug!("unknown host call: {}", call);
            Ok(Exit::What as u64)
        }
    };

    match reason {
        Ok(exit) => {
            state.registers[7] = exit;
            Stepped::new(Reason::Continue, state).with(data)
        }
        Err(reason) => Stepped::new(reason, state).with(data),
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
