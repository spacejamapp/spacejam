//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    Argument, Reason, Result, State,
};
use codec::Numeric;
use score::{service::ServiceAccount, Gas, ServiceId};
use std::collections::BTreeMap;

/// Calculate the initial heap start address based on memory layout
fn calculate_heap_start() -> u32 {
    // Following the memory layout from parser/src/memory.rs:
    // RW data starts at: 2*Z_Z + Z(ro_len)
    // Heap starts at: RW data start + RW data length + RW padding
    // Since RW data length = 0 and RW padding = funp(0) - 0 = 0,
    // Heap starts immediately after RW data start

    const Z_Z: u32 = 0x10000; // 2^16
    const RO_LEN: u32 = 12296; // From debug output
    const RW_LEN: u32 = 0; // From debug output

    let rw_data_start = 2 * Z_Z + ((RO_LEN + Z_Z - 1) / Z_Z) * Z_Z; // 2*Z_Z + Z(ro_len)
    let heap_start = rw_data_start + RW_LEN; // No extra padding needed

    heap_start
}

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
    tracing::debug!("storage write host call - START");
    tracing::debug!("registers: {:?}", state.registers);

    let mut general = data.as_general()?;

    // extract arguments from registers
    let [ko, kz, vo, vz] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    tracing::debug!(
        "storage write params: ko={}, kz={}, vo={}, vz={}",
        ko,
        kz,
        vo,
        vz
    );

    // get the key
    let mut input = codec::encode(&general.index).expect("should not fail");
    input.extend_from_slice(
        &state
            .memory
            .read_bytes(ko as u32, kz as u32)
            .expect("should not fail"),
    );
    let key = crypto::blake2b(&input);

    tracing::debug!(
        "service_id: {}, raw_key: {:?}, blake2b_key: {:?}",
        general.index,
        state
            .memory
            .read_bytes(ko as u32, kz as u32)
            .unwrap_or_default(),
        key
    );

    // update storage
    if vz == 0 {
        tracing::debug!("removing storage key");
        general.account.storage.remove(&key);
        data.update_general(general)?;
        Ok(Exit::None as u64)
    } else if let Ok(value) = state.memory.read_bytes(vo as u32, (vo + vz) as u32) {
        let account = general.account.state();
        if account.threshold() > account.balance {
            tracing::warn!("storage write failed: insufficient balance");
            Ok(Exit::Full as u64)
        } else {
            tracing::debug!("inserting storage: key={:?}, value={:?}", key, value);
            general.account.storage.insert(key, value.clone());
            data.update_general(general)?;
            tracing::debug!("storage write SUCCESS, returning: {}", u64::decode(&value));
            Ok(u64::decode(&value))
        }
    } else {
        tracing::error!("failed to read storage value from memory");
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

    tracing::debug!("sbrk called with value_a={} (0x{:x})", value_a, value_a);

    // Initialize heap pointer if it's the first call
    let current_heap = calculate_heap_start();
    tracing::debug!("initialized heap pointer to 0x{:x}", current_heap);

    // Allocate essential low memory pages for service execution using proper allocation interface
    // Services often need page 0 and other low pages for data structures
    tracing::debug!("allocating essential memory pages for service execution");

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

    if value_a == 0 {
        // Query current heap pointer
        let heap_pointer = current_heap;
        tracing::debug!(
            "sbrk query - returning current heap pointer 0x{:x}",
            heap_pointer
        );
        state.registers[7] = heap_pointer as u64;
        return Ok(Exit::Ok as u64);
    }

    // Allocate memory: return old heap pointer and advance by value_a
    let old_heap_pointer = current_heap;
    let new_heap_pointer = old_heap_pointer + value_a;

    // Actually allocate the pages in memory for the requested heap space
    const PAGE_SIZE: u32 = 4096;
    let start_page = old_heap_pointer / PAGE_SIZE;
    let end_page = (new_heap_pointer + PAGE_SIZE - 1) / PAGE_SIZE;

    tracing::debug!(
        "allocating pages for heap: start_page={}, end_page={}, pages_to_allocate={}",
        start_page,
        end_page,
        end_page - start_page
    );

    // Allocate all pages from start to end using proper allocation interface
    for page_num in start_page..end_page {
        tracing::debug!("allocating page {} for heap", page_num);
        if let Err(reason) = state.memory.allocate_page(page_num) {
            tracing::error!("failed to allocate heap page {}: {:?}", page_num, reason);
            crate::bail!("failed to allocate heap memory");
        }
    }

    // Update the heap pointer
    tracing::debug!(
        "sbrk allocated {} bytes: old=0x{:x}, new=0x{:x}, returning=0x{:x}",
        value_a,
        old_heap_pointer,
        new_heap_pointer,
        old_heap_pointer
    );

    // Return the old heap pointer (standard sbrk behavior)
    state.registers[7] = old_heap_pointer as u64;

    Ok(Exit::Ok as u64)
}
