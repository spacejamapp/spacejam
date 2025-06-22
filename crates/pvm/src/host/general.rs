//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    invocation::{General, State},
    Result,
};
use score::{state::account, Account, Accounts, Gas, Parameters, ServiceId};

impl<R: Accounts> General<R> {
    /// General host calls
    ///
    /// parameters: ϱ,ω,µ,s,...
    ///
    /// with the range 0..5
    pub fn call<Memory: crate::Memory>(
        &mut self,
        call: u32,
        state: &mut State<Memory>,
    ) -> Result<ExitCode> {
        match call {
            0 => self.gas(state.gas as u64),
            1 => self.lookup(state),
            2 => self.read(state),
            3 => self.write(state),
            4 => self.info(state),
            18 => self.fetch(state),
            _ => Ok(Exit::What as u64),
        }
    }

    /// (ΩG) Get the gas to register
    fn gas(&mut self, gas: Gas) -> Result<u64> {
        Ok(gas)
    }

    /// (ΩL) account lookup
    fn lookup<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<u64> {
        let Some((_, mut account)) = self.get(state.registers[7]) else {
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
    fn read<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
        // get the account
        let Some((index, mut account)) = self.get(state.registers[7]) else {
            return Ok(Exit::None as u64);
        };

        // get the key
        let [ko, kz, o] = [state.registers[8], state.registers[9], state.registers[10]];
        let key = state
            .memory
            .read_bytes(ko as u32, kz as u32)
            .expect("should not fail");

        // get the storage value
        let skey = account::storage(index, &key);
        let Some(value) = account.read(&skey) else {
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
    fn write<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
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
                tracing::error!("Failed to read key bytes: {:?}", err);
                return Ok(Exit::OOB as u64);
            }
        };

        let index = self.index;
        let Some(account) = self.account() else {
            return Ok(Exit::None as u64);
        };

        // update storage
        let skey = account::storage(index, &key);
        if vz == 0 {
            let Some(value) = account.remove(&skey) else {
                return Ok(Exit::None as u64);
            };

            self.updated = true;
            Ok(value.len() as u64)
        } else {
            let value = match state.memory.read_bytes(vo as u32, vz as u32) {
                Ok(bytes) => bytes,
                Err(err) => {
                    tracing::warn!("Failed to read value bytes: {:?}", err);
                    return Ok(Exit::OOB as u64);
                }
            };

            let threshold = account.threshold();
            if threshold > account.balance() {
                Ok(Exit::Full as u64)
            } else {
                let length = value.len() as u64;
                account.write(&skey, value);
                self.updated = true;
                Ok(length)
            }
        }
    }

    /// (ΩI) fetch info
    fn info<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
        // get and encode the account state
        let r7 = state.registers[7];
        let Some(account) = if r7 == u64::MAX {
            self.accounts.get(self.index)
        } else {
            self.accounts.get(r7 as ServiceId)
        }
        .and_then(|account| {
            let state = account.info();
            codec::encode(&state).ok()
        }) else {
            return Ok(Exit::None as u64);
        };

        // write the account state to memory
        let address = state.registers[8];
        if let Err(reason) = state.memory.write_bytes(address as u32, &account) {
            crate::bail!("failed to write account state {reason}");
        }

        Ok(Exit::Ok as u64)
    }

    // (ΩY) fetch the on chain parameters
    fn fetch<Memory: crate::Memory>(&mut self, state: &mut State<Memory>) -> Result<ExitCode> {
        let value: Vec<u8> = match state.registers[10] {
            0 => codec::encode(&Parameters::default()).expect("should not fail"),
            14 => codec::encode(&self.operands).expect("should not fail"),
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
}
