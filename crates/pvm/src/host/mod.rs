//! Host functions

use crate::{Reason, State, Stepped};
pub use {accumulate::Accumulate, general::General, refine::Refine};

mod accumulate;
mod general;
mod refine;

/// Call the host function
pub fn call<X: Argument, Memory: crate::Memory>(
    call: u32,
    mut state: State<Memory>,
    data: X,
) -> Stepped<Memory, X> {
    tracing::debug!("host call dispatcher: call={}", call);
    let mut data = data;
    let reason = match call {
        0..6 => {
            tracing::debug!("routing to general::call");
            general::call(call, &mut state, Default::default(), &mut data)
        }
        6..18 => {
            tracing::debug!("routing to accumulate::call");
            accumulate::call(call, &mut state, &mut data)
        }
        18..28 => {
            tracing::debug!("routing to refine::call");
            refine::call(call, &mut state, &mut data)
        }
        // JIP1 logging, currently skipped
        100 => {
            tracing::debug!("routing to logging (100)");
            Ok(Exit::Ok as u64)
        }
        _ => {
            tracing::debug!("unknown host call: {}", call);
            Ok(Exit::What as u64)
        }
    };

    tracing::debug!("host call {} result: {:?}", call, reason);
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
    fn as_general(&self) -> crate::Result<General> {
        crate::bail!("not a general")
    }

    /// update the general argument
    fn update_general(&mut self, _general: General) -> crate::Result<()> {
        crate::bail!("not a general")
    }

    /// returns some if the input data is general
    fn as_general_mut(&mut self) -> crate::Result<&mut General> {
        crate::bail!("not a general")
    }

    /// returns some if the input data is accumulate
    fn as_accumulate_mut(&mut self) -> crate::Result<&mut Accumulate> {
        crate::bail!("not an accumulate")
    }

    /// returns some if the input data is refine
    fn as_refine_mut(&mut self) -> crate::Result<&mut Refine> {
        crate::bail!("not a refine")
    }
}

impl Argument for Accumulate {
    fn as_general(&self) -> crate::Result<General> {
        Ok(General {
            account: self
                .x
                .context
                .accounts
                .get(&self.x.service)
                .unwrap()
                .clone(),
            index: self.x.service,
            accounts: self.x.context.accounts.clone(),
        })
    }

    fn update_general(&mut self, general: General) -> crate::Result<()> {
        self.x.context.accounts = general.accounts;
        self.x.service = general.index;
        Ok(())
    }

    fn as_accumulate_mut(&mut self) -> crate::Result<&mut Accumulate> {
        Ok(self)
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
