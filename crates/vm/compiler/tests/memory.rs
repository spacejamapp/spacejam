//! Tests for the Memory module

use pvm::score::OpaqueHash;
use pvmc::{Memory, MemoryLike, trap};

const INIT_VALUE: u8 = 1;
const REGION_SIZE: usize = pvm::PAGE_SIZE as usize;
const REGION_START: u32 = pvm::ZONE_SIZE as u32;
const REGION_END: u32 = REGION_START + REGION_SIZE as u32;
const UNALLOCATED_ADDR: u32 = pvm::ZONE_SIZE as u32 * 2;

const HASH_READ: OpaqueHash = [0x11; 32];
const HASH_WRITE: OpaqueHash = [0x22; 32];
const HASH_STACK: OpaqueHash = [0x33; 32];
const HASH_ARGS: OpaqueHash = [0x44; 32];
const HASH_HEAP: OpaqueHash = [0x55; 32];
const HASH_REUSE_RW: OpaqueHash = [0x66; 32];
const HASH_REUSE_HEAP: OpaqueHash = [0x77; 32];

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
        HASH_READ,
        &pvm::Memory::default().with_ro_data(vec![INIT_VALUE; REGION_SIZE], REGION_START),
    )?;
    gen_read(memory)
}

#[test]
fn test_write() -> anyhow::Result<()> {
    let memory = Memory::new(
        HASH_WRITE,
        &pvm::Memory::default().with_rw_data(vec![INIT_VALUE; REGION_SIZE], REGION_START),
    )?;
    gen_write(memory)
}

#[test]
fn test_stack() -> anyhow::Result<()> {
    let mut memory = Memory::new(
        HASH_STACK,
        &pvm::Memory::default().with_stack(REGION_START..REGION_END),
    )?;
    memory.write_bytes(REGION_START, &[INIT_VALUE; REGION_SIZE]);
    gen_write(memory)
}

#[test]
fn test_args() -> anyhow::Result<()> {
    let memory = Memory::new(
        HASH_ARGS,
        &pvm::Memory::default().with_args(vec![INIT_VALUE; REGION_SIZE], REGION_START),
    )?;
    gen_read(memory)
}

#[test]
fn test_heap() -> anyhow::Result<()> {
    let mut memory = Memory::new(
        HASH_HEAP,
        &pvm::Memory::default().with_heap(REGION_START..REGION_END),
    )?;
    // allocate takes page number, not address
    let page_num = REGION_START / pvm::PAGE_SIZE as u32;
    memory.allocate(page_num, 1)?;
    memory.write_bytes(REGION_START, &[INIT_VALUE; REGION_SIZE]);
    gen_write(memory)
}

#[cfg(target_os = "linux")]
mod pool {
    use super::*;

    #[test]
    fn slot_reuse_rw_writes() -> anyhow::Result<()> {
        let initial = vec![0xAA_u8; REGION_SIZE];
        let pmem = pvm::Memory::default().with_rw_data(initial.clone(), REGION_START);

        // Phase 1: overwrite initial state with a recognizable pattern.
        {
            let mut memory = Memory::new(HASH_REUSE_RW, &pmem)?;
            memory.write_bytes(REGION_START, &[0xCC; REGION_SIZE]);
            assert_eq!(
                memory.read_bytes(REGION_START, REGION_SIZE as u32),
                &[0xCC; REGION_SIZE][..]
            );
        }

        // Phase 2: same layout. Must see freshly-initialized data, not phase 1's writes.
        {
            let memory = Memory::new(HASH_REUSE_RW, &pmem)?;
            assert_eq!(
                memory.read_bytes(REGION_START, REGION_SIZE as u32),
                initial.as_slice(),
                "rw region must be reset to initial state on slot reuse"
            );
        }

        Ok(())
    }

    #[test]
    fn slot_reuse_heap_allocations() -> anyhow::Result<()> {
        let pmem = pvm::Memory::default().with_heap(REGION_START..REGION_END);
        let page_num = REGION_START / pvm::PAGE_SIZE as u32;

        // Phase 1: allocate the page, write a pattern, drop.
        {
            let mut memory = Memory::new(HASH_REUSE_HEAP, &pmem)?;
            memory.allocate(page_num, 1)?;
            memory.write_bytes(REGION_START, &[0xCC; REGION_SIZE]);
            assert_eq!(
                memory.read_bytes(REGION_START, REGION_SIZE as u32),
                &[0xCC; REGION_SIZE][..]
            );
        }

        // Phase 2: same layout, but no `allocate` call. The page must be PROT_NONE again.
        {
            let memory = Memory::new(HASH_REUSE_HEAP, &pmem)?;
            let Err(info) = trap::with(|| memory.read_bytes(REGION_START, 1)[0]) else {
                panic!("heap page should be unmapped on slot reuse");
            };
            assert!(info.signal == libc::SIGSEGV || info.signal == libc::SIGBUS);
        }

        Ok(())
    }

    #[test]
    fn image_cache_hit_preserves_ro_init() -> anyhow::Result<()> {
        const HASH: OpaqueHash = [0x88; 32];
        let initial = vec![0xAA_u8; REGION_SIZE];
        let pmem = pvm::Memory::default().with_ro_data(initial.clone(), REGION_START);

        // Phase 1: cold image build.
        {
            let memory = Memory::new(HASH, &pmem)?;
            assert_eq!(
                memory.read_bytes(REGION_START, REGION_SIZE as u32),
                initial.as_slice()
            );
        }

        // Phase 2: cache hit. Same memfd, fresh slot binding.
        {
            let mut memory = Memory::new(HASH, &pmem)?;
            assert_eq!(
                memory.read_bytes(REGION_START, REGION_SIZE as u32),
                initial.as_slice(),
                "ro init must be visible via image cache hit"
            );

            let Err(info) = trap::with(|| memory.write_bytes(REGION_START, &[0; REGION_SIZE]))
            else {
                panic!("ro range must trap on write");
            };
            assert!(info.signal == libc::SIGSEGV || info.signal == libc::SIGBUS);
        }

        Ok(())
    }

    #[test]
    fn slot_reuse_swaps_image() -> anyhow::Result<()> {
        const HASH_A: OpaqueHash = [0x99; 32];
        const HASH_B: OpaqueHash = [0xBB; 32];
        let init_a = vec![0xAA_u8; REGION_SIZE];
        let init_b = vec![0xCD_u8; REGION_SIZE];
        let pmem_a = pvm::Memory::default().with_rw_data(init_a, REGION_START);
        let pmem_b = pvm::Memory::default().with_rw_data(init_b.clone(), REGION_START);

        // Phase 1: bind program A, dirty its RW range, drop.
        {
            let mut memory = Memory::new(HASH_A, &pmem_a)?;
            memory.write_bytes(REGION_START, &[0xCC; REGION_SIZE]);
        }

        // Phase 2: different program → different image. Must see B's init bytes,
        // not A's dirty writes or A's init bytes.
        {
            let memory = Memory::new(HASH_B, &pmem_b)?;
            assert_eq!(
                memory.read_bytes(REGION_START, REGION_SIZE as u32),
                init_b.as_slice(),
                "second program's rw init must override prior slot binding"
            );
        }

        Ok(())
    }
}
