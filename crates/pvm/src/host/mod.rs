//! Host functions

use crate::{
    invocation::{Argument, State, Stepped},
    Reason,
};
use score::Accounts;

mod accumulate;
mod general;
mod jip;
mod refine;

/// Call the host function
pub fn call<R: Accounts, X: Argument<R>, Memory: crate::Memory>(
    call: u32,
    mut state: State<Memory>,
    data: X,
) -> Stepped<Memory, X> {
    let mut data = data;
    tracing::debug!("calling host call {call}");
    let reason = match call {
        0 => general::gas(&mut state),
        1..6 => {
            let mut general = match data.as_general() {
                Ok(g) => g,
                Err(e) => return Stepped::new(e, state).with(data),
            };
            let ret = general.call(call, &mut state);
            if general.updated {
                if let Err(e) = data.update_general(general) {
                    return Stepped::new(e, state).with(data);
                }
            }

            ret
        }
        6..14 => {
            tracing::error!("refine host call: {}", call);
            // refine::call(call, &mut state, &mut data)
            Ok(Exit::What as u64)
        }
        14..27 => {
            let accumulate = match data.as_accumulate_mut() {
                Ok(a) => a,
                Err(e) => return Stepped::new(e, state).with(data),
            };
            accumulate.call(call, &mut state)
        }
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
