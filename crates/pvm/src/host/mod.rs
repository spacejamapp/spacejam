//! Host functions

use crate::{Reason, State, Stepped};
pub use {accumulate::Accumulate, general::General, refine::Refine};

mod accumulate;
mod general;
mod jip;
mod refine;

/// Call the host function
pub fn call<X: Argument, Memory: crate::Memory>(
    call: u32,
    mut state: State<Memory>,
    data: X,
) -> Stepped<Memory, X> {
    let mut data = data;
    let reason = match call {
        0..6 => {
            let general = match data.as_general() {
                Ok(g) => g,
                Err(e) => return Stepped::new(e, state).with(data),
            };
            let account = general.account.clone();
            general::call(call, &mut state, account, &mut data)
        }
        6..18 => accumulate::call(call, &mut state, &mut data),
        18..28 => refine::call(call, &mut state, &mut data),
        100 => jip::log(&mut state),
        _ => {
            tracing::debug!("unknown host call: {}", call);
            Ok(Exit::What as u64)
        }
    };

    match reason {
        Ok(exit) => {
            state.registers[7] = exit;
            Stepped::new(Reason::Continue, state).with(data)
        }
        Err(reason) => Stepped::new(reason, state).with(data),
    }
}

/// Dynamic arguments for host calls
pub trait Argument: Default {
    /// returns some if the input data is general
    fn as_general(&self) -> crate::Result<General> {
        crate::bail!("not a general")
    }

    /// update the general argument
    fn update_general(&mut self, _general: General) -> crate::Result<()> {
        crate::bail!("not a general")
    }

    /// returns some if the input data is general
    fn as_general_mut(&mut self) -> crate::Result<&mut General> {
        crate::bail!("not a general")
    }

    /// returns some if the input data is accumulate
    fn as_accumulate_mut(&mut self) -> crate::Result<&mut Accumulate> {
        crate::bail!("not an accumulate")
    }

    /// returns some if the input data is refine
    fn as_refine_mut(&mut self) -> crate::Result<&mut Refine> {
        crate::bail!("not a refine")
    }
}

impl Argument for Accumulate {
    fn as_general(&self) -> crate::Result<General> {
        let account = self
            .x
            .context
            .accounts
            .get(&self.x.service)
            .ok_or_else(|| {
                crate::Reason::Panic(format!("Account {} not found in context", self.x.service))
            })?;

        Ok(General {
            account: account.clone(),
            index: self.x.service,
            accounts: self.x.context.accounts.clone(),
        })
    }

    fn update_general(&mut self, general: General) -> crate::Result<()> {
        tracing::debug!(
            "Accumulate update_general called for service {}, storage entries: {}",
            general.index,
            general.account.storage.len()
        );
        tracing::debug!(
            "Accumulate context before update - accounts: {:?}",
            self.x.context.accounts.keys().collect::<Vec<_>>()
        );
        tracing::debug!(
            "Accumulate context before update - service {} storage entries: {}",
            general.index,
            self.x
                .context
                .accounts
                .get(&general.index)
                .map(|a| a.storage.len())
                .unwrap_or(0)
        );

        tracing::debug!(
            "About to insert account with {} storage entries",
            general.account.storage.len()
        );
        self.x
            .context
            .accounts
            .insert(general.index, general.account.clone());

        // Directly verify what was inserted
        if let Some(inserted_account) = self.x.context.accounts.get(&general.index) {
            tracing::debug!(
                "Verification: inserted account has {} storage entries",
                inserted_account.storage.len()
            );
        } else {
            tracing::debug!("Verification: failed to find inserted account!");
        }

        // Also update any other modified accounts from the general context
        tracing::debug!(
            "About to process general.accounts: {:?}",
            general.accounts.keys().collect::<Vec<_>>()
        );
        for (id, account) in general.accounts {
            // Skip the main service account to avoid overwriting the updated account with stale data
            if id == general.index {
                tracing::debug!(
                    "Skipping main service {} from general.accounts to preserve updated storage",
                    id
                );
                continue;
            }
            tracing::debug!(
                "Inserting account {} from general.accounts with {} storage entries",
                id,
                account.storage.len()
            );
            self.x.context.accounts.insert(id, account);
        }

        // Final verification using the same logic as the "after update" check
        let final_storage_count = self
            .x
            .context
            .accounts
            .get(&general.index)
            .map(|a| a.storage.len())
            .unwrap_or(0);
        tracing::debug!(
            "Accumulate context after update - service {} storage entries: {}",
            general.index,
            final_storage_count
        );

        // Additional debug: check if the account objects are the same
        if let Some(final_account) = self.x.context.accounts.get(&general.index) {
            tracing::debug!(
                "Final account storage keys: {:?}",
                final_account.storage.keys().take(3).collect::<Vec<_>>()
            );
        }

        Ok(())
    }

    fn as_accumulate_mut(&mut self) -> crate::Result<&mut Accumulate> {
        Ok(self)
    }
}

/// Host call results
#[repr(u64)]
pub enum Result {
    /// The return value indicating an item does not exist.
    None = u64::MAX,
    /// Name unknown.
    What = u64::MAX - 1,
    /// The inner PVM memory index provided for reading/writing is not accessible.
    OOB = u64::MAX - 2,
    /// Index unknown
    Who = u64::MAX - 3,
    /// Storage full
    Full = u64::MAX - 4,
    /// Core index unknown
    Core = u64::MAX - 5,
    /// Insufficient funds
    Cash = u64::MAX - 6,
    /// Gas limit too low
    Low = u64::MAX - 7,
    /// The item is already solicited or cannot be forgotten.
    Huh = u64::MAX - 8,
    /// The return value indicating general success.
    Ok = 0,
}

/// The result type of host calls
pub type Exit = Result;

/// The exit code type
pub type ExitCode = u64;
