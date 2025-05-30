//! The memory of a program.

use crate::StandardProgramBlob;
use std::{borrow::Cow, collections::BTreeMap};

/// The memory of a program.
#[derive(Default)]
pub struct Memory<'a> {
    /// The memory (µ).
    pub memory: BTreeMap<u32, (Cow<'a, [u8]>, bool)>,
}

impl<'a> Memory<'a> {
    /// Initialize the memory.
    pub fn init(blob: &StandardProgramBlob<'a>, args: &'a [u8]) -> Self {
        let mut memory = Self::default();
        let funp = |x: u64| x.div_ceil(crate::PAGE_SIZE) * crate::PAGE_SIZE;
        let funz = |x: u64| x.div_ceil(crate::ZONE_SIZE) * crate::ZONE_SIZE;
        let (ro_len, rw_len, args_len) = (
            blob.ro_data.len() as u64,
            blob.rw_data.len() as u64,
            args.len() as u64,
        );

        // RO data: Z_Z ≤ i < Z_Z + |o|
        let mut start = crate::ZONE_SIZE;
        tracing::debug!(
            "Memory layout - RO data: 0x{:x}..0x{:x} (len={})",
            start,
            start + ro_len,
            ro_len
        );
        memory.insert_pages_cow(blob.ro_data.clone(), start, false, crate::PAGE_SIZE);

        // RO padding: Z_Z + |o| ≤ i < Z_Z + P(|o|)
        let ro_padding_len = funp(ro_len) - ro_len;
        start += ro_len;
        if ro_padding_len > 0 {
            tracing::debug!(
                "Memory layout - RO padding: 0x{:x}..0x{:x} (len={})",
                start,
                start + ro_padding_len,
                ro_padding_len
            );
            memory.insert_pages_owned(
                vec![0; ro_padding_len as usize],
                start,
                false,
                crate::PAGE_SIZE,
            );
        }

        // RW data: 2*Z_Z + Z(|o|) ≤ i < 2*Z_Z + Z(|o|) + |w|
        start = 2 * crate::ZONE_SIZE + funz(ro_len);
        tracing::debug!(
            "Memory layout - RW data: 0x{:x}..0x{:x} (len={}, funz(ro_len)=0x{:x})",
            start,
            start + rw_len,
            rw_len,
            funz(ro_len)
        );
        memory.insert_pages_cow(blob.rw_data.clone(), start, true, crate::PAGE_SIZE);

        // RW padding + heap: 2*Z_Z + Z(|o|) + |w| ≤ i < 2*Z_Z + Z(|o|) + P(|w|) + z*Z_P
        start += rw_len;
        let rw_padding_len = funp(rw_len) - rw_len;
        let heap_len = blob.rw_data_padding_pages as u64 * crate::PAGE_SIZE;
        // Increase heap allocation to provide much more space for services that need it
        let extra_heap_len = 64 * crate::PAGE_SIZE; // Add 64 more pages (256KB extra)
        let total_rw_padding_heap_len = rw_padding_len + heap_len + extra_heap_len;
        if total_rw_padding_heap_len > 0 {
            tracing::debug!(
                "Memory layout - RW padding + heap: 0x{:x}..0x{:x} (rw_padding={}, heap={}, extra_heap={})",
                start,
                start + total_rw_padding_heap_len,
                rw_padding_len,
                heap_len,
                extra_heap_len
            );
            memory.insert_pages_owned(
                vec![0; total_rw_padding_heap_len as usize],
                start,
                true,
                crate::PAGE_SIZE,
            );
        }

        // Stack: 2^32 - 2*Z_Z - Z_I - P(s) ≤ i < 2^32 - 2*Z_Z - Z_I
        let stack_padded_len = funp(blob.stack_size as u64);
        start = crate::PVM_MEMORY_SIZE
            - 2 * crate::ZONE_SIZE
            - crate::PVM_INIT_DATA_SIZE
            - stack_padded_len;
        memory.insert_pages_owned(
            vec![0; stack_padded_len as usize],
            start,
            true,
            crate::PAGE_SIZE,
        );

        // Args: 2^32 - Z_Z - Z_I ≤ i < 2^32 - Z_Z - Z_I + |a|
        start = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE;
        memory.insert_pages_cow(Cow::Borrowed(args), start, false, crate::PAGE_SIZE);

        // Args padding: 2^32 - Z_Z - Z_I + |a| ≤ i < 2^32 - Z_Z - Z_I + P(|a|)
        start += args_len;
        let args_padded_len = funp(args_len);
        let args_padding_len = args_padded_len - args_len;
        if args_padding_len > 0 {
            memory.insert_pages_owned(
                vec![0; args_padding_len as usize],
                start,
                false,
                crate::PAGE_SIZE,
            );
        }
        memory
    }

    /// Insert pages from a Cow.
    pub fn insert_pages_cow(
        &mut self,
        data: Cow<'a, [u8]>,
        start: u64,
        write: bool,
        page_size: u64,
    ) {
        let page_size_usize = page_size as usize;
        let start_page = (start / page_size) as u32;

        // If data is exactly page-aligned and page-sized, we can use it directly
        if data.len() == page_size_usize && start % page_size == 0 {
            self.memory.insert(start_page, (data, write));
        } else {
            // Otherwise, chunk and convert to owned for each page
            for (i, chunk) in data.chunks(page_size_usize).enumerate() {
                let page_data = if chunk.len() == page_size_usize && data.len() == page_size_usize {
                    // Single full page, can reuse the original Cow
                    data.clone()
                } else {
                    // Multiple pages or partial page, needs to be owned
                    Cow::Owned(chunk.to_vec())
                };
                self.memory
                    .insert(start_page + i as u32, (page_data, write));
            }
        }
    }

    /// Insert pages from an owned vector.
    pub fn insert_pages_owned(&mut self, data: Vec<u8>, start: u64, write: bool, page_size: u64) {
        let page_size_usize = page_size as usize;
        let start_page = (start / page_size) as u32;

        for (i, chunk) in data.chunks(page_size_usize).enumerate() {
            self.memory
                .insert(start_page + i as u32, (Cow::Owned(chunk.to_vec()), write));
        }
    }
}
