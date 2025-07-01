/* use runtime::{storage::MemoryDb, Runtime};
use spacejam::{chain, validator::LocalValidator, Test};
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn test_sealing() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let validator = LocalValidator::try_from("5".to_string()).unwrap();
    let runtime: Runtime<Test> = Runtime::new(validator, MemoryDb::default(), ());
    let spec = chain::Spec::dev().parse().unwrap();

    runtime
        .import_genesis(spec.genesis_header, &spec.genesis_state)
        .await
        .unwrap();

    let block = runtime
        .author()
        .author(42)
        .await
        .expect("failed to author block");

    runtime.validate(&block.header).await.unwrap();
}
 */
