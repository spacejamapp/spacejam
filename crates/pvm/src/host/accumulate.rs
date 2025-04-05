//! Accumulation related host calls

use crate::State;

/// Accumulation calls
pub fn call<X: Default, Memory: parser::Memory>(
    call: u32,
    state: &mut State<Memory>,
    data: &mut X,
) {
    match call {
        5 => self::bless(state, data),
        6 => self::assign(state, data),
        7 => self::designate(state, data),
        8 => self::checkpoint(state, data),
        9 => self::new(state, data),
        10 => self::upgrade(state, data),
        11 => self::transfer(state, data),
        12 => self::eject(state, data),
        13 => self::query(state, data),
        14 => self::solicit(state, data),
        15 => self::forget(state, data),
        16 => self::yield_(state, data),
        _ => {}
    }
}

/// (ΩB) bless
fn bless<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩA) assign
fn assign<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩD) designate
fn designate<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩC) checkpoint
fn checkpoint<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩN) new
#[allow(clippy::new_ret_no_self)]
fn new<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩU) upgrade
fn upgrade<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩT) transfer
fn transfer<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩE) eject
fn eject<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩQ) query
fn query<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩS) solicit
fn solicit<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩF) forget
fn forget<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩY) yield
fn yield_<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}
