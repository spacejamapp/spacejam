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
    let scale = if cfg!(feature = "full") {
        specjam::Scale::Full
    } else {
        specjam::Scale::Tiny
    };
    let test = registry.erasure(scale)?.test(test)?;
    let (mut data, shards) = codec::decode::<(Vec<u8>, Vec<Vec<u8>>)>(test.input.expect_bin()?)?;

    let n = erasure::Config::default().original;
    let recovery_pairs = || -> Vec<(usize, Vec<u8>)> {
        let mut pairs: Vec<_> = (0..n - 1).map(|i| (i, shards[i].clone())).collect();
        pairs.push((n, shards[n].clone()));
        pairs
    };

    // testing sync
    {
        let encoded = erasure::encode_sync(data.clone())?;
        assert_eq!(encoded, shards);

        let decoded = erasure::decode_sync(recovery_pairs())?;
        data.resize(decoded.len(), 0);
        assert_eq!(decoded, data);
    }

    // testing async
    {
        let encoded = erasure::encode(data.clone()).await?;
        assert_eq!(encoded, shards);

        let decoded = erasure::decode(recovery_pairs()).await?;
        data.resize(decoded.len(), 0);
        assert_eq!(decoded, data);
    }
    Ok(())
}
