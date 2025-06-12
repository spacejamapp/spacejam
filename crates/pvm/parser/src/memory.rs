//! The memory of a program.

use crate::StandardProgramBlob;
use std::{collections::BTreeMap, ops::Range};

/// (µ) The memory of a program.
#[derive(Default)]
pub struct Memory {
    /// The memory (µ).
    pub memory: BTreeMap<u32, (Vec<u8>, bool)>,

    /// The heap range.
    pub heap: Range<u32>,

    /// The read-only range.
    pub read: Range<u32>,

    /// The stack range.
    pub stack: Range<u32>,

    /// The args range.
    pub args: Range<u32>,
}

impl Memory {
    /// Initialize the memory.
    pub fn init(blob: &StandardProgramBlob, args: &[u8]) -> Self {
        let mut memory = Self::default();
        let funp = |x: u64| x.div_ceil(crate::PAGE_SIZE) * crate::PAGE_SIZE;
        let funz = |x: u64| x.div_ceil(crate::ZONE_SIZE) * crate::ZONE_SIZE;
        let (ro_len, rw_len, args_len) = (
            blob.ro_data.len() as u64,
            blob.rw_data.len() as u64,
            args.len() as u64,
        );

        // RO data: Z_Z ≤ i < Z_Z + |o|
        let mut ptr = crate::ZONE_SIZE;
        // tracing::trace!(
        //     "initializing RO data, ptr={ptr} size={ro_len} pages={}..{}",
        //     ptr / crate::PAGE_SIZE,
        //     ro_len / crate::PAGE_SIZE + ptr / crate::PAGE_SIZE
        // );
        memory.insert_pages(blob.ro_data.to_vec(), ptr, false);
        memory.read.start = ptr as u32;

        // RO padding: Z_Z + |o| ≤ i < Z_Z + P(|o|)
        let ro_padding_len = funp(ro_len) - ro_len;
        ptr += ro_len;
        // tracing::trace!(
        //     "initializing RO padding, ptr={ptr} size={ro_padding_len} pages={}..{}",
        //     ptr / crate::PAGE_SIZE,
        //     ro_padding_len / crate::PAGE_SIZE + ptr / crate::PAGE_SIZE
        // );
        memory.insert_pages(vec![0; ro_padding_len as usize], ptr, false);
        memory.read.end = (ptr + ro_padding_len) as u32;

        // (heap) RW data: 2*Z_Z + Z(|o|) ≤ i < 2*Z_Z + Z(|o|) + |w|
        ptr = 2 * crate::ZONE_SIZE + funz(ro_len);
        /* tracing::trace!(
            "initializing RW data, ptr={ptr} size={rw_len} pages={}..{}",
            ptr / crate::PAGE_SIZE,
            rw_len / crate::PAGE_SIZE + ptr / crate::PAGE_SIZE
        ); */
        memory.insert_pages(blob.rw_data.to_vec(), ptr, true);
        memory.heap.start = ptr as u32;

        // (heap) RW padding: 2*Z_Z + Z(|o|) + |w| ≤ i < 2*Z_Z + Z(|o|) + P(|w|) + Z_Z_P
        ptr += rw_len;
        let rw_padding_len =
            funp(rw_len) + crate::PAGE_SIZE * (blob.rw_data_padding_pages as u64) - rw_len;
        /* tracing::trace!(
            "initializing RW padding, ptr={ptr} size={rw_padding_len} pages={}..{}",
            ptr / crate::PAGE_SIZE,
            rw_padding_len / crate::PAGE_SIZE + ptr / crate::PAGE_SIZE
        ); */
        memory.insert_pages(vec![0; rw_padding_len as usize], ptr, true);
        memory.heap.end = (ptr + rw_padding_len) as u32;

        // Stack: 2^32 - 2*Z_Z - Z_I - P(s) ≤ i < 2^32 - 2*Z_Z - Z_I
        let stack_padded_len = funp(blob.stack_size as u64);
        ptr = crate::PVM_MEMORY_SIZE
            - 2 * crate::ZONE_SIZE
            - crate::PVM_INIT_DATA_SIZE
            - stack_padded_len;
        // tracing::debug!(
        //     "initializing stack, ptr={ptr} size={stack_padded_len} pages={}..{}",
        //     ptr / crate::PAGE_SIZE,
        //     stack_padded_len / crate::PAGE_SIZE + ptr / crate::PAGE_SIZE
        // );
        memory.insert_pages(vec![0; stack_padded_len as usize], ptr, true);
        memory.stack = (ptr as u32)..(ptr as u32 + stack_padded_len as u32);

        // Args: 2^32 - Z_Z - Z_I ≤ i < 2^32 - Z_Z - Z_I + |a|
        ptr = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE;
        // tracing::trace!(
        //     "initializing args, ptr={:08x} size={args_len} pages={}..{}",
        //     ptr,
        //     ptr / crate::PAGE_SIZE,
        //     args_len / crate::PAGE_SIZE + ptr / crate::PAGE_SIZE
        // );
        memory.insert_pages(args.to_vec(), ptr, false);
        tracing::debug!("args: {:?}", args);
        memory.args.start = ptr as u32;

        // Args padding: 2^32 - Z_Z - Z_I + |a| ≤ i < 2^32 - Z_Z - Z_I + P(|a|)
        ptr += args_len;
        let args_padding_len = funp(args_len) - args_len;
        // tracing::trace!(
        //     "initializing args padding, ptr={ptr} size={args_padding_len} pages={}..{}",
        //     ptr / crate::PAGE_SIZE,
        //     args_padding_len / crate::PAGE_SIZE + ptr / crate::PAGE_SIZE
        // );
        memory.insert_pages(vec![0; args_padding_len as usize], ptr, false);
        memory.args.end = (ptr + args_padding_len) as u32;
        memory
    }

    /// Insert pages from a vector.
    pub fn insert_pages(&mut self, data: Vec<u8>, ptr: u64, write: bool) {
        let mut buff: Vec<u8> = Vec::new();
        let mut page = (ptr / crate::PAGE_SIZE) as u32;
        for chunk in data.chunks(crate::PAGE_SIZE as usize) {
            buff.extend_from_slice(chunk);
            let mut content = if let Some((content, _)) = self.memory.get(&page) {
                content.clone()
            } else {
                vec![]
            };

            // extend the page with the remaining data
            let size = content.len();
            let rest = crate::PAGE_SIZE as usize - size;
            let taken = rest.min(buff.len());
            content.extend_from_slice(&buff[..taken]);

            // insert the page into the memory
            self.memory.insert(page, (content, write));
            buff = buff[taken..].to_vec();

            // check if the page is full, move to the next page
            if size + taken == crate::PAGE_SIZE as usize {
                page += 1;
            }
        }
    }
}
