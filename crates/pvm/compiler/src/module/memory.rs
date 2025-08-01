//! Memory implementation for compiled functions

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Page size constant (4KB)
pub const PAGE_SIZE: u32 = 4096;

/// Memory page with access control
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Page {
    /// Page data (4KB) - stored as Vec<u8> for serde compatibility  
    pub data: Vec<u8>,
    /// Access permissions: 0=Mutable, 1=Immutable, 2=Inaccessible
    pub access: u8,
}

impl Page {
    /// Create a new page with given access
    pub fn new(access: u8) -> Self {
        Self {
            data: vec![0; PAGE_SIZE as usize],
            access,
        }
    }

    /// Create page from interpreter page
    pub fn from_interpreter(interp_page: &pvmi::Page) -> Self {
        Self {
            data: interp_page.data.to_vec(),
            access: match interp_page.access {
                pvmi::Access::Mutable => 0,
                pvmi::Access::Immutable => 1,
                pvmi::Access::Inaccessible => 2,
            },
        }
    }

    /// Get page data as slice
    pub fn data_slice(&self) -> &[u8] {
        &self.data
    }

    /// Get mutable page data as slice
    pub fn data_slice_mut(&mut self) -> &mut [u8] {
        &mut self.data
    }
}

/// Simplified memory representation for compiled functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// Memory pages
    pub pages: BTreeMap<u32, Page>,
}

impl Memory {
    /// Create empty memory
    pub fn new() -> Self {
        Self {
            pages: BTreeMap::new(),
        }
    }

    /// Initialize memory from interpreter memory format
    pub fn from_interpreter(pages: &BTreeMap<u32, pvmi::Page>) -> Self {
        let mut memory_pages = BTreeMap::new();

        for (&page_num, interp_page) in pages {
            memory_pages.insert(page_num, Page::from_interpreter(interp_page));
        }

        Self {
            pages: memory_pages,
        }
    }

    /// Read bytes from memory
    pub fn read_bytes(&self, address: u32, len: u32) -> Result<Vec<u8>> {
        let mut bytes = vec![0; len as usize];
        let mut read = 0u32;

        while read < len {
            let page_num = (address + read) / PAGE_SIZE;
            let page_offset = (address + read) % PAGE_SIZE;
            let to_read = (len - read).min(PAGE_SIZE - page_offset);

            if let Some(page) = self.pages.get(&page_num) {
                let start = page_offset as usize;
                let end = (page_offset + to_read) as usize;
                bytes[read as usize..(read + to_read) as usize]
                    .copy_from_slice(&page.data[start..end]);
            }

            read += to_read;
        }

        Ok(bytes)
    }

    /// Write bytes to memory
    pub fn write_bytes(&mut self, address: u32, bytes: &[u8]) -> Result<()> {
        let len = bytes.len() as u32;
        let mut written = 0u32;

        while written < len {
            let page_num = (address + written) / PAGE_SIZE;
            let page_offset = (address + written) % PAGE_SIZE;
            let to_write = (len - written).min(PAGE_SIZE - page_offset);

            let page = self.pages.entry(page_num).or_insert_with(|| Page::new(0));

            if page.access != 0 {
                anyhow::bail!("Page {} is not writable", page_num);
            }

            let start = page_offset as usize;
            let end = (page_offset + to_write) as usize;
            page.data[start..end]
                .copy_from_slice(&bytes[written as usize..(written + to_write) as usize]);

            written += to_write;
        }

        Ok(())
    }

}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
