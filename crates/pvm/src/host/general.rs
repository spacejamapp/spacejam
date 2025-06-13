//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    Argument, Reason, Result, State,
};
use score::{service::ServiceAccount, Gas, ServiceId, StorageKeyEncode};
use std::collections::BTreeMap;

/// Input data of general host functions
#[derive(Debug, Clone, Default)]
pub struct General {
    /// (s) The provided service account
    pub account: ServiceAccount,

    /// (s) Service index
    pub index: ServiceId,

    /// (d) Account dictionary
    pub accounts: BTreeMap<ServiceId, ServiceAccount>,
}

impl Argument for General {
    fn as_general(&self) -> Result<General> {
        Ok(self.clone())
    }

    fn update_general(&mut self, general: General) -> Result<()> {
        *self = general;
        Ok(())
    }
}

impl General {
    /// Get service account
    pub fn get(&self, r7: u64) -> Option<(ServiceId, ServiceAccount)> {
        let service = self.index as u64;
        if r7 == u64::MAX || r7 == service {
            return Some((service as ServiceId, self.account.clone()));
        }

        self.accounts
            .get(&(r7 as ServiceId))
            .map(|account| (r7 as ServiceId, account.clone()))
    }
}

/// General host calls
///
/// parameters: ϱ,ω,µ,s,...
///
/// with the range 0..5
pub fn call<X: Argument, Memory: crate::Memory>(
    call: u32,
    state: &mut State<Memory>,
    _account: ServiceAccount,
    data: &mut X,
) -> Result<ExitCode> {
    match call {
        0 => self::gas(state.gas as u64),
        1 => self::lookup(state, data),
        2 => self::read(state, data),
        3 => self::write(state, data),
        4 => self::info(state, data),
        _ => Ok(Exit::What as u64),
    }
}

/// (ΩG) Get the gas to register
fn gas(gas: Gas) -> Result<u64> {
    Ok(gas)
}

/// (ΩL) account lookup
fn lookup<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<u64> {
    let general = data.as_general()?;
    let Some((_, account)) = general.get(state.registers[7]) else {
        return Ok(Exit::None as u64);
    };

    // get the preimage
    let preimage = {
        let address = state.registers[8];

        // get the preimage hash
        let phash = state.memory.read_bytes(address as u32, 32)?;
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&phash);

        let Some(preimage) = account.preimage.get(&hash) else {
            return Ok(Exit::None as u64);
        };

        preimage
    };

    // write patrial preimage to memory
    let plen = preimage.len() as u64;
    let (from, to) = (state.registers[10].min(plen), state.registers[11].min(plen));
    state.memory.write_bytes(
        state.registers[9] as u32,
        &preimage[from as usize..to as usize],
    )?;

    Ok(plen)
}

/// (ΩR) storage lookup
fn read<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let general = data.as_general()?;

    // get the account
    let Some((_index, account)) = general.get(state.registers[7]) else {
        return Ok(Exit::None as u64);
    };

    // get the key
    let [ko, kz, o] = [state.registers[8], state.registers[9], state.registers[10]];
    let key = state
        .memory
        .read_bytes(ko as u32, kz as u32)
        .expect("should not fail");

    // get the storage value
    let skey = (general.index, key.clone()).key();
    let Some(value) = account.storage.get(skey.as_slice()) else {
        return Ok(Exit::None as u64);
    };

    let vlen = value.len() as u64;
    let from = state.registers[11].min(value.len() as u64);
    let length = state.registers[12].min(vlen - from);

    if length > 0 {
        state
            .memory
            .write_bytes(o as u32, &value[from as usize..(from + length) as usize])?;
    }
    Ok(vlen)
}

/// (ΩW) storage write
fn write<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let mut general = match data.as_general() {
        Ok(g) => g,
        Err(e) => {
            tracing::error!("as_general() failed: {:?}", e);
            return Err(e);
        }
    };

    // extract arguments from registers
    let [ko, kz, vo, vz] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    // Get key bytes from memory, log both address and length to help with debugging
    let key = match state.memory.read_bytes(ko as u32, kz as u32) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!("Failed to read key bytes: {:?}", err);
            // Note this OOB will be converted to panic in the caller
            return Ok(Exit::OOB as u64);
        }
    };

    // update storage
    let skey = (general.index, key.clone()).key();
    if vz == 0 {
        let value = general
            .account
            .storage
            .remove(skey.as_slice())
            .unwrap_or_default();
        data.update_general(general)?;
        Ok(value.len() as u64)
    } else {
        let value = match state.memory.read_bytes(vo as u32, vz as u32) {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::warn!("Failed to read value bytes: {:?}", err);
                return Ok(Exit::OOB as u64);
            }
        };

        let threshold = general.account.threshold();
        if threshold > general.account.balance {
            Ok(Exit::Full as u64)
        } else {
            tracing::info!("writing storage: {:?}", skey);
            let length = value.len() as u64;
            general.account.storage.insert(skey.to_vec(), value);
            data.update_general(general)?;
            Ok(length)
        }
    }
}

/// (ΩI) fetch info
fn info<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let general = data.as_general()?;

    // get and encode the account state
    let r7 = state.registers[7];
    let Some(account) = if r7 == u64::MAX {
        general.accounts.get(&general.index)
    } else {
        general.accounts.get(&(r7 as ServiceId))
    }
    .and_then(|account| {
        let state = account.state();
        tracing::debug!("account info: {:?}", state);
        codec::encode(&state).ok()
    }) else {
        return Ok(Exit::None as u64);
    };

    // write the account state to memory
    let address = state.registers[8];
    if let Err(reason) = state.memory.write_bytes(address as u32, &account) {
        crate::bail!("failed to write account state {reason}");
    }

    Ok(Exit::Ok as u64)
}
