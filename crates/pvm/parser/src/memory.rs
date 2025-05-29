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
        let (ro_len, rw_len, args_len) = (
            blob.ro_data.len() as u64,
            blob.rw_data.len() as u64,
            args.len() as u64,
        );

        // insert o pages
        let mut start = crate::ZONE_SIZE;
        memory.insert_pages_cow(blob.ro_data.clone(), start, false, crate::PAGE_SIZE);

        // insert pages from Z_Z + |o| to Z_Z + P(|o|)
        let len = funp(ro_len) as usize - ro_len as usize;
        start += ro_len;
        memory.insert_pages_owned(vec![0; len], start, true, crate::PAGE_SIZE);

        // insert pages between 2Z_Z + Z(|o|) and 2Z_Z + Z(|o|) + Z(|w|)
        start += crate::ZONE_SIZE;
        memory.insert_pages_cow(blob.rw_data.clone(), start, true, crate::PAGE_SIZE);

        // insert pages between 2Z_Z + Z(|o|) + Z(|w|) and 2Z_Z + Z(|o|) + P(|w|) + z * Z_P
        let len = (funp(rw_len) + blob.rw_data_padding_pages as u64 * crate::PAGE_SIZE) as usize
            - rw_len as usize;
        start += rw_len;
        memory.insert_pages_owned(vec![0; len], start, true, crate::PAGE_SIZE);

        // insert pages between 2^32 - Z-Z - Z_I - P(s) and 2^32 - 2Z_Z - Z_I
        let len = funp(blob.stack_size as u64) as usize - blob.stack_size as usize;
        start = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE - len as u64;
        memory.insert_pages_owned(vec![0; len], start, true, crate::PAGE_SIZE);

        // insert pages between 2^32 - Z-Z - Z_I and 2^32 - Z_Z - Z_I + |a|
        start += len as u64;
        memory.insert_pages_cow(Cow::Borrowed(args), start, false, crate::PAGE_SIZE);

        // insert pages between 2^32 - Z_Z - Z_I + |a| and 2^32 - Z_Z - Z_I + P(|a|)
        start += args_len;
        let len = funp(args_len) as usize - args_len as usize;
        memory.insert_pages_owned(vec![0; len], start, false, crate::PAGE_SIZE);
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
