//! Memory write tests

use pvmi::{Access, Error, Memory, Page, PAGE_SIZE};
use smallvec::SmallVec;

#[test]
fn inaccessible() {
    let memory = Memory::default();
    assert_eq!(
        memory.read_bytes(0, 0, 1),
        Err(Error::MemoryInaccessible(0))
    );
}

#[test]
fn immutable() {
    let mut memory = Memory::default();
    memory.pages.insert(
        0,
        Page {
            data: SmallVec::new(),
            access: Access::Immutable,
        },
    );
    assert_eq!(
        memory.write_bytes(0, 0, &[0]),
        Err(Error::MemoryImmutable(0))
    );
}

#[test]
fn write() {
    let mut memory = Memory::default();
    memory.pages.insert(
        0,
        Page {
            data: SmallVec::new(),
            access: Access::Mutable,
        },
    );

    assert!(memory.write_bytes(0, 0, &[0]).is_ok());
    assert_eq!(memory.read_bytes(0, 0, 1), Ok(vec![0]));
}

#[test]
fn write_offset() {
    let mut memory = Memory::default();
    memory.pages.insert(
        0,
        Page {
            data: SmallVec::new(),
            access: Access::Mutable,
        },
    );

    let offset = 10;
    let value = 42;
    let data = vec![value; offset];
    assert!(memory.write_bytes(0, offset as u64, &data).is_ok());
    assert_eq!(memory.read_bytes(0, offset as u64, offset as u64), Ok(data));
}

#[test]
fn read_inaccessible() {
    let mut memory = Memory::default();
    memory.pages.insert(
        0,
        Page {
            data: SmallVec::new(),
            access: Access::Mutable,
        },
    );

    let data = vec![42; 1];
    let page = 0;
    assert!(memory.write_bytes(0, 0, &data).is_ok());
    assert_eq!(
        memory.read_bytes(0, 0, PAGE_SIZE as u64 + 1),
        Err(Error::MemoryInaccessible(page as u32))
    );
}
