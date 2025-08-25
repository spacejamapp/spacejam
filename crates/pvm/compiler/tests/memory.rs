//! Tests for the Memory module

use pvmc::{trap, Memory};

const INIT_VALUE: u8 = 1;
const REGION_SIZE: usize = pvm::PAGE_SIZE as usize;
const REGION_START: u32 = pvm::ZONE_SIZE as u32;
const REGION_END: u32 = REGION_START + REGION_SIZE as u32;
const UNALLOCATED_ADDR: u32 = pvm::ZONE_SIZE as u32 * 2;

/// Generate a read test for the given memory
fn gen_read(mut memory: Memory) -> anyhow::Result<()> {
    let data = memory.read_bytes(REGION_START, REGION_SIZE as u32);
    assert_eq!(data, vec![INIT_VALUE; REGION_SIZE]);

    // try writing to read-only memory
    {
        let Err(info) = trap::with(|| {
            memory.write_bytes(REGION_START, &[2; REGION_SIZE]);
        }) else {
            panic!("should trap");
        };
        assert_eq!(info.signal, libc::SIGSEGV);
        assert_eq!(info.code, 2); // SEGV_ACCERR
    }

    // try accessing unallocated memory
    {
        let Err(info) = trap::with(|| {
            let slice = memory.read_bytes(UNALLOCATED_ADDR, 1);
            slice[0]
        }) else {
            panic!("should trap");
        };
        assert_eq!(info.signal, libc::SIGSEGV);
        assert_eq!(info.code, 2);
    }

    Ok(())
}

/// Generate a write test for the given memory
fn gen_write(mut memory: Memory) -> anyhow::Result<()> {
    // test read
    let data = memory.read_bytes(REGION_START, REGION_SIZE as u32);
    assert_eq!(data, vec![1; REGION_SIZE]);

    // test write
    memory.write_bytes(REGION_START, &[2; REGION_SIZE]);
    assert_eq!(
        memory.read_bytes(REGION_START, REGION_SIZE as u32),
        vec![2; REGION_SIZE]
    );

    // try writing to unallocated memory
    let Err(info) = trap::with(|| {
        memory.write_bytes(REGION_END, &[3; REGION_SIZE]);
    }) else {
        panic!("should trap");
    };

    // check that the trap info is correct
    assert_eq!(info.signal, libc::SIGSEGV);
    assert_eq!(info.code, 2);
    Ok(())
}

#[test]
fn test_read() -> anyhow::Result<()> {
    let memory = Memory::new(
        &pvm::Memory::default().with_ro_data(vec![INIT_VALUE; REGION_SIZE], REGION_START),
    )?;
    gen_read(memory)
}

#[test]
fn test_write() -> anyhow::Result<()> {
    let memory = Memory::new(
        &pvm::Memory::default().with_rw_data(vec![INIT_VALUE; REGION_SIZE], REGION_START),
    )?;
    gen_write(memory)
}

#[test]
fn test_stack() -> anyhow::Result<()> {
    let mut memory = Memory::new(&pvm::Memory::default().with_stack(REGION_START..REGION_END))?;
    memory.write_bytes(REGION_START, &[INIT_VALUE; REGION_SIZE]);
    gen_write(memory)
}

#[test]
fn test_args() -> anyhow::Result<()> {
    let memory = Memory::new(
        &pvm::Memory::default().with_args(vec![INIT_VALUE; REGION_SIZE], REGION_START),
    )?;
    gen_read(memory)
}

#[test]
fn test_heap() -> anyhow::Result<()> {
    let mut memory = Memory::new(&pvm::Memory::default().with_heap(REGION_START..REGION_END))?;
    // allocate takes page number, not address
    let page_num = REGION_START / pvm::PAGE_SIZE as u32;
    memory.allocate(page_num, 1)?;
    memory.write_bytes(REGION_START, &[INIT_VALUE; REGION_SIZE]);
    gen_write(memory)
}

#[test]
fn test_trap_write_to_readonly() -> anyhow::Result<()> {
    // Create memory with read-only region
    let memory =
        Memory::new(&pvm::Memory::default().with_ro_data(vec![1; 1024], pvm::ZONE_SIZE as u32))?;

    // Get base pointer for direct access
    let memory_ptr = memory.base();

    // Try to write to read-only memory - should trap
    let result = trap::with(move || unsafe {
        let ptr = memory_ptr.add(pvm::ZONE_SIZE as usize) as *mut u8;
        *ptr = 0xFF; // Direct write to trigger SIGSEGV
    });

    // Verify we caught the SIGSEGV trap
    match result {
        Err(trap_info) => {
            assert_eq!(trap_info.signal, libc::SIGSEGV);
            assert_eq!(trap_info.code, 2); // SEGV_ACCERR
            println!(
                "Successfully caught SIGSEGV for read-only write: {:?}",
                trap_info
            );
        }
        Ok(_) => panic!("Expected SIGSEGV trap when writing to read-only memory"),
    }

    Ok(())
}

#[test]
fn test_multiple_traps() -> anyhow::Result<()> {
    println!("Testing multiple traps in sequence...");

    // Create memory with read-only region
    let memory =
        Memory::new(&pvm::Memory::default().with_ro_data(vec![1; 1024], pvm::ZONE_SIZE as u32))?;

    let memory_ptr = memory.base();

    // First trap: write to read-only
    println!("First trap: write to read-only...");
    let result1 = trap::with(move || unsafe {
        let ptr = memory_ptr.add(pvm::ZONE_SIZE as usize) as *mut u8;
        *ptr = 0xFF;
    });
    assert!(result1.is_err());
    println!("First trap caught successfully");

    // Second trap: access unallocated
    println!("Second trap: access unallocated...");
    let result2 = trap::with(move || unsafe {
        let ptr = memory_ptr.add(UNALLOCATED_ADDR as usize);
        *ptr
    });
    assert!(result2.is_err());
    println!("Second trap caught successfully");

    println!("Both traps handled correctly!");
    Ok(())
}
