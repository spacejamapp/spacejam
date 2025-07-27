//! Basic VM tests

use testing::Jam;

const PROGRAM: &[u8] = include_bytes!("../../../target/jam/jam-null-authorizer.jam");
const AUTHORIZER: u32 = 500;

#[test]
fn test_null_authorizer() {
    testing::init_logger();

    let mut jam = Jam::default().with_auth(AUTHORIZER, PROGRAM.to_vec());
    let package = jam
        .send(AUTHORIZER, vec![])
        .expect("failed to send work item");

    let result = jam.authorize(&package, 0).expect("failed to authorize");
    assert!(result.is_ok());
}
