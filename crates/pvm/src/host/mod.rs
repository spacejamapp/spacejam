//! Host functions

use crate::{invocation::Argument, Reason};

mod accumulate;
mod general;
mod jip;
mod refine;

/// Call the host function
pub fn call<X: Argument>(call: u32, mut ctx: X) -> (Reason, X) {
    if !X::SUPPORTED_CALLS.contains(&call) {
        tracing::error!("unsupported host call: {}", call);
        ctx.rset(7, Exit::What as u64);
        return (Reason::Continue, ctx);
    }

    tracing::debug!("calling host call {call}");
    let reason = match call {
        0 => general::gas(&ctx),
        1 => general::fetch(&mut ctx),
        2 => general::lookup(&mut ctx),
        3 => general::read(&mut ctx),
        4 => general::write(&mut ctx),
        5 => general::info(&mut ctx),
        6..14 => {
            tracing::error!("refine host call: {}", call);
            Ok(Exit::What as u64)
        }
        14 => accumulate::bless(&mut ctx),
        15 => accumulate::assign(&mut ctx),
        16 => accumulate::designate(&mut ctx),
        17 => accumulate::checkpoint(&mut ctx),
        18 => accumulate::new_(&mut ctx),
        19 => accumulate::upgrade(&mut ctx),
        20 => accumulate::transfer(&mut ctx),
        21 => accumulate::eject(&mut ctx),
        22 => accumulate::query(&mut ctx),
        23 => accumulate::solicit(&mut ctx),
        24 => accumulate::forget(&mut ctx),
        25 => accumulate::yield_(&mut ctx),
        26 => accumulate::provide(&mut ctx),
        100 => jip::log(&mut ctx),
        _ => {
            tracing::debug!("unknown host call: {}", call);
            Ok(Exit::What as u64)
        }
    };

    match reason {
        Ok(exit) => {
            ctx.rset(7, exit);
            (Reason::Continue, ctx)
        }
        Err(reason) => (reason, ctx),
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
