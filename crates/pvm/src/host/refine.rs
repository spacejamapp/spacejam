//! refine host calls

use crate::State;

/// refine host call
pub fn call<X: Default, Memory: parser::Memory>(
    call: u32,
    state: &mut State<Memory>,
    data: &mut X,
) {
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
        _ => {}
    }
}

/// (ΩH) historical lookup
fn historical_lookup<X: Default, Memory: parser::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) {
}

/// (ΩP) fetch
fn fetch<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩX) export
fn export<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩM) machine
fn machine<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩP) peek
fn peek<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩP) poke
fn poke<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩZ) zero
fn zero<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩV) void
fn void<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩI) invoke
fn invoke<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩE) expunge
fn expunge<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}
