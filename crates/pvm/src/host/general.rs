//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    Argument, Reason, Result, State,
};
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
        .read_bytes(ko as u32, kz as u32)
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
    let key_bytes = match state.memory.read_bytes(ko as u32, kz as u32) {
        Ok(bytes) => bytes,
        Err(err) => {
            tracing::error!("Failed to read key bytes: {:?}", err);
            return Ok(Exit::OOB as u64);
        }
    };

    // get the key by hashing account index + key bytes
    let mut input = codec::encode(&general.index).expect("should not fail");
    input.extend_from_slice(&key_bytes);
    let key = crypto::blake2b(&input);

    // update storage
    if vz == 0 {
        general.account.storage.remove(&key);
        data.update_general(general)?;
        Ok(Exit::None as u64)
    } else {
        let value = match state.memory.read_bytes(vo as u32, vz as u32) {
            Ok(bytes) => bytes,
            Err(err) => {
                tracing::error!("Failed to read value bytes: {:?}", err);
                return Ok(Exit::OOB as u64);
            }
        };

        let account = general.account.state();
        if account.threshold() > account.balance {
            Ok(Exit::Full as u64)
        } else {
            general.account.storage.insert(key, value.clone());
            data.update_general(general)?;
            Ok(Exit::Ok as u64)
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

/// (ΩS) sbrk - adjust program break
fn sbrk<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    // Get requested heap increment from A0 register (register 7)
    let value_a = state.registers[7] as u32;

    // Initialize heap pointer if not already done
    let current_heap = if let Some(heap_ptr) = state.memory.get_heap_pointer() {
        heap_ptr
    } else {
        let initial_heap = state.memory.initial_heap();
        state.memory.set_heap_pointer(initial_heap);
        initial_heap
    };

    // If valueA is 0, just return the current heap pointer
    if value_a == 0 {
        state.registers[7] = current_heap as u64;
        return Ok(Exit::Ok as u64);
    }

    // Record old heap pointer to return and calculate new heap pointer
    let old_heap_pointer = current_heap;
    let new_heap_pointer = old_heap_pointer + value_a;

    const MEM_PAGE_SIZE: u32 = 4096;

    // Calculate the next page boundary and final boundary after allocation
    let next_page_boundary =
        ((old_heap_pointer + MEM_PAGE_SIZE - 1) / MEM_PAGE_SIZE) * MEM_PAGE_SIZE;

    // Allocate pages if we cross a page boundary
    if new_heap_pointer > next_page_boundary {
        let final_boundary =
            ((new_heap_pointer + MEM_PAGE_SIZE - 1) / MEM_PAGE_SIZE) * MEM_PAGE_SIZE;
        let start_page = next_page_boundary / MEM_PAGE_SIZE;
        let end_page = final_boundary / MEM_PAGE_SIZE;

        // Allocate all pages in the range
        for page_num in start_page..end_page {
            match state.memory.allocate_page(page_num) {
                Ok(_) => tracing::debug!("allocated page {}", page_num),
                Err(reason) => {
                    tracing::warn!("failed to allocate page {}: {:?}", page_num, reason);
                    return Err(reason);
                }
            }
        }
    }

    // Set the new heap pointer and return the old one
    state.memory.set_heap_pointer(new_heap_pointer);
    state.registers[7] = old_heap_pointer as u64;
    Ok(Exit::Ok as u64)
}
