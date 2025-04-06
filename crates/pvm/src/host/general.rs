//! General host call functions

use crate::{Argument, Reason, State};
use score::{service::ServiceAccount, Gas};
use std::collections::BTreeMap;

/// Input data of general host functions
pub struct General {
    /// (s) The provided service account
    pub account: ServiceAccount,

    /// () Service index
    pub index: u64,

    /// () Account dicionary
    pub accounts: BTreeMap<u64, ServiceAccount>,
}

/// General host calls
///
/// parameters: ϱ,ω,µ,s,...
///
/// with the range 0..4
pub fn call<X: Argument, Memory: parser::Memory>(
    call: u32,
    state: &mut State<Memory>,
    _account: ServiceAccount,
    data: &mut X,
) -> Reason {
    match call {
        0 => self::gas(&mut state.registers, state.gas as u64),
        1 => self::lookup(state, data),
        2 => self::read(state, data),
        3 => self::write(state, data),
        4 => self::info(state, data),
        _ => Reason::Panic("host call not found".into()),
    }
}

/// (ΩG) Get the gas to register
fn gas(registers: &mut [u64; 13], gas: Gas) -> Reason {
    registers[7] = gas;
    Reason::Continue
}

/// (ΩL) account lookup
fn lookup<X: Argument, Memory: parser::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(general) = data.as_general_mut() else {
        return Reason::Panic("could not find general arguments".into());
    };

    // get the account
    let account = {
        let mut account: Option<ServiceAccount> = None;
        let mbindex = state.registers[7];
        if mbindex >= general.index {
            account = Some(general.account.clone());
        } else if let Some(acc) = general.accounts.get(&mbindex) {
            account = Some(acc.clone())
        }
        account
    };

    let preimage = {
        let address = state.registers[8];
    };

    let o = state.registers[9];

    Reason::Continue
}

/// (ΩR) storage lookup
fn read<X: Argument, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩW) storage write
fn write<X: Argument, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}

/// (ΩI) fetch info
fn info<X: Argument, Memory: parser::Memory>(_state: &mut State<Memory>, _data: &mut X) -> Reason {
    Reason::Continue
}
