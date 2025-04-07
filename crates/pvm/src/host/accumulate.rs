//! Accumulation related host calls

use crate::{
    host::{Exit, ExitCode},
    Argument, Reason, Result, State,
};
use codec::Numeric;
use score::{
    service::{GasLimit, Privileges, ServiceAccount},
    vm::{AccumulateContext, DeferredTransfer},
    TimeSlot,
};
use std::collections::BTreeMap;

/// Accumulate arguments
#[derive(Default)]
pub struct Accumulate {
    /// The regular dimension
    pub x: AccumulateContext,

    /// The exceptional dimension
    pub y: AccumulateContext,

    /// The timeslot
    pub timeslot: TimeSlot,
}

impl Accumulate {
    /// Create a new accumulate
    pub fn new(x: AccumulateContext, timeslot: TimeSlot) -> Self {
        Self {
            y: x.clone(),
            x,
            timeslot,
        }
    }

    /// Get the account
    pub fn account(&mut self) -> Result<&mut ServiceAccount> {
        self.x
            .context
            .accounts
            .get_mut(&self.x.service)
            .ok_or(Reason::Panic("Could not find account".into()))
    }
}

/// Accumulation calls
pub fn call<X: Argument, Memory: crate::Memory>(
    call: u32,
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    match call {
        5 => self::bless(state, data),
        6 => self::assign(state, data),
        7 => self::designate(state, data),
        8 => self::checkpoint(state, data),
        9 => self::new(state, data),
        10 => self::upgrade(state, data),
        11 => self::transfer(state, data),
        12 => self::eject(state, data),
        13 => self::query(state, data),
        14 => self::solicit(state, data),
        15 => self::forget(state, data),
        16 => self::yield_(state, data),
        _ => Ok(Exit::What as u64),
    }
}

/// (ΩB) bless
fn bless<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let [m, a, v, o, n] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
        state.registers[11],
    ];

    // get the data source
    let source = state.memory.read_bytes(o as u32, (12 * n) as u32)?;

    // parse always accumulate service ids
    let mut map = BTreeMap::new();
    for chunk in source.chunks(12) {
        let index = u64::decode(&chunk[..4]);
        let value = u64::decode(&chunk[4..]);
        map.insert(index as u32, value);
    }

    // return if unknown index
    let u32max = u32::MAX as u64;
    if m > u32max || a > u32max || v > u32max {
        return Ok(Exit::Who as u64);
    }

    accumulate.x.context.privileges = Privileges {
        bless: m as u32,
        assign: a as u32,
        designate: v as u32,
        always_acc: map,
    };

    Ok(Exit::Ok as u64)
}

/// (ΩA) assign
fn assign<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the data source
    let o = state.registers[8];
    let source = state
        .memory
        .read_bytes(o as u32, (12 * score::QUEUE_ITEMS) as u32)?;

    // return if invalid core index
    let core_index = state.registers[7];
    if core_index > score::CORES_COUNT as u64 {
        return Ok(Exit::Core as u64);
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
    accumulate.x.context.authorization[core_index as usize] = queue;
    Ok(Exit::Ok as u64)
}

/// (ΩD) designate
fn designate<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the data source
    let o = state.registers[7];
    let source = state
        .memory
        .read_bytes(o as u32, 336 * score::VALIDATORS_COUNT as u32)?;

    // decode validators
    let Some(validators) = source
        .chunks(336)
        .map(|chunk| codec::decode(chunk).ok())
        .collect::<Option<Vec<_>>>()
    else {
        crate::bail!("Could not parse validators");
    };

    // set the validators
    accumulate.x.context.validators = validators;
    Ok(Exit::Ok as u64)
}

/// (ΩC) checkpoint
fn checkpoint<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // set the checkpoint
    accumulate.y = accumulate.x.clone();
    Ok(state.gas as u64)
}

/// (ΩN) new
#[allow(clippy::new_ret_no_self)]
fn new<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let [o, l, g, m] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    // get the hash
    let code = state.memory.read_bytes(o as u32, 32)?;
    let mut hash = [0u8; 32];
    hash.copy_from_slice(&code);

    // update the creator's account
    let creator = accumulate.account()?;
    if creator.balance < score::BALANCE_PER_SERVICE {
        return Ok(Exit::Cash as u64);
    }

    // create the new accumulated
    creator.balance -= score::BALANCE_PER_SERVICE;
    let mut account = ServiceAccount::new(GasLimit {
        accumulate: g,
        transfer: m,
    });
    account.code = hash;
    account.lookup.insert((hash, l as u32), vec![]);

    // insert the new account to the map
    let index = accumulate.x.index;
    accumulate.x.context.accounts.insert(index, account);
    accumulate.x.check(index);
    Ok(index as u64)
}

/// (ΩU) upgrade service code
fn upgrade<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let [o, g, m] = [state.registers[7], state.registers[8], state.registers[9]];
    let code = state.memory.read_hash(o as u32)?;
    let account = accumulate.account()?;

    account.code = code;
    account.gas.transfer = m;
    account.gas.accumulate = g;
    Ok(Exit::Ok as u64)
}

/// (ΩT) transfer
fn transfer<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let [d, a, limit, o] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    // check if the recipient exists
    let Some(dest) = accumulate.x.context.accounts.get(&(d as u32)) else {
        return Ok(Exit::Who as u64);
    };

    // check if the transfer limit is enough
    if limit < dest.gas.transfer {
        return Ok(Exit::Low as u64);
    }

    // update the sender's account
    let account = accumulate.account()?;

    // check if the sender has enough balance
    if account.balance < a + account.threshold() {
        return Ok(Exit::Cash as u64);
    }

    // update the sender's balance
    account.balance -= a;

    // get the memo
    let memo = state
        .memory
        .read_bytes(o as u32, score::TRANSFER_MEMO_SIZE)?;

    // create the deferred transfer
    accumulate.x.transfer.push(DeferredTransfer {
        sender: accumulate.x.service,
        recipient: d as u32,
        amount: a,
        memo,
        gas_limit: limit,
    });
    Ok(Exit::Ok as u64)
}

/// (ΩE) eject
fn eject<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let (d, o) = (state.registers[7] as u32, state.registers[8] as u32);
    let hash = state.memory.read_hash(o)?;

    // check if the service exists
    let Some(dest) = accumulate.x.context.accounts.get(&d) else {
        return Ok(Exit::Who as u64);
    };

    // check the creation code
    let ibytes = accumulate.x.service.to_le_bytes();
    let mut code = [0u8; 32];
    code[..4].copy_from_slice(&ibytes);
    if dest.code != code {
        return Ok(Exit::Who as u64);
    }

    // check items
    let dest = dest.clone();
    let total = (dest.total().max(81) - 81) as u32;
    if dest.items() != 2 {
        return Ok(Exit::Huh as u64);
    }

    // check the lookup data
    let Some(lookup) = dest.lookup.get(&(hash, total)) else {
        return Ok(Exit::Huh as u64);
    };

    // check if the preimage is expunged
    if *lookup.get(1).unwrap_or(&0) >= accumulate.timeslot - score::EXPUNGED_TIME {
        return Ok(Exit::Huh as u64);
    }

    // update the account map
    accumulate.x.context.accounts.remove(&d);
    let account = accumulate.account()?;

    account.balance += dest.balance;
    Ok(Exit::Ok as u64)
}

/// (ΩQ) query
fn query<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let (o, z) = (state.registers[7] as u32, state.registers[8] as u32);
    let hash = state.memory.read_hash(o)?;
    let account = accumulate.account()?;

    // query the lookup state
    let Some(lookup) = account.lookup.get(&(hash, z)) else {
        state.registers[8] = 0;
        return Ok(Exit::None as u64);
    };

    // update registers
    let exit = if lookup.is_empty() {
        state.registers[8] = 0;
        0
    } else if lookup.len() == 1 {
        state.registers[8] = 0;
        1 + u32::MAX as u64 * lookup[0] as u64
    } else if lookup.len() == 2 {
        state.registers[8] = lookup[1] as u64;
        2 + u32::MAX as u64 * lookup[0] as u64
    } else {
        state.registers[8] = lookup[1] as u64 + u32::MAX as u64 * lookup[2] as u64;
        3 + u32::MAX as u64 * lookup[0] as u64
    };
    Ok(exit)
}

/// (ΩS) solicit
fn solicit<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let [o, z] = [state.registers[7], state.registers[8]];
    let hash = state.memory.read_hash(o as u32)?;

    // check if the account has enough balance
    let timeslot = accumulate.timeslot;
    let account = accumulate.account()?;
    let this = account.clone();
    if this.balance < this.threshold() {
        return Ok(Exit::Full as u64);
    }

    // get the lookup
    let lookup = account.lookup.entry((hash, z as u32)).or_insert(vec![]);
    if lookup.len() == 2 {
        lookup.push(timeslot);
    } else {
        return Ok(Exit::Huh as u64);
    }

    Ok(Exit::Ok as u64)
}

/// (ΩF) forget
fn forget<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let [o, z] = [state.registers[7], state.registers[8]];
    let hash = state.memory.read_hash(o as u32)?;

    // get the lookup data
    let timeslot = accumulate.timeslot;
    let account = accumulate.account()?;
    let Some(lookup) = account.lookup.get_mut(&(hash, z as u32)) else {
        return Ok(Exit::Huh as u64);
    };

    let expunged = timeslot - score::EXPUNGED_TIME;
    if lookup.is_empty() || (lookup.len() == 2 && lookup[1] < expunged) {
        account.lookup.remove(&(hash, z as u32));
        account.preimage.remove(&hash);
    } else if lookup.len() == 1 {
        lookup.push(timeslot);
    } else if lookup.len() == 3 && lookup[2] < expunged {
        *lookup = vec![lookup[2], timeslot];
    } else {
        return Ok(Exit::Huh as u64);
    }

    Ok(Exit::Ok as u64)
}

/// (ΩY) yield
fn yield_<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Result<ExitCode> {
    let accumulate = data.as_accumulate_mut()?;

    // get the arguments
    let o = state.registers[7];

    // get the hash
    let hash = state.memory.read_hash(o as u32)?;

    accumulate.x.output = Some(hash);
    Ok(Exit::Ok as u64)
}
