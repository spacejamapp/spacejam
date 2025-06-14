//! refine host calls

use crate::{
    host::{Exit, ExitCode},
    invocation::general::State,
    Argument, Reason, Result,
};

/// Reine host call arguments
pub struct Refine {}

/// refine host call
pub fn call<X: Argument, Memory: crate::Memory>(
    call: u32,
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    match call {
        17 => self::historical_lookup(state, data),
        18 => self::fetch(state, data),
        19 => self::export(state, data),
        20 => self::machine(state, data),
        21 => self::peek(state, data),
        22 => self::poke(state, data),
        23 => self::zero(state, data),
        24 => self::void(state, data),
        25 => self::invoke(state, data),
        26 => self::expunge(state, data),
        _ => Ok(Exit::What as u64),
    }
}

/// (ΩH) historical lookup
fn historical_lookup<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩP) fetch
fn fetch<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩX) export
fn export<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩM) machine
fn machine<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩP) peek
fn peek<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩP) poke
fn poke<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩZ) zero
fn zero<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩV) void
fn void<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩI) invoke
fn invoke<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}

/// (ΩE) expunge
fn expunge<X: Argument, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    crate::bail!("not implemented")
}
