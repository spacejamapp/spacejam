//! Accumulation related host calls

use crate::{Reason, State};

/// Accumulate arguments
pub struct Accumulate {
    /// argument x of the accumulation result
    pub x: Vec<u8>,
    /// argument y of the accumulation result
    pub y: Vec<u8>,
}

/// Accumulation calls
pub fn call<X: Default, Memory: crate::Memory>(
    call: u32,
    state: &mut State<Memory>,
    data: &mut X,
) -> Reason {
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
        _ => Reason::Panic("Host call not found".into()),
    }
}

/// (ΩB) bless
fn bless<X: Default, Memory: crate::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩA) assign
fn assign<X: Default, Memory: crate::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩD) designate
fn designate<X: Default, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Reason {
    Reason::Continue
}

/// (ΩC) checkpoint
fn checkpoint<X: Default, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Reason {
    Reason::Continue
}

/// (ΩN) new
#[allow(clippy::new_ret_no_self)]
fn new<X: Default, Memory: crate::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩU) upgrade
fn upgrade<X: Default, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Reason {
    Reason::Continue
}

/// (ΩT) transfer
fn transfer<X: Default, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Reason {
    Reason::Continue
}

/// (ΩE) eject
fn eject<X: Default, Memory: crate::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩQ) query
fn query<X: Default, Memory: crate::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩS) solicit
fn solicit<X: Default, Memory: crate::Memory>(
    _state: &mut State<Memory>,
    _data: &mut X,
) -> Reason {
    Reason::Continue
}

/// (ΩF) forget
fn forget<X: Default, Memory: crate::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩY) yield
fn yield_<X: Default, Memory: crate::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}
