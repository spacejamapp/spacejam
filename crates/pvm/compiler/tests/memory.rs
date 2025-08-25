//! Tests for the Memory module

use pvmc::Memory;

#[test]
fn test_memory_creation() {
    let mut parser_memory = pvm::Memory::default();
    parser_memory.memory.insert(16, (vec![1, 2, 3, 4], false));
    parser_memory.memory.insert(32, (vec![5, 6, 7, 8], true));

    let memory = Memory::new(&parser_memory).expect("Failed to create memory");
    unsafe {
        let ro_data = memory.read_bytes(0x10000, 4);
        assert_eq!(ro_data, &[1, 2, 3, 4]);

        let rw_data = memory.read_bytes(0x20000, 4);
        assert_eq!(rw_data, &[5, 6, 7, 8]);
    }
}

#[test]
fn test_sbrk() {
    let parser_memory = pvm::Memory::default();
    let mut memory = Memory::new(&parser_memory).expect("Failed to create memory");
    memory.sbrk(48, 2).expect("Failed to allocate");

    unsafe {
        memory.write_bytes(0x30000, &[0xAA; 16]);
        let data = memory.read_bytes(0x30000, 16);
        assert_eq!(data, &[0xAA; 16]);
    }
}

#[test]
fn test_read_write() {
    let mut parser_memory = pvm::Memory::default();
    parser_memory.memory.insert(64, (vec![0; 4096], true));
    let mut memory = Memory::new(&parser_memory).expect("Failed to create memory");

    unsafe {
        memory.write(0x40000, 0x12345678u32);
        let value: u32 = memory.read(0x40000);
        assert_eq!(value, 0x12345678);
        let data = vec![0xFF; 32];
        memory.write_bytes(0x40100, &data);
        let read_data = memory.read_bytes(0x40100, 32);
        assert_eq!(read_data, &data[..]);
    }
}

#[test]
fn test_drop() {
    let parser_memory = pvm::Memory::default();
    {
        let _memory = Memory::new(&parser_memory).expect("Failed to create memory");
    }
}

#[test]
fn test_memory_with_multiple_pages() {
    let mut parser_memory = pvm::Memory::default();

    for i in 10..20 {
        let data = vec![i as u8; 100];
        let writable = i % 2 == 0;
        parser_memory.memory.insert(i, (data, writable));
    }

    let mut memory = Memory::new(&parser_memory).expect("Failed to create memory");
    unsafe {
        for i in 10..20 {
            let addr = i * pvm::PAGE_SIZE as u32;
            let data = memory.read_bytes(addr, 1);
            assert_eq!(data[0], i as u8);
            if i % 2 == 0 {
                memory.write_bytes(addr + 1, &[0xFF]);
                let written = memory.read_bytes(addr + 1, 1);
                assert_eq!(written[0], 0xFF);
            }
        }
    }
}
