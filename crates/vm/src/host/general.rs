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
        0 => codec::encode(&Parameters::default()).expect("should not fail"),
        1 => codec::encode(&ctx.entropy()).expect("should not fail"),
        14 => codec::encode(&ctx.operands()).expect("should not fail"),
        15 => {
            let operands = ctx.operands();
            let index = ctx.rget(11);
            if let Some(operand) = operands.get(index as usize) {
                codec::encode(operand).expect("should not fail")
            } else {
                Default::default()
            }
        }
        kind => {
            tracing::warn!("kind {kind} not supported");
            return Ok(Exit::None as u64);
        }
    };

    tracing::debug!("fetch kind: {:?}", kind);

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
    let [acc, address, target, from, to] = [
        ctx.rget(7),
        ctx.rget(8),
        ctx.rget(9),
        ctx.rget(10),
        ctx.rget(11),
    ];
    let phash = ctx.read(address as u32, 32)?;
    let Ok(account) = ctx.or_this(acc) else {
        return Ok(Exit::None as u64);
    };

    // get the preimage
    let preimage = {
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&phash);
        let Some(preimage) = account.preimage(hash) else {
            return Ok(Exit::None as u64);
        };

        preimage
    };

    // write patrial preimage to memory
    let plen = preimage.len() as u64;
    let (from, to) = (from.min(plen), to.min(plen));
    ctx.write(target as u32, &preimage[from as usize..to as usize])?;
    Ok(plen)
}

/// (ΩR) storage lookup
pub fn read(ctx: &mut impl Argument) -> Result<ExitCode> {
    // get the key
    let [acc, ko, kz, o] = [ctx.rget(7), ctx.rget(8), ctx.rget(9), ctx.rget(10)];
    let key = ctx.read(ko as u32, kz as u32)?;

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
    let value = ctx.read(vo as u32, vz as u32)?;
    let key = ctx.read(ko as u32, kz as u32)?;

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
        // TODO: we actually can update the key here for avoiding hashing for twice
        account.write(&key, value);
    }

    Ok(result)
}

/// (ΩI) fetch account info
pub fn info(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [acc, output, from, to] = [ctx.rget(7), ctx.rget(8), ctx.rget(9), ctx.rget(10)];
    let Ok(account) = ctx.or_this(acc) else {
        return Ok(Exit::None as u64);
    };

    let Ok(info) = account.info().host() else {
        crate::bail!("failed to encode account info");
    };

    // Get memory write parameters from registers
    let total_len = info.len() as u64;
    let (from, to) = (from.min(total_len) as usize, to.min(total_len) as usize);
    if to > from {
        ctx.write(output as u32, &info[from..to])?;
    }

    // Return total length of encoded data
    Ok(total_len)
}
