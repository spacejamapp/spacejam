//! Memory write tests

use parser::{Memory, PAGE_SIZE};

#[test]
fn inaccessible() {
    let memory = Memory::default();
    // Trying to read from unallocated memory should fail
    assert!(memory.read_bytes(0, 1).is_err());
}

#[test]
fn immutable() {
    let mut memory = Memory::default();

    // Insert an immutable page
    let page_data = vec![0u8; PAGE_SIZE as usize];
    memory.memory.insert(0, (page_data, false));

    // Reading from immutable memory should succeed
    assert!(memory.read_bytes(0, 1).is_ok());
    assert_eq!(memory.read_bytes(0, 1).unwrap(), vec![0]);

    // Trying to write to immutable memory should fail
    assert!(memory.write_bytes(0, &[42]).is_err());
}

#[test]
fn write() {
    let mut memory = Memory::default();

    // Insert a mutable page
    let page_data = vec![0u8; PAGE_SIZE as usize];
    memory.memory.insert(0, (page_data, true));

    // Writing should succeed
    assert!(memory.write_bytes(0, &[42]).is_ok());
    assert_eq!(memory.read_bytes(0, 1).unwrap(), vec![42]);
}

#[test]
fn write_offset() {
    let mut memory = Memory::default();

    // Insert a mutable page
    let page_data = vec![0u8; PAGE_SIZE as usize];
    memory.memory.insert(0, (page_data, true));

    let offset = 10;
    let value = 42;
    let data = vec![value; offset];

    assert!(memory.write_bytes(offset as u32, &data).is_ok());
    assert_eq!(
        memory.read_bytes(offset as u32, offset as u32).unwrap(),
        data
    );
}

#[test]
fn cross_page_read() {
    let mut memory = Memory::default();

    // Insert two pages
    let page_data1 = vec![1u8; PAGE_SIZE as usize];
    let page_data2 = vec![2u8; PAGE_SIZE as usize];
    memory.memory.insert(0, (page_data1, true));
    memory.memory.insert(1, (page_data2, true));

    // Read across page boundary
    let addr = PAGE_SIZE as u32 - 5;
    let result = memory.read_bytes(addr, 10).unwrap();

    assert_eq!(result.len(), 10);
    assert_eq!(&result[0..5], &[1u8; 5]);
    assert_eq!(&result[5..10], &[2u8; 5]);
}

#[test]
fn cross_page_write() {
    let mut memory = Memory::default();

    // Insert two mutable pages
    let page_data1 = vec![0u8; PAGE_SIZE as usize];
    let page_data2 = vec![0u8; PAGE_SIZE as usize];
    memory.memory.insert(0, (page_data1, true));
    memory.memory.insert(1, (page_data2, true));

    // Write across page boundary
    let addr = PAGE_SIZE as u32 - 5;
    let data = vec![42u8; 10];

    assert!(memory.write_bytes(addr, &data).is_ok());

    // Verify the write
    let result = memory.read_bytes(addr, 10).unwrap();
    assert_eq!(result, data);
}

#[test]
fn write_to_readonly_page_fails() {
    let mut memory = Memory::default();

    // Insert a read-only page
    let page_data = vec![0u8; PAGE_SIZE as usize];
    memory.memory.insert(0, (page_data, false));

    // Writing should fail
    assert!(memory.write_bytes(0, &[42]).is_err());

    // Reading should still work
    assert_eq!(memory.read_bytes(0, 1).unwrap(), vec![0]);
}

#[test]
fn allocate_pages() {
    let mut memory = Memory::default();

    // Allocate 3 pages starting from page 5
    assert!(memory.allocate(5, 3).is_ok());

    // Verify pages were allocated
    assert!(memory.memory.contains_key(&5));
    assert!(memory.memory.contains_key(&6));
    assert!(memory.memory.contains_key(&7));

    // Verify they are writable and read as zero-filled
    for page_num in 5..8 {
        let (_, writable) = &memory.memory[&page_num];
        assert!(*writable, "Page {} should be writable", page_num);
        let data = memory
            .read_bytes(page_num * PAGE_SIZE as u32, PAGE_SIZE as u32)
            .unwrap();
        assert_eq!(data.len(), PAGE_SIZE as usize);
        assert!(
            data.iter().all(|&b| b == 0),
            "Page {} should be zero-filled",
            page_num
        );
    }

    // Should be able to write to allocated pages
    assert!(memory.write_bytes(5 * PAGE_SIZE as u32, &[42]).is_ok());
}

#[test]
fn read_beyond_allocated_memory() {
    let mut memory = Memory::default();

    // Insert a page with only partial data
    let page_data = vec![42u8; 100];
    memory.memory.insert(0, (page_data, false));

    // Read beyond the allocated data should return zeros
    let result = memory.read_bytes(50, 100).unwrap();
    assert_eq!(result.len(), 100);
    assert_eq!(&result[0..50], &[42u8; 50]);
    assert_eq!(&result[50..100], &[0u8; 50]);
}

#[test]
fn write_extends_page_data() {
    let mut memory = Memory::default();

    // Insert a page with minimal data
    let page_data = vec![0u8; 10];
    memory.memory.insert(0, (page_data, true));

    // Write beyond current page data
    assert!(memory.write_bytes(50, &[42, 43, 44]).is_ok());

    // Verify page was extended and data was written
    let result = memory.read_bytes(50, 3).unwrap();
    assert_eq!(result, vec![42, 43, 44]);

    // Verify page was extended with zeros in between
    let result = memory.read_bytes(10, 40).unwrap();
    assert!(result.iter().all(|&b| b == 0));
}

#[test]
fn atomic_write_validation() {
    let mut memory = Memory::default();

    // Insert two pages: one writable, one read-only
    memory
        .memory
        .insert(0, (vec![0u8; PAGE_SIZE as usize], true));
    memory
        .memory
        .insert(1, (vec![0u8; PAGE_SIZE as usize], false));

    // Try to write across both pages - should fail atomically
    let addr = PAGE_SIZE as u32 - 5;
    let data = vec![42u8; 10];

    assert!(memory.write_bytes(addr, &data).is_err());

    // Verify no data was written to either page (atomic failure)
    let result = memory.read_bytes(addr, 5).unwrap();
    assert_eq!(result, vec![0u8; 5]);
}
