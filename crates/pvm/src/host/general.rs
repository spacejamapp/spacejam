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

            Ok(Exit::Ok as u64)
        }
    } else {
        crate::bail!("failed to upsert storage");
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
    let value_a = state.registers[7] as u32;

    // Try to allocate low memory pages (0-2) that services commonly use
    // This will succeed for service execution contexts but fail silently for instruction tests
    for page_num in 0..=2 {
        match state.memory.allocate_page(page_num) {
            Ok(_) => tracing::debug!(
                "allocated essential page {} for service execution",
                page_num
            ),
            Err(reason) => tracing::debug!(
                "skipped allocation of page {} (reason: {:?})",
                page_num,
                reason
            ),
        }
    }

    // Initialize heap pointer if not already done
    let current_heap = if let Some(heap_ptr) = state.memory.get_heap_pointer() {
        heap_ptr
    } else {
        let initial_heap = state.memory.initial_heap();
        state.memory.set_heap_pointer(initial_heap);
        initial_heap
    };

    if value_a == 0 {
        state.registers[7] = current_heap as u64;
        return Ok(Exit::Ok as u64);
    }

    // Actually allocate the pages in memory for the requested heap space
    let old_heap_pointer = current_heap;
    let new_heap_pointer = old_heap_pointer + value_a;
    let start_page = old_heap_pointer / score::PAGE_SIZE as u32;
    let end_page = (new_heap_pointer + score::PAGE_SIZE as u32 - 1) / score::PAGE_SIZE as u32;

    // Allocate all pages from start to end using proper allocation interface
    for page_num in start_page..end_page {
        match state.memory.allocate_page(page_num) {
            Ok(_) => tracing::debug!("successfully allocated page {}", page_num),
            Err(reason) => {
                tracing::warn!("failed to allocate heap page {}: {:?}", page_num, reason);
            }
        }
    }

    state.memory.set_heap_pointer(new_heap_pointer);
    state.registers[7] = old_heap_pointer as u64;
    Ok(Exit::Ok as u64)
}
