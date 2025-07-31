//! Basic VM tests

use jam_null_authorizer::SERVICE;
use testing::Jam;

const AUTHORIZER: u32 = 500;

#[test]
fn test_null_authorizer() {
    let mut jam = Jam::default().with_auth(AUTHORIZER, SERVICE.to_vec());
    let package = jam
        .send(AUTHORIZER, vec![])
        .expect("failed to send work item");

    let result = jam.authorize(&package, 0).expect("failed to authorize");
    assert!(result.is_ok());
}
