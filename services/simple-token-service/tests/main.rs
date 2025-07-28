//! Basic VM tests

use simple_token_service::{Instruction, SERVICE};
use testing::Jam;

const SERVICE_ID: u32 = 500;

#[test]
fn test_refine() {
    testing::util::init_logger();

    let mut jam = Jam::default();
    jam.add_service(SERVICE_ID, SERVICE.to_vec());

    let package = jam
        .send(SERVICE_ID, vec![])
        .expect("failed to send work item");

    let refined = jam.refine(&package).expect("failed to refine");
    assert!(
        refined.executed.is_ok(),
        "refine execution failed: {:?}",
        refined.executed.exec
    );
}
