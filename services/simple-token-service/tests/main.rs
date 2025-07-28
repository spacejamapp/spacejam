//! Basic VM tests

use podec::Encode;
use simple_token_service::{Holders, Instruction, SERVICE};
use testing::Jam;

const SERVICE_ID: u32 = 500;
const ALICE: u32 = 0;

#[test]
fn test_mint() {
    testing::util::init_logger();
    let mut jam = Jam::default();
    jam.add_service(SERVICE_ID, SERVICE.to_vec());

    // 1. send a mint instruction
    let amount = 100;
    let instr = vec![Instruction::Mint { to: ALICE, amount }];
    let info = jam
        .execute(SERVICE_ID, instr.encode())
        .expect("failed to execute work item");

    // 2. check the balance
    let holders: Holders = info
        .get_storage(SERVICE_ID, &Holders::key())
        .expect("failed to get holders");
    assert_eq!(holders.balance(ALICE), amount);
}
