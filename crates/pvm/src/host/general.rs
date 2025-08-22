//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    invocation::State,
    Argument, Result,
};
use score::{Account, Parameters, ServiceId};

/// (ΩG) Get the gas to register
pub fn gas(state: &mut State) -> Result<u64> {
    Ok(state.gas as u64)
}

/// (ΩL) account lookup
pub fn lookup(ctx: &mut impl Argument, state: &mut State) -> Result<u64> {
    let Ok(account) = ctx.or_this(state.registers[7]) else {
        return Ok(Exit::None as u64);
    };

    // get the preimage
    let preimage = {
        let address = state.registers[8];

        // get the preimage hash
        let phash = state.memory.read_bytes(address as u32, 32)?;
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&phash);

        let Some(preimage) = account.preimage(hash) else {
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
pub fn read(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    // get the account
    let Ok(account) = ctx.or_this(state.registers[7]) else {
        return Ok(Exit::None as u64);
    };

    tracing::debug!("reading from account {}", account.index());

    // get the key
    let [ko, kz, o] = [state.registers[8], state.registers[9], state.registers[10]];
    let key = state
        .memory
        .read_bytes(ko as u32, kz as u32)
        .expect("should not fail");

    // get the storage value
    let Some(value) = account.read(&key) else {
        return Ok(Exit::None as u64);
    };

    let vlen = value.len() as u64;
    let from = state.registers[11].min(vlen);
    let length = state.registers[12].min(vlen - from);
    if length > 0 {
        state
            .memory
            .write_bytes(o as u32, &value[from as usize..(from + length) as usize])?;
    }
    Ok(vlen)
}

/// (ΩW) storage write
pub fn write(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
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
            crate::bail!("Failed to read key bytes: {:?}", err);
        }
    };

    // check if the account has enough balance to cover the threshold
    let account = ctx.this()?;
    if account.threshold() > account.balance() {
        return Ok(Exit::Full as u64);
    }

    // update storage
    let result = if let Some(prev) = account.read(&key) {
        prev.len() as u64
    } else {
        Exit::None as u64
    };

    if vz == 0 {
        let Some(_value) = account.remove(&key) else {
            return Ok(Exit::None as u64);
        };
    } else {
        let value = match state.memory.read_bytes(vo as u32, vz as u32) {
            Ok(bytes) => bytes,
            Err(err) => {
                crate::bail!("Failed to read value bytes: {:?}", err);
            }
        };

        // TODO: we actually can update the key here for avoiding hashing for twice
        account.write(&key, value);
    }

    Ok(result)
}

/// (ΩI) fetch info
///
/// fetch state of the account per Gray Paper specification
pub fn info(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    // Get service ID from register 7 (u64::MAX means current service)
    let r7 = state.registers[7];
    let service_id = if r7 == u64::MAX {
        ctx.service()
    } else {
        r7 as ServiceId
    };

    // Get the account or return NONE if not found
    let Ok(account) = ctx.account(service_id as u64) else {
        return Ok(Exit::None as u64);
    };

    let Ok(info) = account.info().host(service_id == ctx.service()) else {
        crate::bail!("failed to encode account info");
    };

    // Get memory write parameters from registers
    let total_len = info.len() as u64;
    let output = state.registers[8] as u32;
    let from = state.registers[9].min(total_len) as usize;
    let length = state.registers[10].min(total_len - from as u64) as usize;
    if length > 0 {
        state
            .memory
            .write_bytes(output, &info[from..(from + length)])?;
    }

    // Return total length of encoded data
    Ok(total_len)
}

// (ΩY) fetch the on chain parameters
pub fn fetch(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let value: Vec<u8> = match state.registers[10] {
        0 => codec::encode(&Parameters::default()).expect("should not fail"),
        1 => codec::encode(&ctx.entropy()).expect("should not fail"),
        14 => codec::encode(&ctx.operands()).expect("should not fail"),
        kind => {
            tracing::warn!("kind {kind} not supported");
            return Ok(Exit::None as u64);
        }
    };

    let vlen = value.len() as u64;
    let out = state.registers[7];
    let from = state.registers[8].min(vlen);
    let length = state.registers[9].min(vlen - from);
    if length > 0 {
        state
            .memory
            .write_bytes(out as u32, &value[from as usize..(from + length) as usize])?;
    }

    Ok(vlen)
}
