//! Tests for the Memory module

use pvmc::{trap, Memory, MemoryLike};

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
        let Err(info) = trap::with(|| memory.write_bytes(REGION_START, &[2; REGION_SIZE])) else {
            panic!("should trap");
        };

        assert!(info.signal == libc::SIGSEGV || info.signal == libc::SIGBUS);
    }

    // try accessing unallocated memory
    {
        let Err(info) = trap::with(|| memory.read_bytes(UNALLOCATED_ADDR, 1)[0]) else {
            panic!("should trap");
        };
        assert!(info.signal == libc::SIGSEGV || info.signal == libc::SIGBUS);
    }

    /*     // try accessing unallocated memory near the allocated memory
    {
        let Err(info) = trap::with(|| {
            let slice = memory.read_bytes(REGION_END, 1);
            slice[0]
        }) else {
            panic!("should trap on reading unallocated memory (REGION_END + 1)");
        };
        assert!(info.signal == libc::SIGSEGV || info.signal == libc::SIGBUS);
    } */

    Ok(())
}

/// Generate a write test for the given memory
fn gen_write(mut memory: Memory) -> anyhow::Result<()> {
    let data = memory.read_bytes(REGION_START, REGION_SIZE as u32);
    assert_eq!(data, vec![1; REGION_SIZE]);

    // try writing to writable memory
    {
        memory.write_bytes(REGION_START, &[2; REGION_SIZE]);
        assert_eq!(
            memory.read_bytes(REGION_START, REGION_SIZE as u32),
            vec![2; REGION_SIZE]
        );
    }

    // try writing to unallocated memory
    {
        let Err(info) = trap::with(|| {
            memory.write_bytes(UNALLOCATED_ADDR, &[3; REGION_SIZE]);
        }) else {
            panic!("should trap on unallocated memory access");
        };
        assert!(info.signal == libc::SIGSEGV || info.signal == libc::SIGBUS);
    }

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
