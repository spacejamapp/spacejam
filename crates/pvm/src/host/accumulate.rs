//! Accumulation related host calls

use crate::{
    host::{Exit, ExitCode},
    invocation::State,
    Argument, Result,
};
use score::{
    safrole::ValidatorData,
    service::{Privileges, ServiceAccount, ServiceInfo},
    vm::DeferredTransfer,
    Account,
};
use std::collections::BTreeMap;

/// (ΩB) bless
pub fn bless(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [bless, assign, designate, acc, entries] = [
        state.registers[7],  // m: bless service id
        state.registers[8],  // a: memory address of assign array
        state.registers[9],  // v: designate service id
        state.registers[10], // o: memory address of always_acc map
        state.registers[11], // n: count of always_acc entries
    ];

    // Check if current service is the blessed service
    let privileges = ctx.privileges();
    if ctx.service() != privileges.bless {
        return Ok(Exit::Huh as u64);
    }

    // Check if bless and designate are valid service IDs
    if bless > u32::MAX as u64 || designate > u32::MAX as u64 {
        return Ok(Exit::Who as u64);
    }

    // Read assign array from memory
    let assign = {
        let size = 4 * score::CORES_COUNT as u32;
        let data = state.memory.read_bytes(assign as u32, size)?;
        let mut assign = [0u32; score::CORES_COUNT];
        for (i, chunk) in data.chunks(4).enumerate() {
            if i < score::CORES_COUNT {
                assign[i] = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            }
        }

        assign
    };

    // Read always accumulate map from memory
    let mut always_acc = BTreeMap::new();
    if entries > 0 {
        let source = state.memory.read_bytes(acc as u32, (12 * entries) as u32)?;
        for chunk in source.chunks(12) {
            let service_id = u32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]);
            let gas_allowance = u64::from_le_bytes([
                chunk[4], chunk[5], chunk[6], chunk[7], chunk[8], chunk[9], chunk[10], chunk[11],
            ]);
            always_acc.insert(service_id, gas_allowance);
        }
    }

    // Update privileges: tuple{m, 𝐚, v, 𝐳}
    ctx.set_privileges(Privileges {
        bless: bless as u32,
        assign,
        designate: designate as u32,
        always_acc,
    });

    Ok(Exit::Ok as u64)
}

/// (ΩA) assign authorization queue
pub fn assign(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [core, o, assign] = [state.registers[7], state.registers[8], state.registers[9]];
    let source = state
        .memory
        .read_bytes(o as u32, (12 * score::QUEUE_ITEMS) as u32)?;

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
pub fn designate(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    // get the data source
    let o = state.registers[7];
    let source = state
        .memory
        .read_bytes(o as u32, 336 * score::VALIDATORS_COUNT as u32)?;

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
pub fn checkpoint(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    ctx.checkpoint();
    Ok(state.gas as u64)
}

/// (ΩN) new
pub fn new_(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [o, length, accumulate_gas, transfer_gas, f] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
        state.registers[11],
    ];

    // check if the service is blessed
    let privileges = ctx.privileges();
    if f != 0 && privileges.bless != ctx.service() {
        return Ok(Exit::Huh as u64);
    }

    // get the account code
    let code = state.memory.read_hash(o as u32)?;
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

    state.gas -= accumulate_gas as i64;
    ctx.upsert(index, created);

    let new_index = ctx.check(((index - (1 << 8) + 42) % score::CHECK_SALT) + (1 << 8));
    ctx.set_index(new_index);
    Ok(index as u64)
}

/// (ΩU) upgrade service code
pub fn upgrade(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [o, g, m] = [state.registers[7], state.registers[8], state.registers[9]];
    let code = state.memory.read_hash(o as u32)?;
    let account = ctx.this()?;
    account.set_code(code);
    account.set_transfer_gas(m);
    account.set_accumulate_gas(g);
    Ok(Exit::Ok as u64)
}

/// (ΩT) transfer funds from the sender to the destination
pub fn transfer(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [dest, amount, limit, memo] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    // check if the defer transfer is valid
    let memo = state
        .memory
        .read_bytes(memo as u32, score::TRANSFER_MEMO_SIZE)?;
    let transfer = DeferredTransfer {
        sender: ctx.service(),
        recipient: dest as u32,
        amount,
        memo,
        gas_limit: limit,
    };

    // check if the sender has enough balance
    let sender = ctx.this()?;
    let sender_id = sender.index();
    let balance = sender.balance();
    if balance.saturating_sub(amount) < sender.threshold() {
        return Ok(Exit::Cash as u64);
    }

    // drop the sender account to handle the dest account
    let _ = sender;
    let dest = ctx.account(dest as u64)?;

    // check if the destination has enough transfer gas
    if dest.transfer_gas() < limit {
        return Ok(Exit::Low as u64);
    }

    // add the transfer to the deferred transfers
    tracing::debug!("transferring {amount} from {sender_id} to {}", dest.index());
    ctx.transfer(transfer);
    *ctx.this()?.balance_mut() -= amount;
    Ok(Exit::Ok as u64)
}

/// (ΩE) eject a sub account
pub fn eject(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [dest, o] = [state.registers[7], state.registers[8]];
    let hash = state.memory.read_hash(o as u32)?;
    if dest == ctx.service() as u64 {
        crate::bail!("cannot eject to self");
    }

    let service = ctx.service();
    let timeslot = ctx.timeslot();
    let Ok(dest) = ctx.account(dest) else {
        return Ok(Exit::Who as u64);
    };

    // check if the code is valid
    let mut code = [0; 32];
    code[..4].copy_from_slice(&service.to_le_bytes());
    if dest.code() != code {
        return Ok(Exit::Who as u64);
    }

    // check if the look up is valid
    if dest.items() != 2 {
        return Ok(Exit::Huh as u64);
    }
    let length = dest.total().saturating_sub(81);
    let Some(lookup) = dest.lookup(hash, length as u32) else {
        return Ok(Exit::Huh as u64);
    };

    // remove account and add the balance to the parent account
    if lookup.len() == 2 && lookup[1] < timeslot.saturating_sub(score::EXPUNGED_TIME) {
        let balance = dest.balance();
        let to_remote = dest.index();
        let _ = dest;
        *ctx.this()?.balance_mut() += balance;
        ctx.remove(to_remote);
        return Ok(Exit::Ok as u64);
    }

    Ok(Exit::Huh as u64)
}

/// (ΩQ) query an lookup entry
pub fn query(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let (o, z) = (state.registers[7] as u32, state.registers[8] as u32);
    let hash = state.memory.read_hash(o)?;
    let account = ctx.this()?;
    let Some(lookup) = account.lookup(hash, z) else {
        state.registers[8] = 0;
        return Ok(Exit::None as u64);
    };

    // update result
    let base = 1u64 << 32;
    let exit = if lookup.is_empty() {
        state.registers[8] = 0;
        0
    } else if lookup.len() == 1 {
        state.registers[8] = 0;
        1 + base * lookup[0] as u64
    } else if lookup.len() == 2 {
        state.registers[8] = lookup[1] as u64;
        2 + base * lookup[0] as u64
    } else {
        state.registers[8] = lookup[1] as u64 + base * lookup[2] as u64;
        3 + base * lookup[0] as u64
    };
    Ok(exit)
}

/// (ΩS) solicit new lookup
pub fn solicit(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [o, z] = [state.registers[7], state.registers[8]];
    let hash = state.memory.read_hash(o as u32)?;

    // check if the account has enough balance
    let timeslot = ctx.timeslot();
    let account = ctx.this()?;
    if account.balance() < account.threshold() {
        return Ok(Exit::Full as u64);
    }

    // get the lookup
    let Some(mut lookup) = account.lookup(hash, z as u32) else {
        account.insert_lookup(hash, z as u32, vec![]);
        return Ok(Exit::Ok as u64);
    };

    if lookup.len() == 2 {
        lookup.push(timeslot);
        account.insert_lookup(hash, z as u32, lookup);
    } else {
        return Ok(Exit::Huh as u64);
    }

    Ok(Exit::Ok as u64)
}

/// (ΩF) forget
pub fn forget(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [o, z] = [state.registers[7], state.registers[8]];
    let hash = state.memory.read_hash(o as u32)?;

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
pub fn yield_(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let o = state.registers[7];
    let hash = state.memory.read_hash(o as u32)?;
    ctx.output(hash);
    Ok(Exit::Ok as u64)
}

/// (ΩP) provide new preimage
pub fn provide(ctx: &mut impl Argument, state: &mut State) -> Result<ExitCode> {
    let [mut service, from, size] = [state.registers[7], state.registers[8], state.registers[9]];
    if service == u64::MAX {
        service = ctx.service() as u64;
    }

    let image = state.memory.read_bytes(from as u32, size as u32)?;
    let Ok(account) = ctx.account(service) else {
        return Ok(Exit::Who as u64);
    };

    // check if the preimage is already in the account
    let hash = crypto::blake2b(&image);
    if account.lookup(hash, size as u32) != Some(vec![]) {
        return Ok(Exit::Huh as u64);
    }

    // check if the preimage is already in the account
    if account.preimage(hash).is_some() {
        return Ok(Exit::Huh as u64);
    }

    account.insert_preimage(hash, image);
    Ok(Exit::Ok as u64)
}
