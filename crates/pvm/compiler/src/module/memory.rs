//! Memory implementation for compiled functions

use anyhow::Result;
use std::collections::BTreeMap;
use translator::constants::{access, PAGE_SIZE};

/// Memory page with access control
#[repr(C)]
#[derive(Debug, Clone)]
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
}

/// Memory representation for compiled functions
#[derive(Debug, Clone)]
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
                if page.access != access::MUTABLE {
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

    /// Read bytes from memory
    pub fn read_bytes(&self, address: u32, len: u32) -> Result<Vec<u8>> {
        let mut result = vec![0u8; len as usize];
        let mut read_offset = 0u32;

        while read_offset < len {
            let page_num = (address + read_offset) / PAGE_SIZE;
            let page_offset = (address + read_offset) % PAGE_SIZE;
            let to_read = (len - read_offset).min(PAGE_SIZE - page_offset);

            if let Some(page) = self.pages.get(&page_num) {
                // Check if page is accessible for reading
                if page.access == access::INACCESSIBLE {
                    // Inaccessible page - return zeros for these bytes
                    // This matches PVM behavior for inaccessible memory
                } else {
                    let start = page_offset as usize;
                    let end = (page_offset + to_read) as usize;
                    result[read_offset as usize..(read_offset + to_read) as usize]
                        .copy_from_slice(&page.data[start..end]);
                }
            }
            // If page doesn't exist, bytes remain as zeros

            read_offset += to_read;
        }

        Ok(result)
    }

    /// Read a single byte from memory
    pub fn read_u8(&self, address: u32) -> Result<u8> {
        let bytes = self.read_bytes(address, 1)?;
        Ok(bytes[0])
    }

    /// Read a 16-bit value from memory (little endian)
    pub fn read_u16(&self, address: u32) -> Result<u16> {
        let bytes = self.read_bytes(address, 2)?;
        Ok(u16::from_le_bytes([bytes[0], bytes[1]]))
    }

    /// Read a 32-bit value from memory (little endian)
    pub fn read_u32(&self, address: u32) -> Result<u32> {
        let bytes = self.read_bytes(address, 4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    /// Read a 64-bit value from memory (little endian)
    pub fn read_u64(&self, address: u32) -> Result<u64> {
        let bytes = self.read_bytes(address, 8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }
}

impl Default for Memory {
    fn default() -> Self {
        Self::new()
    }
}
