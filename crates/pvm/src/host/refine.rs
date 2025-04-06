//! refine host calls

use crate::{Reason, State};

/// Reine host call arguments
pub struct Refine {}

/// refine host call
pub fn call<X: Default, Memory: parser::Memory>(
    call: u32,
    state: &mut State<Memory>,
    data: &mut X,
) -> Reason {
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
        _ => Reason::Panic("host call not found".into()),
    }
}

/// (ΩH) historical lookup
fn historical_lookup<X: Default, Memory: parser::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Reason {
    Reason::Continue
}

/// (ΩP) fetch
fn fetch<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩX) export
fn export<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩM) machine
fn machine<X: Default, Memory: parser::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Reason {
    Reason::Continue
}

/// (ΩP) peek
fn peek<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩP) poke
fn poke<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩZ) zero
fn zero<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩV) void
fn void<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩI) invoke
fn invoke<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩE) expunge
fn expunge<X: Default, Memory: parser::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Reason {
    Reason::Continue
}
