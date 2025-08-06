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
}

/// Memory representation for compiled functions
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

    /// Write bytes to memory
    pub fn write_bytes(&mut self, address: u32, bytes: &[u8]) -> Result<()> {
        let len = bytes.len() as u32;

        // First pass: validate all pages that will be accessed
        let mut check_offset = 0u32;
        while check_offset < len {
            let page_num = (address + check_offset) / PAGE_SIZE;
            let page_offset = (address + check_offset) % PAGE_SIZE;
            let to_check = (len - check_offset).min(PAGE_SIZE - page_offset);

            // Check if page exists and is writable
            if let Some(page) = self.pages.get(&page_num) {
                if page.access != 0 {
                    anyhow::bail!("Page {} is not writable", page_num);
                }
            } else {
                // Page doesn't exist - this should cause a page fault
                anyhow::bail!("Page {} is not allocated", page_num);
            }

            check_offset += to_check;
        }

        // Second pass: perform the actual write (all pages are now validated)
        let mut written = 0u32;
        while written < len {
            let page_num = (address + written) / PAGE_SIZE;
            let page_offset = (address + written) % PAGE_SIZE;
            let to_write = (len - written).min(PAGE_SIZE - page_offset);

            let page = self.pages.get_mut(&page_num).unwrap(); // Safe due to validation above

            let start = page_offset as usize;
            let end = (page_offset + to_write) as usize;
            page.data[start..end]
                .copy_from_slice(&bytes[written as usize..(written + to_write) as usize]);

            written += to_write;
        }

        Ok(())
    }

    /// Write a single byte to memory
    pub fn write_u8(&mut self, address: u32, value: u8) -> Result<()> {
        self.write_bytes(address, &[value])
    }

    /// Write a 16-bit value to memory (little endian)
    pub fn write_u16(&mut self, address: u32, value: u16) -> Result<()> {
        let bytes = value.to_le_bytes();
        self.write_bytes(address, &bytes)
    }

    /// Write a 32-bit value to memory (little endian)
    pub fn write_u32(&mut self, address: u32, value: u32) -> Result<()> {
        let bytes = value.to_le_bytes();
        self.write_bytes(address, &bytes)
    }

    /// Write a 64-bit value to memory (little endian)
    pub fn write_u64(&mut self, address: u32, value: u64) -> Result<()> {
        let bytes = value.to_le_bytes();
        self.write_bytes(address, &bytes)
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
