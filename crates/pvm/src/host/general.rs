//! General host call functions

use crate::{
    host::{Exit, ExitCode},
    invocation::State,
    Argument, Reason, Result,
};
use score::{
    account::{Account, Accounts},
    state::account,
    Gas, ServiceId,
};

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
        let from = state.registers[11].min(value.len() as u64);
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
                tracing::info!("writing storage: {:?}", skey);
                let length = value.len() as u64;
                account.write(&skey, value);
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
            tracing::debug!("account info: {:?}", state);
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
}

/// Input data of general host functions
#[derive(Debug, Clone)]
pub struct General<R: Accounts> {
    /// (s) Service index
    pub index: ServiceId,

    /// (d) Account dictionary
    pub accounts: R,
}

impl<R: Accounts> General<R> {
    /// Get service account
    pub fn get(&mut self, r7: u64) -> Option<(ServiceId, impl Account + '_)> {
        let service = self.index as u64;
        let mut index = r7 as ServiceId;
        if r7 == u64::MAX || r7 == service {
            index = self.index;
        }

        self.accounts
            .get(index)
            .map(|account| (index, account.clone()))
    }

    /// Get the account
    pub fn account(&mut self) -> Option<&mut (impl Account + '_)> {
        self.accounts.get(self.index)
    }
}

impl<R: Accounts> Argument<R> for General<R> {
    fn as_general(&self) -> crate::Result<General<R>> {
        Ok(self.clone())
    }

    fn update_general(&mut self, general: General<R>) -> crate::Result<()> {
        *self = general;
        Ok(())
    }
}
