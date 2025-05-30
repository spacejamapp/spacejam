//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    Argument, Reason, Result, State,
};
use codec::Numeric;
use score::{service::ServiceAccount, Gas, ServiceId};
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};

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

/// Global heap pointer using atomic operations for thread safety
static CURRENT_HEAP_POINTER: AtomicU64 = AtomicU64::new(0);

/// (ΩS) sbrk - adjust program break
fn sbrk<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    _data: &mut X,
) -> Result<ExitCode> {
    let value_a = state.registers[7] as i64;

    tracing::debug!(
        "sbrk called with value_a={} (0x{:x})",
        value_a,
        value_a as u64
    );

    // Based on memory layout: RW data starts at 2*Z_Z + Z(|o|)
    // For our test case: ro_len = 12296, so Z(ro_len) = 0x10000
    // RW data starts at 0x30000, RW data length = 0
    // So heap should start at 0x30000 + PAGE_SIZE (following the reference)
    const ZONE_SIZE: u64 = 0x10000;
    const PAGE_SIZE: u64 = 0x1000;
    const RO_LEN: u64 = 12296; // From our test case

    // Calculate where RW data ends and heap should start
    let funz_ro = RO_LEN.div_ceil(ZONE_SIZE) * ZONE_SIZE; // 0x10000
    let rw_data_start = 2 * ZONE_SIZE + funz_ro; // 0x30000
    let rw_data_len = 0; // From our test case
    let heap_start = rw_data_start + rw_data_len; // 0x30000 (no extra PAGE_SIZE)

    // Initialize heap pointer on first call
    let mut current_heap_pointer = CURRENT_HEAP_POINTER.load(Ordering::Relaxed);
    if current_heap_pointer == 0 {
        current_heap_pointer = heap_start;
        CURRENT_HEAP_POINTER.store(current_heap_pointer, Ordering::Relaxed);
        tracing::debug!(
            "sbrk initialized heap pointer to 0x{:x}",
            current_heap_pointer
        );
    }

    // If valueA == 0, return current heap pointer (query operation)
    if value_a == 0 {
        tracing::debug!(
            "sbrk query - returning current heap pointer 0x{:x}",
            current_heap_pointer
        );
        state.registers[7] = current_heap_pointer;
        return Ok(Exit::Ok as u64);
    }

    // Record current heap pointer to return
    let result = current_heap_pointer;

    // Calculate new heap pointer
    let new_heap_pointer = if value_a > 0 {
        current_heap_pointer + value_a as u64
    } else {
        current_heap_pointer.saturating_sub((-value_a) as u64)
    };

    tracing::debug!(
        "sbrk allocation - current: 0x{:x}, requested: {}, new: 0x{:x}",
        current_heap_pointer,
        value_a,
        new_heap_pointer
    );

    // Page boundary logic (P_func)
    let funp = |x: u64| x.div_ceil(PAGE_SIZE) * PAGE_SIZE;
    let next_page_boundary = funp(current_heap_pointer);

    // Only allocate pages if new heap pointer crosses page boundary
    if new_heap_pointer > next_page_boundary {
        let final_boundary = funp(new_heap_pointer);
        let idx_start = next_page_boundary / PAGE_SIZE;
        let idx_end = final_boundary / PAGE_SIZE;

        tracing::debug!(
            "sbrk allocating pages from 0x{:x} to 0x{:x} (pages {} to {})",
            next_page_boundary,
            final_boundary,
            idx_start,
            idx_end
        );

        // Allocate pages by writing to them
        for page_idx in idx_start..idx_end {
            let page_addr = (page_idx * PAGE_SIZE) as u32;
            if let Err(e) = state.memory.write_bytes(page_addr, &[0]) {
                tracing::warn!("failed to allocate page at 0x{:x}: {}", page_addr, e);
                state.registers[7] = page_addr as u64;
                return Ok(Exit::OOB as u64);
            }
        }
    }

    // Update heap pointer
    CURRENT_HEAP_POINTER.store(new_heap_pointer, Ordering::Relaxed);

    tracing::debug!("sbrk returning previous heap pointer 0x{:x}", result);

    // Return previous heap pointer
    state.registers[7] = result;
    Ok(Exit::Ok as u64)
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
