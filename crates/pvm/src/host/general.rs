//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    Argument, Reason, Result, State,
};
use codec::Numeric;
use score::{service::ServiceAccount, Gas, ServiceId};
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
        4 => self::sbrk(state, data),
        5 => self::info(state, data),
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
    let Some((index, account)) = general.get(state.registers[7]) else {
        return Ok(Exit::None as u64);
    };

    // get the key
    let [ko, kz, o] = [state.registers[8], state.registers[9], state.registers[10]];
    let mut input = codec::encode(&index).expect("should not fail");
    let shash = state
        .memory
        .read_bytes(ko as u32, (ko + kz) as u32)
        .expect("should not fail");
    input.extend_from_slice(&shash);

    // get the storage value
    let Some(value) = account.storage.get(&crypto::blake2b(&input)) else {
        return Ok(Exit::None as u64);
    };

    let vlen = value.len() as u64;
    let (from, to) = (state.registers[11].min(vlen), state.registers[12].min(vlen));
    state
        .memory
        .write_bytes(o as u32, &value[from as usize..to as usize])?;

    Ok(vlen)
}

/// (ΩW) storage write
fn write<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let mut general = data.as_general()?;

    // extract arguments from registers
    let [ko, kz, vo, vz] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    // get the key
    let mut input = codec::encode(&general.index).expect("should not fail");
    input.extend_from_slice(
        &state
            .memory
            .read_bytes(ko as u32, kz as u32)
            .expect("should not fail"),
    );
    let key = crypto::blake2b(&input);

    // update storage
    if vz == 0 {
        general.account.storage.remove(&key);
        data.update_general(general)?;
        Ok(Exit::None as u64)
    } else if let Ok(value) = state.memory.read_bytes(vo as u32, (vo + vz) as u32) {
        let account = general.account.state();
        if account.threshold() > account.balance {
            Ok(Exit::Full as u64)
        } else {
            general.account.storage.insert(key, value.clone());
            data.update_general(general)?;
            Ok(u64::decode(&value))
        }
    } else {
        crate::bail!("failed to upsert storage");
    }
}

/// (ΩS) sbrk - adjust program break
fn sbrk<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    let increment = state.registers[7] as i64;

    // TODO: For now, implement a simple heap that starts after the RW data region
    // The RO data starts at ZONE_SIZE (0x10000), and RW data starts at 2*ZONE_SIZE + funz(ro_len)
    // We'll start the heap at a safe address that doesn't conflict with pre-allocated regions
    // Using 0x100000 (1MB) as a safe starting point for dynamic heap allocation

    // Current break is stored in a fixed location for simplicity
    // In production, this would be tracked by the host system
    static mut CURRENT_BREAK: u64 = 0x100000; // Start heap at 1MB to avoid conflicts

    // sbrk(0) returns current break without changing it
    if increment == 0 {
        unsafe { return Ok(CURRENT_BREAK) };
    }

    // For positive increment, allocate memory
    if increment > 0 {
        unsafe {
            let old_break = CURRENT_BREAK;
            let new_break = old_break + increment as u64;

            // Allocate pages from old_break to new_break
            let page_size = 4096u32;
            let start_page = (old_break as u32) / page_size;
            let end_page = ((new_break - 1) as u32) / page_size;

            // For each page that needs to be allocated, add it to memory
            for page_num in start_page..=end_page {
                // Try to write to ensure the page exists and is writable
                // This will trigger page allocation if needed
                let page_addr = page_num * page_size;
                if let Err(e) = state.memory.write_bytes(page_addr, &[0]) {
                    // If allocation fails, return error
                    tracing::warn!("failed to write to page {page_addr}: {e}");
                    return Ok(Exit::OOB as u64);
                }
            }

            CURRENT_BREAK = new_break;
            return Ok(old_break);
        }
    }

    // For negative increment, deallocate memory
    if increment < 0 {
        unsafe {
            let old_break = CURRENT_BREAK;
            let new_break = old_break.saturating_sub((-increment) as u64);

            // Don't allow break to go below initial heap start
            if new_break < 0x100000 {
                return Ok(Exit::What as u64);
            }

            CURRENT_BREAK = new_break;
            return Ok(old_break);
        }
    }

    Ok(Exit::What as u64)
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
    .and_then(|account| codec::encode(&account.state()).ok()) else {
        return Ok(Exit::None as u64);
    };

    // write the account state to memory
    let address = state.registers[8];
    if let Err(reason) = state.memory.write_bytes(address as u32, &account) {
        crate::bail!("failed to write account state {reason}");
    }

    Ok(Exit::Ok as u64)
}
