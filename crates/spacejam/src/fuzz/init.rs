//! Initialization helpers

use account::Accounts as _;
use anyhow::Result;
use runtime::{
    Accounts,
    storage::{MemoryDb, StateStorage},
    tx::ticket::lazy,
};
use score::state::{ServiceField, StateKey, StateKeyInfo, StateKeyLike};
use spacevm::{Memory, pvm::Context};
use std::sync::Arc;
use tokio::task::JoinSet;

/// Initialize the verifier
pub async fn verifier(data: Arc<MemoryDb>) -> Result<()> {
    let safrole = data.safrole()?;
    let timeslot = data.timeslot()?;
    let validators = data.current_validators()?;
    let epoch = timeslot / score::EPOCH_LENGTH;
    lazy::drawn(epoch + 1, &safrole.validators);
    lazy::drawn(epoch, &validators);
    Ok(())
}

/// Initialize the programs
///
/// Find all accounts and their codes
///
/// FIXM: currently only supports accumulation
pub async fn programs(data: Arc<MemoryDb>) -> Result<()> {
    let mut accounts = Accounts::new(data.clone());
    let mut queue = JoinSet::new();
    for pair in data.state_iter()? {
        let (k, _v) = pair?;
        let info = k.as_state_key().info();
        if let StateKey::Account {
            service,
            field: ServiceField::Info,
        } = info
        {
            let Some(hash) = accounts.code_hash(service) else {
                continue;
            };

            // compile for twice to pass the confirmation
            if let Some(blob) = accounts.blob(service) {
                let blob_1 = blob.clone();
                queue.spawn_blocking(move || {
                    spacevm::compile::<Context<'static, (), Memory>>(blob_1, vec![], hash, false)
                });
                queue.spawn_blocking(move || {
                    spacevm::compile::<Context<'static, (), Memory>>(blob, vec![], hash, false)
                });
            }
        }
    }

    let _ = queue.join_all().await;
    Ok(())
}
