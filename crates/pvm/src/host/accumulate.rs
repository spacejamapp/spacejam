//! Accumulation related host calls

use crate::{
    host::{Exit, ExitCode},
    invocation::{Accumulate, State},
    Result,
};
use codec::Numeric;
use score::{
    service::{GasLimit, Privileges, ServiceAccount},
    vm::DeferredTransfer,
    Account, Accounts,
};
use std::collections::BTreeMap;

impl<R: Accounts> Accumulate<R> {
    /// Call an accumulate host function
    pub fn call<M: crate::Memory>(&mut self, call: u32, state: &mut State<M>) -> Result<ExitCode> {
        match call {
            14 => self.bless(state),
            15 => self.assign(state),
            16 => self.designate(state),
            17 => self.checkpoint(state),
            18 => self.new_(state),
            19 => self.upgrade(state),
            20 => self.transfer(state),
            21 => self.eject(state),
            22 => self.query(state),
            23 => self.solicit(state),
            24 => self.forget(state),
            25 => self.yield_(state),
            26 => {
                // TODO: PROVIDE
                Ok(Exit::What as u64)
            }
            _ => Ok(Exit::What as u64),
        }
    }

    /// (ΩB) bless
    pub fn bless<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
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

        // TODO: fix the assign array
        self.x.context.privileges = Privileges {
            bless: m as u32,
            assign: [a as u32; score::CORES_COUNT],
            designate: v as u32,
            always_acc: map,
        };

        Ok(Exit::Ok as u64)
    }

    /// (ΩA) assign
    pub fn assign<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
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
        self.x.context.authorization[core_index as usize] = queue;
        Ok(Exit::Ok as u64)
    }

    /// (ΩD) designate
    pub fn designate<Memory: crate::Memory>(
        &mut self,
        state: &mut State<Memory>,
    ) -> Result<ExitCode> {
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
        self.x.context.validators = validators;
        Ok(Exit::Ok as u64)
    }

    /// (ΩC) checkpoint
    pub fn checkpoint<Memory: crate::Memory>(
        &mut self,
        state: &mut State<Memory>,
    ) -> Result<ExitCode> {
        // set the checkpoint
        self.y = self.x.clone();
        Ok(state.gas as u64)
    }

    /// (ΩN) new
    pub fn new_<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
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
        let creator = self.account()?;
        if creator.balance() < score::BALANCE_PER_SERVICE {
            return Ok(Exit::Cash as u64);
        }

        // create the new accumulated
        *creator.balance_mut() -= score::BALANCE_PER_SERVICE;
        let mut account = ServiceAccount::new(GasLimit {
            accumulate: g,
            transfer: m,
        });
        account.info.balance = score::BALANCE_PER_SERVICE;
        account.info.code = hash;
        account.lookup.insert((hash, l as u32), vec![]);

        // Set metadata fields for new service account
        account.info.creation = self.timeslot;
        account.info.update = self.timeslot;
        account.info.parent = self.x.service;

        // insert the new account to the map
        //
        // FIXME: this upsert doesn't consider operations.
        let index = self.x.index;
        self.x.context.accounts.upsert(index, account);
        self.x.check(index);
        Ok(index as u64)
    }

    /// (ΩU) upgrade service code
    pub fn upgrade<Memory: crate::Memory>(
        &mut self,
        state: &mut State<Memory>,
    ) -> Result<ExitCode> {
        // get the arguments
        let [o, g, m] = [state.registers[7], state.registers[8], state.registers[9]];
        let code = state.memory.read_hash(o as u32)?;
        let account = self.account()?;

        account.set_code(code);
        account.set_transfer_gas(m);
        account.set_accumulate_gas(g);
        Ok(Exit::Ok as u64)
    }

    /// (ΩT) transfer
    pub fn transfer<Memory: crate::Memory>(
        &mut self,
        state: &mut State<Memory>,
    ) -> Result<ExitCode> {
        // get the arguments
        let [d, a, limit, o] = [
            state.registers[7],
            state.registers[8],
            state.registers[9],
            state.registers[10],
        ];

        // check if the recipient exists
        let Some(dest) = self.x.context.accounts.get(d as u32) else {
            return Ok(Exit::Who as u64);
        };

        // check if the transfer limit is enough
        if limit < dest.transfer_gas() {
            return Ok(Exit::Low as u64);
        }

        // update the sender's account
        let account = self.account()?;

        // check if the sender has enough balance
        if account.balance() < a + account.threshold() {
            return Ok(Exit::Cash as u64);
        }

        // update the sender's balance
        *account.balance_mut() -= a;
        let memo = state
            .memory
            .read_bytes(o as u32, score::TRANSFER_MEMO_SIZE)?;

        // create the deferred transfer
        self.x.transfer.push(DeferredTransfer {
            sender: self.x.service,
            recipient: d as u32,
            amount: a,
            memo,
            gas_limit: limit,
        });
        Ok(Exit::Ok as u64)
    }

    /// (ΩE) eject
    pub fn eject<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
        // get the arguments
        let (d, o) = (state.registers[7] as u32, state.registers[8] as u32);
        let hash = state.memory.read_hash(o)?;

        // check if the service exists
        let Some(dest) = self.x.context.accounts.get(d) else {
            return Ok(Exit::Who as u64);
        };

        // check the creation code
        let ibytes = self.x.service.to_le_bytes();
        let mut code = [0u8; 32];
        code[..4].copy_from_slice(&ibytes);
        if dest.code() != code {
            return Ok(Exit::Who as u64);
        }

        // check items
        let total = (dest.total().max(81) - 81) as u32;
        if dest.items() != 2 {
            return Ok(Exit::Huh as u64);
        }

        // check the lookup data
        let Some(lookup) = dest.lookup(hash, total) else {
            return Ok(Exit::Huh as u64);
        };

        // check if the preimage is expunged
        if *lookup.get(1).unwrap_or(&0) >= self.timeslot - score::EXPUNGED_TIME {
            return Ok(Exit::Huh as u64);
        }

        // update the account
        let balance = dest.balance();
        let _ = dest;
        self.x.context.accounts.remove(d);
        let account = self.account()?;
        *account.balance_mut() += balance;
        Ok(Exit::Ok as u64)
    }

    /// (ΩQ) query
    pub fn query<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
        let (o, z) = (state.registers[7] as u32, state.registers[8] as u32);
        let hash = state.memory.read_hash(o)?;
        let account = self.account()?;
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

    /// (ΩS) solicit
    pub fn solicit<Memory: crate::Memory>(
        &mut self,
        state: &mut State<Memory>,
    ) -> Result<ExitCode> {
        let [o, z] = [state.registers[7], state.registers[8]];
        let hash = state.memory.read_hash(o as u32)?;

        // check if the account has enough balance
        let timeslot = self.timeslot;
        tracing::debug!("solicit: timeslot: {}", timeslot);
        let account = self.account()?;
        tracing::debug!("solicit: balance: {}", account.balance());
        if account.balance() < account.threshold() {
            tracing::debug!("solicit: full");
            return Ok(Exit::Full as u64);
        }

        // get the lookup
        let Some(mut lookup) = account.lookup(hash, z as u32) else {
            tracing::debug!("solicit: empty");
            account.insert_lookup(hash, z as u32, vec![]);
            return Ok(Exit::Ok as u64);
        };

        if lookup.len() == 2 {
            tracing::debug!("solicit: double");
            lookup.push(timeslot);
            account.insert_lookup(hash, z as u32, lookup);
        } else {
            tracing::debug!("solicit: huh");
            return Ok(Exit::Huh as u64);
        }

        Ok(Exit::Ok as u64)
    }

    /// (ΩF) forget
    pub fn forget<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
        let [o, z] = [state.registers[7], state.registers[8]];
        let hash = state.memory.read_hash(o as u32)?;

        // get the lookup data
        let timeslot = self.timeslot;
        let account = self.account()?;
        let Some(mut lookup) = account.lookup(hash, z as u32) else {
            return Ok(Exit::Huh as u64);
        };

        let expunged = timeslot.saturating_sub(score::EXPUNGED_TIME);
        tracing::debug!("forget: timeslot={timeslot}, lookup={lookup:?}, expunged={expunged}",);
        if lookup.is_empty() || (lookup.len() == 2 && lookup[1] < expunged) {
            tracing::debug!("forget: empty or expired");
            account.remove_lookup(hash, z as u32);
            account.remove_preimage(hash);
        } else if lookup.len() == 1 {
            tracing::debug!("forget: single");
            lookup.push(timeslot);
            account.insert_lookup(hash, z as u32, lookup);
        } else if lookup.len() == 3 && lookup[1] < expunged {
            tracing::debug!("forget: triple");
            account.insert_lookup(hash, z as u32, vec![lookup[2], timeslot]);
        } else {
            return Ok(Exit::Huh as u64);
        }

        Ok(Exit::Ok as u64)
    }

    /// (ΩY) yield
    pub fn yield_<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
        // get the arguments
        let o = state.registers[7];

        // get the hash
        let hash = state.memory.read_hash(o as u32)?;

        self.x.output = Some(hash);
        Ok(Exit::Ok as u64)
    }
}
