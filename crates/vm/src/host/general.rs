//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    Argument, Result,
};
use account::Account;
use score::Parameters;

/// (ΩG) Get the gas to register
pub fn gas(ctx: &impl Argument) -> Result<u64> {
    Ok(ctx.gas())
}

// (ΩY) fetch the on chain parameters
pub fn fetch(ctx: &mut impl Argument) -> Result<ExitCode> {
    let kind = ctx.rget(10);
    let value: Vec<u8> = match kind {
        0 => codec::encode(&Parameters::default()),
        1 => codec::encode(&ctx.entropy()),
        8 => match ctx.auth_config() {
            Some(config) => config,
            None => return Ok(Exit::None as u64),
        },
        14 => codec::encode(&ctx.items()),
        15 => {
            let items = ctx.items();
            let index = ctx.rget(11);
            if let Some(item) = items.get(index as usize) {
                codec::encode(&item)
            } else {
                return Ok(Exit::None as u64);
            }
        }
        kind => {
            tracing::debug!("kind {kind} not supported");
            return Ok(Exit::None as u64);
        }
    };

    let vlen = value.len() as u64;
    let out = ctx.rget(7);
    let from = ctx.rget(8).min(vlen);
    let length = ctx.rget(9).min(vlen - from);
    if length > 0 {
        ctx.write(out as u32, &value[from as usize..(from + length) as usize])?;
    }

    Ok(vlen)
}

/// (ΩL) account lookup
pub fn lookup(ctx: &mut impl Argument) -> Result<u64> {
    let [acc, address, target, from, len] = [
        ctx.rget(7),
        ctx.rget(8),  // h
        ctx.rget(9),  // o
        ctx.rget(10), // f
        ctx.rget(11), // l
    ];
    let mut hash = [0u8; 32];
    ctx.read_into(address as u32, &mut hash)?;
    let Ok(account) = ctx.or_this(acc) else {
        return Ok(Exit::None as u64);
    };

    // get the preimage
    let preimage = {
        let Some(preimage) = account.preimage(hash) else {
            return Ok(Exit::None as u64);
        };

        preimage
    };

    // write partial preimage to memory (per graypaper: l = min(registers_11, len(v) - f))
    let plen = preimage.len() as u64;
    let from = from.min(plen);
    let len = len.min(plen.saturating_sub(from));
    if len > 0 {
        ctx.write(
            target as u32,
            &preimage[from as usize..(from + len) as usize],
        )?;
    }
    Ok(plen)
}

/// (ΩR) storage lookup
pub fn read(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [acc, ko, kz, o] = [ctx.rget(7), ctx.rget(8), ctx.rget(9), ctx.rget(10)];
    if kz > u32::MAX as u64 {
        crate::bail!("read: key size exceeds PVM address space");
    }
    crate::check_range(ko as u32, kz as u32)?;
    let mut key = vec![0u8; kz as usize];
    ctx.read_into(ko as u32, &mut key)?;

    // get the account
    let Ok(account) = ctx.or_this(acc) else {
        return Ok(Exit::None as u64);
    };

    // get the storage value
    let Some(value) = account.read(&key) else {
        return Ok(Exit::None as u64);
    };

    let vlen = value.len() as u64;
    let from = ctx.rget(11).min(vlen);
    let length = ctx.rget(12).min(vlen - from);
    if length > 0 {
        ctx.write(o as u32, &value[from as usize..(from + length) as usize])?;
    }
    Ok(vlen)
}

/// (ΩW) storage write
pub fn write(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [ko, kz, vo, vz] = [ctx.rget(7), ctx.rget(8), ctx.rget(9), ctx.rget(10)];
    if vz > u32::MAX as u64 || kz > u32::MAX as u64 {
        crate::bail!("write: size exceeds PVM address space");
    }
    crate::check_range(vo as u32, vz as u32)?;
    crate::check_range(ko as u32, kz as u32)?;
    let mut value = vec![0u8; vz as usize];
    ctx.read_into(vo as u32, &mut value)?;
    let mut key = vec![0u8; kz as usize];
    ctx.read_into(ko as u32, &mut key)?;

    // check if the account has enough balance to cover the threshold
    let account = ctx.this()?;
    let prev = account.read(&key);

    // Removing a key that doesn't exist is a no-op; skip the threshold check.
    if vz == 0 && prev.is_none() {
        return Ok(Exit::None as u64);
    }

    let threshold = account
        .write_threshold(&key, &value, prev.as_deref())
        .unwrap_or(u64::MAX);

    if threshold > account.balance() {
        return Ok(Exit::Full as u64);
    }

    // update storage
    let result = if let Some(prev) = prev {
        prev.len() as u64
    } else {
        Exit::None as u64
    };

    if vz == 0 {
        let Some(_value) = account.remove(&key) else {
            return Ok(Exit::None as u64);
        };
    } else {
        account.write(&key, value);
    }

    Ok(result)
}

/// (ΩI) fetch account info
pub fn info(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [acc, output] = [ctx.rget(7), ctx.rget(8)];
    let Ok(account) = ctx.or_this(acc) else {
        return Ok(Exit::None as u64);
    };

    let info = account.info().vm();
    let info = codec::encode(&info);

    // Get memory write parameters from registers
    let tlen = info.len() as u64;
    let from = ctx.rget(9).min(tlen) as usize;
    let length = ctx.rget(10).min(tlen - from as u64) as usize;
    if from < tlen as usize {
        ctx.write(output as u32, &info[from..from + length])?;
    }

    // Return total length of encoded data
    Ok(tlen)
}
