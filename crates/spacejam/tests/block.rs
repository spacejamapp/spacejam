use runtime::{
    storage::{KVStorage, MemoryDb},
    Runtime, Storage,
};
use spacejam::{chain, validator::LocalValidator, Test};
use tracing_subscriber::EnvFilter;

#[tokio::test]
async fn test_block_sealing() {
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

#[test]
fn genesis_state_root() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let spec = chain::Spec::dev().parse().unwrap();
    let memdb = MemoryDb::default();

    for (k, v) in spec.genesis_state {
        memdb.set(k, v).unwrap();
    }

    // This is the root calculated by polkajam
    //
    // 0x566a95e5ae04266c715387e8f6db64aaa446afa9f168f2bb7fac96082a443bd7
    let root = memdb.root().unwrap();
    println!("root: 0x{}", hex::encode(root));
}
