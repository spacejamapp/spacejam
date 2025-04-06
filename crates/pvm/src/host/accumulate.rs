//! Accumulation related host calls

use crate::{host::Result, Argument, Reason, State};
use codec::Numeric;
use score::{
    service::{GasLimit, Privileges, ServiceAccount},
    vm::{AccumulateContext, DeferredTransfer},
    TimeSlot,
};
use std::collections::BTreeMap;

/// Accumulate arguments
pub struct Accumulate {
    /// The regular dimension
    pub x: AccumulateContext,

    /// The exceptional dimension
    pub y: AccumulateContext,

    /// The timeslot
    pub timeslot: TimeSlot,
}

/// Accumulation calls
pub fn call<X: Argument, Memory: crate::Memory>(
    call: u32,
    state: &mut State<Memory>,
    data: &mut X,
) -> Reason {
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
        _ => Reason::Panic("Host call not found".into()),
    }
}

/// (ΩB) bless
fn bless<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let [m, a, v, o, n] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
        state.registers[11],
    ];

    // get the data source
    let source = match state.memory.read_bytes(o as u32, (12 * n) as u32) {
        Ok(source) => source,
        Err(e) => return Reason::Panic(e.to_string()),
    };

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
        state.registers[7] = Result::Who as u64;
        return Reason::Continue;
    }

    accumulate.x.context.privileges = Privileges {
        bless: m as u32,
        assign: a as u32,
        designate: v as u32,
        always_acc: map,
    };

    state.registers[7] = Result::Ok as u64;
    Reason::Continue
}

/// (ΩA) assign
fn assign<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the data source
    let o = state.registers[8];
    let source = match state
        .memory
        .read_bytes(o as u32, (12 * score::QUEUE_ITEMS) as u32)
    {
        Ok(source) => source,
        Err(e) => return Reason::Panic(e.to_string()),
    };

    // return if invalid core index
    let core_index = state.registers[7];
    if core_index > score::CORES_COUNT as u64 {
        state.registers[7] = Result::Core as u64;
        return Reason::Continue;
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
    state.registers[7] = Result::Ok as u64;
    Reason::Continue
}

/// (ΩD) designate
fn designate<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the data source
    let o = state.registers[7];
    let source = match state
        .memory
        .read_bytes(o as u32, 336 * score::VALIDATORS_COUNT as u32)
    {
        Ok(source) => source,
        Err(e) => return Reason::Panic(e.to_string()),
    };

    // decode validators
    let Some(validators) = source
        .chunks(336)
        .map(|chunk| codec::decode(chunk).ok())
        .collect::<Option<Vec<_>>>()
    else {
        return Reason::Panic("Could not parse validators".into());
    };

    // set the validators
    accumulate.x.context.validators = validators;
    state.registers[7] = Result::Ok as u64;
    Reason::Continue
}

/// (ΩC) checkpoint
fn checkpoint<X: Argument, Memory: crate::Memory>(
    state: &mut State<Memory>,
    data: &mut X,
) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // set the checkpoint
    accumulate.y = accumulate.x.clone();
    state.registers[7] = state.gas as u64;
    Reason::Continue
}

/// (ΩN) new
#[allow(clippy::new_ret_no_self)]
fn new<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let [o, l, g, m] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    // get the hash
    let code = match state.memory.read_bytes(o as u32, 32) {
        Ok(vhash) => {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&vhash);
            hash
        }
        Err(e) => return Reason::Panic(e.to_string()),
    };

    // update the creator's account
    let Some(creator) = accumulate.x.account() else {
        return Reason::Panic("Could not find account".into());
    };

    if creator.balance < score::BALANCE_PER_SERVICE {
        state.registers[7] = Result::Cash as u64;
        return Reason::Continue;
    }

    // create the new accumulated
    creator.balance -= score::BALANCE_PER_SERVICE;
    let mut account = ServiceAccount::new(GasLimit {
        accumulate: g,
        transfer: m,
    });
    account.code = code;
    account.lookup.insert((code, l as u32), vec![]);

    // insert the new account to the map
    let index = accumulate.x.index;
    accumulate.x.context.accounts.insert(index, account);
    accumulate.x.check(index);
    state.registers[7] = index as u64;
    Reason::Continue
}

/// (ΩU) upgrade service code
fn upgrade<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let [o, g, m] = [state.registers[7], state.registers[8], state.registers[9]];
    let chash = match state.memory.read_bytes(o as u32, 32) {
        Ok(chash) => {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&chash);
            hash
        }
        Err(e) => return Reason::Panic(e.to_string()),
    };

    let Some(account) = accumulate.x.account() else {
        return Reason::Panic("Could not find service account".into());
    };

    account.code = chash;
    account.gas.transfer = m;
    account.gas.accumulate = g;
    state.registers[7] = Result::Ok as u64;
    Reason::Continue
}

/// (ΩT) transfer
fn transfer<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let [d, a, limit, o] = [
        state.registers[7],
        state.registers[8],
        state.registers[9],
        state.registers[10],
    ];

    // check if the recipient exists
    let Some(dest) = accumulate.x.context.accounts.get(&(d as u32)) else {
        state.registers[7] = Result::Who as u64;
        return Reason::Continue;
    };

    // check if the transfer limit is enough
    if limit < dest.gas.transfer {
        state.registers[7] = Result::Low as u64;
        return Reason::Continue;
    }

    // update the sender's account
    let Some(account) = accumulate.x.account() else {
        return Reason::Panic("Could not find service account".into());
    };

    // check if the sender has enough balance
    if account.balance < a + account.threshold() {
        state.registers[7] = Result::Cash as u64;
        return Reason::Continue;
    }

    // update the sender's balance
    account.balance -= a;

    // get the memo
    let memo = match state.memory.read_bytes(o as u32, score::TRANSFER_MEMO_SIZE) {
        Ok(source) => source,
        Err(e) => return Reason::Panic(e.to_string()),
    };

    // create the deferred transfer
    accumulate.x.transfer.push(DeferredTransfer {
        sender: accumulate.x.service,
        recipient: d as u32,
        amount: a,
        memo,
        gas_limit: limit,
    });
    state.registers[7] = Result::Ok as u64;
    Reason::Continue
}

/// (ΩE) eject
fn eject<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let (d, o) = (state.registers[7] as u32, state.registers[8] as u32);

    // get the hash
    let hash = match state.memory.read_bytes(o, 32) {
        Ok(rhash) => {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&rhash);
            hash
        }
        Err(e) => return Reason::Panic(e.to_string()),
    };

    // check if the service exists
    let Some(dest) = accumulate.x.context.accounts.get(&d) else {
        state.registers[7] = Result::Who as u64;
        return Reason::Continue;
    };

    // check the creation code
    let ibytes = accumulate.x.service.to_le_bytes();
    let mut code = [0u8; 32];
    code[..4].copy_from_slice(&ibytes);
    if dest.code != code {
        state.registers[7] = Result::Who as u64;
        return Reason::Continue;
    }

    // check items
    let dest = dest.clone();
    let total = (dest.total().max(81) - 81) as u32;
    if dest.items() != 2 {
        state.registers[7] = Result::Huh as u64;
        return Reason::Continue;
    }

    // check the lookup data
    let Some(lookup) = dest.lookup.get(&(hash, total)) else {
        state.registers[7] = Result::Huh as u64;
        return Reason::Continue;
    };

    // check if the preimage is expunged
    if *lookup.get(1).unwrap_or(&0) >= accumulate.timeslot - score::EXPUNGED_TIME {
        state.registers[7] = Result::Huh as u64;
        return Reason::Continue;
    }

    // update the account map
    accumulate.x.context.accounts.remove(&d);
    let Some(account) = accumulate.x.account() else {
        return Reason::Panic("Could not find service account".into());
    };

    account.balance += dest.balance;
    Reason::Continue
}

/// (ΩQ) query
fn query<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let [o, z] = [state.registers[7], state.registers[8]];
    let hash = match state.memory.read_bytes(o as u32, 32) {
        Ok(rhash) => {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&rhash);
            hash
        }
        Err(e) => return Reason::Panic(e.to_string()),
    };

    let Some(account) = accumulate.x.account() else {
        return Reason::Panic("Could not find service account".into());
    };

    // query the lookup state
    let Some(lookup) = account.lookup.get(&(hash, z as u32)) else {
        state.registers[7] = Result::None as u64;
        state.registers[8] = 0;
        return Reason::Continue;
    };

    // update registers
    if lookup.is_empty() {
        state.registers[7] = 0;
        state.registers[8] = 0;
    } else if lookup.len() == 1 {
        state.registers[7] = 1 + u32::MAX as u64 * lookup[0] as u64;
        state.registers[8] = 0;
    } else if lookup.len() == 2 {
        state.registers[7] = 2 + u32::MAX as u64 * lookup[0] as u64;
        state.registers[8] = lookup[1] as u64;
    } else {
        state.registers[7] = 3 + u32::MAX as u64 * lookup[0] as u64;
        state.registers[8] = lookup[1] as u64 + u32::MAX as u64 * lookup[2] as u64;
    }
    Reason::Continue
}

/// (ΩS) solicit
fn solicit<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let [o, z] = [state.registers[7], state.registers[8]];

    // get the hash
    let hash = match state.memory.read_bytes(o as u32, 32) {
        Ok(rhash) => {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&rhash);
            hash
        }
        Err(e) => return Reason::Panic(e.to_string()),
    };

    // get the account
    let Some(account) = accumulate.x.account() else {
        return Reason::Panic("Could not find service account".into());
    };

    // check if the account has enough balance
    let this = account.clone();
    if this.balance < this.threshold() {
        state.registers[7] = Result::Full as u64;
        return Reason::Continue;
    }

    // get the lookup
    let lookup = account.lookup.entry((hash, z as u32)).or_insert(vec![]);
    if lookup.len() == 2 {
        lookup.push(accumulate.timeslot);
    } else {
        state.registers[7] = Result::Huh as u64;
        return Reason::Continue;
    }

    state.registers[7] = Result::Ok as u64;
    Reason::Continue
}

/// (ΩF) forget
fn forget<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let [o, z] = [state.registers[7], state.registers[8]];

    // get the hash
    let hash = match state.memory.read_bytes(o as u32, 32) {
        Ok(rhash) => {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&rhash);
            hash
        }
        Err(e) => return Reason::Panic(e.to_string()),
    };

    // get the account
    let Some(account) = accumulate.x.account() else {
        return Reason::Panic("Could not find service account".into());
    };

    // get the lookup data
    let Some(lookup) = account.lookup.get_mut(&(hash, z as u32)) else {
        state.registers[7] = Result::Huh as u64;
        return Reason::Continue;
    };

    let expunged = accumulate.timeslot - score::EXPUNGED_TIME;
    if lookup.is_empty() || (lookup.len() == 2 && lookup[1] < expunged) {
        account.lookup.remove(&(hash, z as u32));
        account.preimage.remove(&hash);
    } else if lookup.len() == 1 {
        lookup.push(accumulate.timeslot);
    } else if lookup.len() == 3 && lookup[2] < expunged {
        *lookup = vec![lookup[2], accumulate.timeslot];
    } else {
        state.registers[7] = Result::Huh as u64;
        return Reason::Continue;
    }

    state.registers[7] = Result::Ok as u64;
    Reason::Continue
}

/// (ΩY) yield
fn yield_<X: Argument, Memory: crate::Memory>(state: &mut State<Memory>, data: &mut X) -> Reason {
    let Some(accumulate) = data.as_accumulate_mut() else {
        return Reason::Panic("Could not find accumulate arguments".into());
    };

    // get the arguments
    let o = state.registers[7];

    // get the hash
    let hash = match state.memory.read_bytes(o as u32, 32) {
        Ok(rhash) => {
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&rhash);
            hash
        }
        Err(e) => return Reason::Panic(e.to_string()),
    };

    accumulate.x.output = Some(hash);
    Reason::Continue
}
