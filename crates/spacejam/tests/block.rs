use runtime::{storage::MemoryDb, Runtime, Validator};
use spacejam::{chain, validator::LocalValidator, Test};
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn test_block_sealing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let validator = LocalValidator::dev();
    let runtime: Runtime<Test> = Runtime::new(validator, MemoryDb::default(), ());
    let spec = chain::Spec::dev().parse().unwrap();

    runtime
        .import_genesis(spec.genesis_header, &spec.genesis_state)
        .await
        .unwrap();

    let _block = runtime
        .author()
        .author(42)
        .await
        .expect("failed to author block");

    // runtime.validate(&block.header).await.unwrap();
}
