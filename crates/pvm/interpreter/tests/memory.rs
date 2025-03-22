//! Memory write tests

use pvmi::{Access, Error, Memory, Page, PAGE_SIZE};
use smallvec::SmallVec;

#[test]
fn inaccessible() {
    let memory = Memory::default();
    assert_eq!(memory.read_bytes(0, 0, 1), Err(Error::MemoryInaccessible));
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
    assert_eq!(memory.write_bytes(0, 0, &[0]), Err(Error::MemoryImmutable));
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
fn write_multiple() {
    let mut memory = Memory::default();
    for i in 0..3 {
        memory.pages.insert(
            i,
            Page {
                data: SmallVec::new(),
                access: Access::Mutable,
            },
        );
    }

    let three_pages = 3 * PAGE_SIZE as usize;
    let data = vec![42; three_pages];
    let page = vec![42; PAGE_SIZE as usize];

    assert!(memory.write_bytes(0, 0, &data).is_ok());
    for i in 0..3 {
        assert_eq!(
            memory.read_bytes(i * PAGE_SIZE, 0, PAGE_SIZE as u64),
            Ok(page.clone())
        );
    }
}

#[test]
fn write_partial() {
    let mut memory = Memory::default();
    for i in 0..2 {
        memory.pages.insert(
            i,
            Page {
                data: SmallVec::new(),
                access: Access::Mutable,
            },
        );
    }

    let full = 6666 as usize;
    let page1 = PAGE_SIZE as usize;
    let page2 = full - page1;
    let data = vec![42; full];
    assert!(memory.write_bytes(0, 0, &data).is_ok());
    assert_eq!(memory.read_bytes(0, 0, page1 as u64), Ok(vec![42; page1]));
    assert_eq!(
        memory.read_bytes(PAGE_SIZE, 0, page2 as u64),
        Ok(vec![42; page2])
    );
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
    assert!(memory.write_bytes(0, 0, &data).is_ok());
    assert_eq!(
        memory.read_bytes(0, 0, PAGE_SIZE as u64),
        Err(Error::MemoryInaccessible)
    );
}
