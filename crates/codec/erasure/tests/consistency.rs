use anyhow::Result;
use spacejam_erasure as erasure;
use specjam::Registry;
use std::path::PathBuf;

#[tokio::test]
async fn ec_3() -> anyhow::Result<()> {
    run_codec("ec-3").await
}

#[tokio::test]
async fn ec_4096() -> anyhow::Result<()> {
    run_codec("ec-4096").await
}

async fn run_codec(test: &str) -> anyhow::Result<()> {
    let registry = Registry::new(PathBuf::from("../../../res/jam-test-vectors"));
    let test = registry.erasure(specjam::Scale::Tiny)?.test(test)?;
    let mut data = hex::decode(test.input.trim_start_matches("0x"))?;
    let shards = serde_json::from_str::<Vec<String>>(&test.output)?
        .into_iter()
        .map(|shard| {
            hex::decode(shard.trim_start_matches("0x"))
                .map_err(|e| anyhow::anyhow!("Failed to decode shard: {e}"))
        })
        .collect::<Result<Vec<_>>>()?;

    // testing sync
    {
        let encoded = erasure::encode_sync(data.clone())?;
        assert_eq!(encoded, shards);

        let decoded = erasure::decode_sync(vec![(0, shards[0].clone()), (2, shards[2].clone())])?;
        data.resize(decoded.len(), 0);
        assert_eq!(decoded, data);
    }

    // testing async
    {
        let encoded = erasure::encode(data.clone()).await?;
        assert_eq!(encoded, shards);

        let decoded = erasure::decode(vec![(0, shards[0].clone()), (2, shards[2].clone())]).await?;
        data.resize(decoded.len(), 0);
        assert_eq!(decoded, data);
    }
    Ok(())
}
