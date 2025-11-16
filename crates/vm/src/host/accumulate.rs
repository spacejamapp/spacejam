//! Accumulation related host calls

use crate::{
    host::{Exit, ExitCode},
    Argument, Result,
};
use account::Account;
use score::{
    safrole::ValidatorData,
    service::{Privileges, ServiceAccount, ServiceInfo},
    vm::DeferredTransfer,
};
use std::collections::BTreeMap;

/// (ΩB) bless
pub fn bless(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [bless, assign, designate, register, acc, entries] = [
        ctx.rget(7),  // m: bless service id
        ctx.rget(8),  // a: memory address of assign array
        ctx.rget(9),  // v: designate service id
        ctx.rget(10), // r: register service id
        ctx.rget(11), // o: memory address of always_acc map
        ctx.rget(12), // n: count of always_acc entries
    ];

    // (a) Read assign array from memory
    let assign = {
        let size = 4 * score::CORES_COUNT as u32;
        let data = ctx.read(assign as u32, size)?;
        let mut assign = [0u32; score::CORES_COUNT];
        for (i, chunk) in data.chunks(4).enumerate() {
            if i < score::CORES_COUNT {
                assign[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            }
        }

        assign
    };

    // (z) Read always accumulate map from memory
    let mut always_acc = BTreeMap::new();
    if entries > 0 {
        let source = ctx.read(acc as u32, (12 * entries) as u32)?;
        for chunk in source.chunks(12) {
            let service_id = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let gas_allowance = u64::from_le_bytes([
                chunk[4], chunk[5], chunk[6], chunk[7], chunk[8], chunk[9], chunk[10], chunk[11],
            ]);
            always_acc.insert(service_id, gas_allowance);
        }
    }

    // (m, v, r) Check if bless and designate are valid service IDs
    if [bless, designate, register]
        .iter()
        .any(|&id| (id != 0 && id < score::MINIMUM_SERVICE_ID as u64) || id > u32::MAX as u64)
    {
        return Ok(Exit::Who as u64);
    }

    // Update privileges: tuple{m, 𝐚, v, 𝐳}
    ctx.set_privileges(Privileges {
        bless: bless as u32,
        register: register as u32,
        assign,
        designate: designate as u32,
        always_acc,
    });

    Ok(Exit::Ok as u64)
}

/// (ΩA) assign authorization queue
pub fn assign(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [core, o, assign] = [ctx.rget(7), ctx.rget(8), ctx.rget(9)];
    let source = ctx.read(o as u32, (12 * score::QUEUE_ITEMS) as u32)?;

    // return if invalid core index
    if core > score::CORES_COUNT as u64 {
        return Ok(Exit::Core as u64);
    }

    // check if the service is a core
    let privileges = ctx.privileges();
    if ctx.service() != privileges.assign[core as usize] {
        return Ok(Exit::Huh as u64);
    }

    // parse the authorization queue
    let queue: Vec<[u8; 32]> = source
        .chunks(32)
        .map(|chunk| {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(chunk);
            hash
        })
        .collect();

    // set the authorization queue
    ctx.set_authorization(core as u16, queue);
    ctx.set_assign(core as u16, assign as u32);
    Ok(Exit::Ok as u64)
}

/// (ΩD) designate the validators to be drawn for the next epoch
pub fn designate(ctx: &mut impl Argument) -> Result<ExitCode> {
    // get the data source
    let o = ctx.rget(7);
    let source = ctx.read(o as u32, 336 * score::VALIDATORS_COUNT as u32)?;

    let privileges = ctx.privileges();
    if ctx.service() != privileges.designate {
        return Ok(Exit::Huh as u64);
    }

    let validators = {
        if source.len() != 336 * score::VALIDATORS_COUNT as usize {
            crate::bail!(
                "Invalid encoded validators, expected length: {}, got: {}",
                336 * score::VALIDATORS_COUNT as usize,
                source.len()
            );
        }

        let mut validators = [ValidatorData::default(); score::VALIDATORS_COUNT as usize];
        for (i, chunk) in source.chunks(336).enumerate() {
            let Ok(validator) = codec::decode(chunk) else {
                crate::bail!("Could not parse validators");
            };
            validators[i] = validator;
        }
        validators
    };

    // set the validators
    ctx.set_validators(validators);
    Ok(Exit::Ok as u64)
}

/// (ΩC) checkpoint
pub fn checkpoint(ctx: &mut impl Argument) -> Result<ExitCode> {
    ctx.checkpoint();
    Ok(ctx.gas())
}

/// (ΩN) new
pub fn new_(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [o, length, accumulate_gas, transfer_gas, f] = [
        ctx.rget(7),
        ctx.rget(8),
        ctx.rget(9),
        ctx.rget(10),
        ctx.rget(11),
    ];

    // check if the service is blessed
    let privileges = ctx.privileges();
    if f != 0 && privileges.bless != ctx.service() {
        return Ok(Exit::Huh as u64);
    }

    // get the account code
    let code = ctx.read_hash(o as u32)?;
    if length > u32::MAX as u64 {
        crate::bail!("Invalid length");
    }

    // create a new account with proper storage accounting
    let index = ctx.index();
    let mut created = ServiceAccount {
        index,
        info: ServiceInfo {
            code,
            balance: score::BALANCE_PER_SERVICE,
            accumulate: accumulate_gas,
            transfer: transfer_gas,
            creation: ctx.timeslot(),
            update: 0,
            parent: ctx.service(),
            ..Default::default()
        },
        ..Default::default()
    };
    created.insert_lookup(code, length as u32, vec![]);

    // Calculate the full threshold cost
    let new_account_threshold = created.threshold();
    let service = ctx.this()?;
    if service.balance() < service.threshold() + new_account_threshold {
        return Ok(Exit::Cash as u64);
    }

    // Deduct full threshold from parent and give it to new account
    *service.balance_mut() -= new_account_threshold;
    created.info.balance = new_account_threshold;
    ctx.upsert(index, created);

    // Check and find a free account index
    let base =
        score::MINIMUM_SERVICE_ID + (index - score::MINIMUM_SERVICE_ID + 42) % score::CHECK_SALT;
    let new_index = ctx.check(base);
    ctx.set_index(new_index);
    Ok(index as u64)
}

/// (ΩU) upgrade service code
pub fn upgrade(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [o, g, m] = [ctx.rget(7), ctx.rget(8), ctx.rget(9)];
    let code = ctx.read_hash(o as u32)?;
    let account = ctx.this()?;
    account.set_code(code);
    account.set_transfer_gas(m);
    account.set_accumulate_gas(g);
    Ok(Exit::Ok as u64)
}

/// (ΩT) transfer funds from the sender to the destination
pub fn transfer(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [dest, amount, limit, memo] = [ctx.rget(7), ctx.rget(8), ctx.rget(9), ctx.rget(10)];

    // check if the defer transfer is valid
    let memo = {
        let bytes = ctx.read(memo as u32, score::TRANSFER_MEMO_SIZE)?;
        let mut memo = [0u8; score::TRANSFER_MEMO_SIZE as usize];
        memo[..bytes.len()].copy_from_slice(&bytes);
        memo
    };
    let service = ctx.service();

    // check if the recipient exists
    if ctx.account(dest).is_err() {
        return Ok(Exit::Who as u64);
    }

    // check if the sender has enough balance
    let sender = ctx.this()?;
    let balance = sender.balance();
    if balance.saturating_sub(amount) < sender.threshold() {
        return Ok(Exit::Cash as u64);
    }

    // check if the recipient has enough transfer gas
    let recipient = ctx.account(dest)?;
    if limit < recipient.transfer_gas() {
        return Ok(Exit::Low as u64);
    }

    // add the transfer to the deferred transfers
    let transfer = DeferredTransfer {
        sender: service,
        recipient: dest as u32,
        amount,
        memo,
        gas_limit: limit,
    };
    ctx.transfer(transfer);
    *ctx.this()?.balance_mut() -= amount;
    Ok(Exit::Ok as u64)
}

/// (ΩE) eject a sub account
pub fn eject(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [dest, o] = [ctx.rget(7), ctx.rget(8)];
    let hash = ctx.read_hash(o as u32)?;
    if dest == ctx.service() as u64 {
        crate::bail!("cannot eject to self");
    }

    let service = ctx.service();
    let timeslot = ctx.timeslot();
    let Ok(dest) = ctx.account(dest) else {
        tracing::debug!("failed to eject: account not found");
        return Ok(Exit::Who as u64);
    };

    // check if the code is valid
    let mut code = [0; 32];
    code[..4].copy_from_slice(&service.to_le_bytes());
    if dest.code() != code {
        tracing::debug!("failed to eject: code mismatch");
        return Ok(Exit::Who as u64);
    }

    // check if the look up is valid
    if dest.items() != 2 {
        tracing::debug!("failed to eject: items mismatch");
        return Ok(Exit::Huh as u64);
    }
    let length = dest.total().saturating_sub(81);
    let Some(lookup) = dest.lookup(hash, length as u32) else {
        tracing::debug!("failed to eject: lookup not found");
        return Ok(Exit::Huh as u64);
    };

    // remove account and add the balance to the parent account
    if lookup.len() == 2 && lookup[1] < timeslot.saturating_sub(score::EXPUNGED_TIME) {
        let balance = dest.balance();
        let to_remove = dest.index();
        let _ = dest;
        tracing::debug!("eject: balance={balance}, to_remove={to_remove}");
        *ctx.this()?.balance_mut() += balance;
        ctx.remove(to_remove);
        return Ok(Exit::Ok as u64);
    }

    tracing::debug!("failed to eject: lookup not expired");
    Ok(Exit::Huh as u64)
}

/// (ΩQ) query an lookup entry
pub fn query(ctx: &mut impl Argument) -> Result<ExitCode> {
    let (o, z) = (ctx.rget(7) as u32, ctx.rget(8) as u32);
    let hash = ctx.read_hash(o)?;
    let account = ctx.this()?;
    let Some(lookup) = account.lookup(hash, z) else {
        ctx.rset(8, 0);
        return Ok(Exit::None as u64);
    };

    // update result
    let base = 1u64 << 32;
    let exit = if lookup.is_empty() {
        ctx.rset(8, 0);
        0
    } else if lookup.len() == 1 {
        ctx.rset(8, 0);
        1 + base * lookup[0] as u64
    } else if lookup.len() == 2 {
        ctx.rset(8, lookup[1] as u64);
        2 + base * lookup[0] as u64
    } else {
        let reg8_val = lookup[1] as u64 + base * lookup[2] as u64;
        ctx.rset(8, reg8_val);
        3 + base * lookup[0] as u64
    };
    Ok(exit)
}

/// (ΩS) solicit new lookup
pub fn solicit(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [o, z] = [ctx.rget(7), ctx.rget(8)];
    let hash = ctx.read_hash(o as u32)?;

    // check if the account has enough balance
    let timeslot = ctx.timeslot();
    let account = ctx.this()?;
    let mut slots = vec![];
    if let Some(lookup) = account.lookup(hash, z as u32) {
        if lookup.len() == 2 {
            slots = vec![lookup[0], lookup[1], timeslot];
        } else {
            return Ok(Exit::Huh as u64);
        }
    }

    // pre-calculate the new threshold
    let threshold = account.lookup_threshold(z).unwrap_or(u64::MAX);
    if account.balance() < threshold {
        return Ok(Exit::Full as u64);
    }

    account.insert_lookup(hash, z as u32, slots);
    Ok(Exit::Ok as u64)
}

/// (ΩF) forget
pub fn forget(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [o, z] = [ctx.rget(7), ctx.rget(8)];
    let hash = ctx.read_hash(o as u32)?;

    // get the lookup data
    let timeslot = ctx.timeslot();
    let account = ctx.this()?;
    let Some(mut lookup) = account.lookup(hash, z as u32) else {
        return Ok(Exit::Huh as u64);
    };

    let expunged = timeslot.saturating_sub(score::EXPUNGED_TIME);
    if lookup.is_empty() || (lookup.len() == 2 && lookup[1] < expunged) {
        account.remove_lookup(hash, z as u32);
        account.remove_preimage(hash);
    } else if lookup.len() == 1 {
        lookup.push(timeslot);
        account.insert_lookup(hash, z as u32, lookup);
    } else if lookup.len() == 3 && lookup[1] < expunged {
        account.insert_lookup(hash, z as u32, vec![lookup[2], timeslot]);
    } else {
        return Ok(Exit::Huh as u64);
    }

    Ok(Exit::Ok as u64)
}

/// (ΩY) yield
pub fn yield_(ctx: &mut impl Argument) -> Result<ExitCode> {
    let o = ctx.rget(7);
    let hash = ctx.read_hash(o as u32)?;
    ctx.output(hash);
    Ok(Exit::Ok as u64)
}

/// (ΩP) provide new preimage
pub fn provide(ctx: &mut impl Argument) -> Result<ExitCode> {
    let [mut service, from, size] = [ctx.rget(7), ctx.rget(8), ctx.rget(9)];
    let timeslot = ctx.timeslot();
    if service == u64::MAX {
        service = ctx.service() as u64;
    }

    let preimage = ctx.read(from as u32, size as u32)?;
    let Ok(account) = ctx.account(service) else {
        return Ok(Exit::Who as u64);
    };

    // check if the preimage is already in the account
    let hash = crypto::blake2b(&preimage);
    if account.lookup(hash, size as u32) != Some(vec![]) {
        return Ok(Exit::Huh as u64);
    }

    // check if the preimage is already in the account
    if account.preimage(hash).is_some() {
        return Ok(Exit::Huh as u64);
    }

    // FIXME: the lookup insert is not specified in graypper, could be a bug
    // in the fuzzy tests or our implementation.
    account.insert_lookup(hash, size as u32, vec![timeslot]);
    account.insert_preimage(hash, preimage);
    Ok(Exit::Ok as u64)
}
