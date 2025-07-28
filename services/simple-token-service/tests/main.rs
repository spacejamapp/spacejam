//! Basic VM tests

use podec::Encode;
use simple_token_service::{Instruction, SERVICE};
use testing::Jam;

const SERVICE_ID: u32 = 500;

#[test]
fn test_mint() {
    testing::util::init_logger();
    let mut jam = Jam::default();
    jam.add_service(SERVICE_ID, SERVICE.to_vec());

    // 1. send a mint instruction
    let instr = vec![Instruction::Mint { to: 0, amount: 100 }];
    let package = jam
        .send(SERVICE_ID, instr.encode())
        .expect("failed to send work item");

    // 2. refine the package
    let report = jam.refine(&package).expect("failed to refine");

    // 3. run accumulate
    jam.accumulate(&report).expect("failed to accumulate");
}
