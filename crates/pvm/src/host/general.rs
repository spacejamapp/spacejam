//! General host call functions

use crate::{host::Result, Argument, Reason, State};
use codec::Numeric;
use score::{service::ServiceAccount, Gas};
use std::collections::BTreeMap;

/// Input data of general host functions
pub struct General {
    /// (s) The provided service account
    pub account: ServiceAccount,

    /// (s) Service index
    pub index: u64,

    /// (d) Account dictionary
    pub accounts: BTreeMap<u64, ServiceAccount>,
}

impl General {
    /// Get service account
    pub fn get(&self, r7: u64) -> Option<(u64, ServiceAccount)> {
        if r7 == u64::MAX || r7 == self.index {
            return Some((self.index, self.account.clone()));
        }

        self.accounts.get(&r7).map(|account| (r7, account.clone()))
    }
}

/// General host calls
///
/// parameters: ϱ,ω,µ,s,...
///
/// with the range 0..4
pub fn call<X: Argument, Memory: crate::Memory>(
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
fn lookup<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(general) = data.as_general_mut() else {
        return Reason::Panic("could not find general arguments".into());
    };

    // get the account
    let Some((_, account)) = general.get(state.registers[7]) else {
        state.registers[7] = Result::None as u64;
        return Reason::Continue;
    };

    // get the preimage
    let preimage = {
        let address = state.registers[8];

        // get the preimage hash
        let hash = match state.memory.read_bytes(address as u32, 32) {
            Ok(hash) => {
                let mut phash = [0u8; 32];
                phash.copy_from_slice(&hash);
                phash
            }
            Err(reason) => return reason,
        };

        let Some(preimage) = account.preimage.get(&hash) else {
            state.registers[7] = Result::None as u64;
            return Reason::Continue;
        };

        preimage
    };

    // write patrial preimage to memory
    let plen = preimage.len() as u64;
    let (from, to) = (state.registers[10].min(plen), state.registers[11].min(plen));
    if let Err(reason) = state.memory.write_bytes(
        state.registers[9] as u32,
        &preimage[from as usize..to as usize],
    ) {
        return reason;
    }

    state.registers[7] = plen;
    Reason::Continue
}

/// (ΩR) storage lookup
fn read<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(general) = data.as_general_mut() else {
        return Reason::Panic("could not find general arguments".into());
    };

    // get the account
    let Some((index, account)) = general.get(state.registers[7]) else {
        state.registers[7] = Result::None as u64;
        return Reason::Continue;
    };

    // get the key
    let [ko, kz, o] = [state.registers[8], state.registers[9], state.registers[10]];
    let mut input = codec::encode(&(index as u32)).expect("should not fail");
    let shash = state
        .memory
        .read_bytes(ko as u32, (ko + kz) as u32)
        .expect("should not fail");
    input.extend_from_slice(&shash);

    // get the storage value
    let Some(value) = account.storage.get(&crypto::blake2b(&input)) else {
        state.registers[7] = Result::None as u64;
        return Reason::Continue;
    };

    let vlen = value.len() as u64;
    let (from, to) = (state.registers[11].min(vlen), state.registers[12].min(vlen));
    if let Err(reason) = state
        .memory
        .write_bytes(o as u32, &value[from as usize..to as usize])
    {
        return Reason::Panic(format!("failed to write storage {reason}"));
    }

    state.registers[7] = value.len() as u64;
    Reason::Continue
}

/// (ΩW) storage write
fn write<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(general) = data.as_general_mut() else {
        return Reason::Panic("could not find general arguments".into());
    };

    // extract arguments from registers
    let [ko, kz, vo, vz] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    // get the key
    let mut input = codec::encode(&(general.index as u32)).expect("should not fail");
    input.extend_from_slice(
        &state
            .memory
            .read_bytes(ko as u32, kz as u32)
            .expect("should not fail"),
    );
    let key = crypto::blake2b(&input);

    // update storage
    state.registers[7] = if vz == 0 {
        general.account.storage.remove(&key);
        Result::None as u64
    } else if let Ok(value) = state.memory.read_bytes(vo as u32, (vo + vz) as u32) {
        let account = general.account.state();
        if account.threshold() > account.balance {
            Result::Full as u64
        } else {
            general.account.storage.insert(key, value.clone());
            u64::decode(&value)
        }
    } else {
        return Reason::Panic("failed to upsert storage".into());
    };
    Reason::Continue
}

/// (ΩI) fetch info
fn info<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(general) = data.as_general_mut() else {
        return Reason::Panic("could not find general arguments".into());
    };

    // get and encode the account state
    let r7 = state.registers[7];
    let Some(account) = if r7 == u64::MAX {
        general.accounts.get(&general.index)
    } else {
        general.accounts.get(&r7)
    }
    .and_then(|account| codec::encode(&account.state()).ok()) else {
        state.registers[7] = Result::None as u64;
        return Reason::Continue;
    };

    // write the account state to memory
    let address = state.registers[8];
    if let Err(reason) = state.memory.write_bytes(address as u32, &account) {
        return Reason::Panic(format!("failed to write account state {reason}"));
    }

    state.registers[7] = Result::Ok as u64;
    Reason::Continue
}
