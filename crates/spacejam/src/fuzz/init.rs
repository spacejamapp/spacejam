//! Initialization helpers

use anyhow::Result;
use runtime::{
    Accounts,
    storage::{MemoryDb, StateStorage},
    tx::ticket::lazy,
};
use score::{
    Accounts as _,
    state::{ServiceField, StateKey, StateKeyInfo, StateKeyLike},
};
use spacevm::{
    Memory,
    pvm::{Context, invocation::Accumulate},
};
use std::sync::Arc;
use tokio::task::JoinSet;

/// Initialize the verifier
pub async fn verifier(data: Arc<MemoryDb>) -> Result<()> {
    let safrole = data.safrole()?;
    let timeslot = data.timeslot()?;
    let epoch = timeslot / score::EPOCH_LENGTH;
    let _ = lazy::drawn(epoch, &safrole.validators).await;
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

            if let Some(blob) = accounts.blob(service) {
                queue.spawn_blocking(move || {
                    spacevm::compile::<Context<'_, Accumulate<Accounts<MemoryDb>>, Memory>>(
                        blob,
                        vec![],
                        hash,
                    )
                });
            }
        }
    }

    let _ = queue.join_all().await;
    Ok(())
}
