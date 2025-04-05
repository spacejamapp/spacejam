//! General host call functions
use crate::State;
use score::{service::ServiceAccount, Gas};

/// General host calls
///
/// parameters: ϱ,ω,µ,s,...
///
/// with the range 0..4
pub fn call<X: Default, Memory: parser::Memory>(
    call: u32,
    state: &mut State<Memory>,
    _account: ServiceAccount,
    data: &mut X,
) {
    match call {
        0 => self::gas(&mut state.registers, state.gas as u64),
        1 => self::lookup(state, data),
        2 => self::read(state, data),
        3 => self::write(state, data),
        4 => self::info(state, data),
        _ => {}
    };
}

/// (ΩG) Get the gas to register
fn gas(registers: &mut [u64; 13], gas: Gas) {
    registers[7] = gas;
}

/// (ΩL) account lookup
fn lookup<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩR) storage lookup
fn read<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩW) storage write
fn write<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}

/// (ΩI) fetch info
fn info<X: Default, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) {}
