//! Tests for the RRR instruction.

use crate::init_tracing;
use pvm::{Pvm, Status};

#[test]
fn add_32() {
    init_tracing();

    let mut pvm = Pvm::default()
        .registers([0, 0, 0, 0, 0, 0, 0, 1, 2, 0, 0, 0, 0])
        .gas(10000);

    pvm.interp([0, 0, 3, 0xbe, 135, 9, 1])
        .expect("interp failed");
    assert_eq!(pvm.status, Status::Trap);
    assert_eq!(pvm.registers, [0, 0, 0, 0, 0, 0, 0, 1, 2, 3, 0, 0, 0]);
}

#[test]
fn and() {
    let mut pvm = Pvm::default()
        .registers([0, 0, 0, 0, 0, 0, 0, 5, 3, 0, 0, 0, 0])
        .gas(10000);

    pvm.interp([0, 0, 3, 210, 135, 9, 1])
        .expect("interp failed");
    assert_eq!(pvm.status, Status::Trap);
    assert_eq!(pvm.registers, [0, 0, 0, 0, 0, 0, 0, 5, 3, 1, 0, 0, 0]);
}
