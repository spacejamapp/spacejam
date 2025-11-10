//! Host functions

use crate::{Argument, Reason};

mod accumulate;
mod general;
mod jip;
mod refine;

/// Call the host function
pub fn call<X: Argument>(call: u32, ctx: &mut X) -> Reason {
    if !X::SUPPORTED_CALLS.contains(&call) {
        tracing::error!("unsupported host call: {}", call);
        ctx.rset(7, Exit::What as u64);
        return Reason::Continue;
    }

    tracing::debug!("host call: {}", call);
    let reason = match call {
        0 => general::gas(ctx),
        1 => general::fetch(ctx),
        2 => general::lookup(ctx),
        3 => general::read(ctx),
        4 => general::write(ctx),
        5 => general::info(ctx),
        6..14 => {
            tracing::warn!("refine host call: {}", call);
            Ok(Exit::What as u64)
        }
        14 => accumulate::bless(ctx),
        15 => accumulate::assign(ctx),
        16 => accumulate::designate(ctx),
        17 => accumulate::checkpoint(ctx),
        18 => accumulate::new_(ctx),
        19 => accumulate::upgrade(ctx),
        20 => accumulate::transfer(ctx),
        21 => accumulate::eject(ctx),
        22 => accumulate::query(ctx),
        23 => accumulate::solicit(ctx),
        24 => accumulate::forget(ctx),
        25 => accumulate::yield_(ctx),
        26 => accumulate::provide(ctx),
        100 => jip::log(ctx),
        _ => {
            tracing::warn!("unknown host call: {}", call);
            Ok(Exit::What as u64)
        }
    };

    match reason {
        Ok(exit) => {
            ctx.rset(7, exit);
            Reason::Continue
        }
        Err(reason) => reason,
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
