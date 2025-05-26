//! Iterate over the storage

use crate::chain;
use score::{
    extrinsic::TicketsOrKeys,
    safrole::{Safrole, ValidatorIter, ValidatorsData},
    state::key,
    Block,
};
use std::path::PathBuf;

/// Iterate over the storage
pub async fn run(data: PathBuf) -> anyhow::Result<()> {
    let spec = chain::Spec::dev().parse()?;
    let safrole = spec.genesis_state.get(&key::SAFROLE).unwrap();
    let safrole: Safrole = codec::decode(safrole.as_ref())?;
    let TicketsOrKeys::Keys(keys) = &safrole.series else {
        anyhow::bail!("series is not keys");
    };

    println!(
        "keys: {:#?}",
        keys.iter()
            .enumerate()
            .map(|(i, k)| format!("{i:02} | 0x{}", hex::encode(k)))
            .collect::<Vec<_>>()
    );

    let validators = spec.genesis_state.get(&key::NEXT_VALIDATORS).unwrap();
    let validators: ValidatorsData = codec::decode(validators.as_ref())?;
    let validators = validators.bandersnatch();
    println!(
        "validators: {:#?}",
        validators
            .iter()
            .enumerate()
            .map(|(i, v)| format!("{i:02} | {}", hex::encode(v)))
            .collect::<Vec<_>>()
    );

    let mut blocks = Vec::new();
    let db = sled::open(data.join("dev"))?;
    db.iter().for_each(|entry| {
        let (key, value) = entry.unwrap();
        if key.starts_with(b"block") && key.len() == 37 {
            let block: Block = codec::decode(value.as_ref()).unwrap();
            if block.header.slot == 0 {
                return;
            }

            blocks.push((block.header.slot, block.header.author_index))
        }
    });

    blocks.sort_by_key(|(slot, _)| *slot);
    println!(
        "blocks: {:#?}",
        blocks
            .iter()
            .map(|(slot, author)| format!(
                "{:02} | {slot} | {author:02} | 0x{}",
                slot % score::EPOCH_LENGTH,
                hex::encode(validators[*author as usize])
            ))
            .collect::<Vec<_>>()
    );

    Ok(())
}
