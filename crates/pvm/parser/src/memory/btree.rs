//! The memory of a program.

use crate::{memory::MemoryInfo, StandardProgramBlob};
use std::{collections::BTreeMap, ops::Range};

/// (µ) The memory of a program.
#[derive(Default, Clone, Debug)]
pub struct Memory {
    /// The memory (µ).
    pub memory: BTreeMap<u32, (Vec<u8>, bool)>,

    /// The heap pointer.
    pub heap_ptr: u32,

    /// The memory information.
    pub info: MemoryInfo,
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
        memory.insert_pages(blob.ro_data.to_vec(), ptr, false);
        memory.info.read.start = ptr as u32;

        // RO padding: Z_Z + |o| ≤ i < Z_Z + P(|o|)
        let ro_padding_len = funp(ro_len) - ro_len;
        ptr += ro_len;
        memory.insert_pages(vec![0; ro_padding_len as usize], ptr, false);
        memory.info.read.end = ptr as u32;

        // RW data: 2*Z_Z + Z(|o|) ≤ i < 2*Z_Z + Z(|o|) + |w|
        ptr = 2 * crate::ZONE_SIZE + funz(ro_len);
        memory.insert_pages(blob.rw_data.to_vec(), ptr, true);
        memory.info.write.start = ptr as u32;

        // RW padding: 2*Z_Z + Z(|o|) + |w| ≤ i < 2*Z_Z + Z(|o|) + P(|w|) + Z_Z_P
        ptr += rw_len;
        let rw_padding_len =
            funp(rw_len) + crate::PAGE_SIZE * (blob.rw_data_padding_pages as u64) - rw_len;
        memory.insert_pages(vec![0; rw_padding_len as usize], ptr, true);
        memory.info.write.end = (ptr + rw_padding_len) as u32;

        // between write and stack, it's heap
        memory.info.heap.start = memory.info.write.end;
        memory.heap_ptr = memory.info.heap.start;

        // Stack: 2^32 - 2*Z_Z - Z_I - P(s) ≤ i < 2^32 - 2*Z_Z - Z_I
        let stack_padded_len = funp(blob.stack_size as u64);
        ptr = crate::PVM_MEMORY_SIZE
            - 2 * crate::ZONE_SIZE
            - crate::PVM_INIT_DATA_SIZE
            - stack_padded_len;
        memory.insert_pages(vec![0; stack_padded_len as usize], ptr, true);
        memory.info.heap.end = ptr as u32;
        memory.info.stack = (ptr as u32)..(ptr as u32 + stack_padded_len as u32);

        // Args: 2^32 - Z_Z - Z_I ≤ i < 2^32 - Z_Z - Z_I + |a|
        ptr = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE;
        memory.insert_pages(args.to_vec(), ptr, false);
        memory.info.args.start = ptr as u32;

        // Args padding: 2^32 - Z_Z - Z_I + |a| ≤ i < 2^32 - Z_Z - Z_I + P(|a|)
        ptr += args_len;
        let args_padding_len = funp(args_len) - args_len;
        memory.insert_pages(vec![0; args_padding_len as usize], ptr, false);
        memory.info.args.end = (ptr + args_padding_len) as u32;
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
            if size + taken == crate::PAGE_SIZE as usize {
                page += 1;
            }
        }
    }

    /// Read bytes from memory at given address
    pub fn read_bytes(&self, addr: u32, len: u32) -> anyhow::Result<Vec<u8>> {
        if len == 0 {
            return Ok(Vec::new());
        }

        let mut result = Vec::new();
        let mut ptr = addr;
        let mut remaining = len;
        while remaining > 0 {
            let page_num = ptr / crate::PAGE_SIZE as u32;
            let offset = ptr % crate::PAGE_SIZE as u32;
            let Some((page_data, _)) = self.memory.get(&page_num) else {
                anyhow::bail!("Memory page {} not accessible", page_num);
            };

            // Calculate how much to read from this page
            let length = crate::PAGE_SIZE as u32 - offset;
            let to_read = remaining.min(length).min(page_data.len() as u32 - offset);
            if to_read == 0 || offset as usize >= page_data.len() {
                let zero_bytes = remaining.min(length);
                result.extend(vec![0u8; zero_bytes as usize]);
                remaining -= zero_bytes;
                ptr += zero_bytes;
            } else {
                let end = (offset + to_read).min(page_data.len() as u32) as usize;
                result.extend_from_slice(&page_data[offset as usize..end]);
                remaining -= to_read;
                ptr += to_read;
            }
        }

        Ok(result)
    }

    /// Write bytes to memory at given address
    pub fn write_bytes(&mut self, addr: u32, bytes: &[u8]) -> anyhow::Result<()> {
        if bytes.is_empty() {
            return Ok(());
        }

        // validate all pages are accessible and writable
        let mut ptr = addr;
        let mut remaining = bytes.len();
        while remaining > 0 {
            let page_num = ptr / crate::PAGE_SIZE as u32;
            let page_offset = ptr % crate::PAGE_SIZE as u32;
            let Some((_, writable)) = self.memory.get(&page_num) else {
                anyhow::bail!("Memory page {} not accessible", page_num);
            };

            if !writable {
                anyhow::bail!("Attempting to write to read-only memory page {}", page_num);
            }

            let available_in_page = crate::PAGE_SIZE as u32 - page_offset;
            let chunk_size = remaining.min(available_in_page as usize);
            remaining -= chunk_size;
            ptr += chunk_size as u32;
        }

        // write the bytes
        ptr = addr;
        let mut bytes_written = 0;
        while bytes_written < bytes.len() {
            let page_num = ptr / crate::PAGE_SIZE as u32;
            let page_offset = ptr % crate::PAGE_SIZE as u32;
            let available_in_page = crate::PAGE_SIZE as u32 - page_offset;
            let remaining_bytes = bytes.len() - bytes_written;
            let chunk_size = remaining_bytes.min(available_in_page as usize);

            // Get mutable reference to the page data directly
            let (page_data, _) = self.memory.get_mut(&page_num).unwrap();
            let needed_size = page_offset as usize + chunk_size;
            if page_data.len() < needed_size {
                page_data.resize(needed_size.min(crate::PAGE_SIZE as usize), 0);
            }

            // Write the chunk directly
            let page_start = page_offset as usize;
            let page_end = page_start + chunk_size;
            let data_start = bytes_written;
            let data_end = data_start + chunk_size;
            page_data[page_start..page_end].copy_from_slice(&bytes[data_start..data_end]);
            bytes_written += chunk_size;
            ptr += chunk_size as u32;
        }

        Ok(())
    }

    /// Read a 32-byte hash from memory at given address
    pub fn read_hash(&self, addr: u32) -> anyhow::Result<[u8; 32]> {
        let bytes = self.read_bytes(addr, 32)?;
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&bytes);
        Ok(hash)
    }

    /// Allocate pages for sbrk implementation
    pub fn allocate(&mut self, start: u32, count: u32) -> anyhow::Result<()> {
        for page_num in start..start + count {
            let page_data = vec![0u8; crate::PAGE_SIZE as usize];
            self.memory.insert(page_num, (page_data, true));
        }
        Ok(())
    }

    pub fn args(&self) -> anyhow::Result<Vec<u8>> {
        self.read_bytes(
            self.info.args.start,
            self.info.args.end - self.info.args.start,
        )
    }

    /// Read the RO data from memory
    pub fn ro_data(&self) -> anyhow::Result<Vec<u8>> {
        self.read_bytes(
            self.info.read.start,
            self.info.read.end - self.info.read.start,
        )
    }

    /// Read the RW data from memory
    pub fn rw_data(&self) -> anyhow::Result<Vec<u8>> {
        self.read_bytes(
            self.info.write.start,
            self.info.write.end - self.info.write.start,
        )
    }

    /// Insert RO data into memory
    pub fn with_ro_data(mut self, data: Vec<u8>, start: u32) -> Self {
        let size = data.len() as u64;
        self.insert_pages(data, start as u64, false);
        self.info.read = start..(start + size as u32);
        self
    }

    /// Insert RW data into memory
    pub fn with_rw_data(mut self, data: Vec<u8>, start: u32) -> Self {
        let size = data.len() as u64;
        self.insert_pages(data, start as u64, true);
        self.info.write = start..(start + size as u32);
        self
    }

    /// Insert args into memory
    pub fn with_args(mut self, data: Vec<u8>, start: u32) -> Self {
        let size = data.len() as u64;
        self.insert_pages(data, start as u64, false);
        self.info.args = start..(start + size as u32);
        self
    }

    /// Set the heap range
    pub fn with_heap(mut self, range: Range<u32>) -> Self {
        self.heap_ptr = range.start;
        self.info.heap = range;
        self
    }

    /// Set the args range
    pub fn with_stack(mut self, range: Range<u32>) -> Self {
        self.insert_pages(vec![0; range.len()], range.start as u64, true);
        self.info.stack = range;
        self
    }
}


