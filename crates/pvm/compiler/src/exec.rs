//! Executable instance for the compiled object

use anyhow::{Context, Result};
use cranelift::object::object::{
    self, Architecture, Object, ObjectSection, ObjectSymbol, RelocationKind,
};
use pvm::{Argument, PAGE_SIZE};
use std::{collections::HashMap, ptr};

/// The executable instance for the compiled object
#[derive(Debug, Clone, Default)]
pub struct Executable {
    /// The symbol table
    symbols: HashMap<String, usize>,

    /// The memory
    memory: *mut u8,

    /// The size of the memory
    size: usize,
}

impl Executable {
    /// Get the address of the symbol
    pub fn get(&self, symbol: &str) -> Result<usize> {
        let address = self
            .symbols
            .get(symbol)
            .cloned()
            .with_context(|| format!("Unresolved symbol: {}", symbol))?;

        Ok(self.memory as usize + address)
    }

    /// Load the executable into memory
    pub fn load<X: Argument>(&mut self, object: &[u8]) -> Result<()> {
        let elf = object::File::parse(object)?;
        if elf.architecture() != Architecture::X86_64 {
            return Err(anyhow::anyhow!("Unsupported architecture"));
        }

        // allocate pre-sized memory and load the object
        self.allocate_pre(&elf)?;
        self.load_symbols(&elf);
        self.load_sections(&elf)?;

        // make memory executable
        unsafe {
            if libc::mprotect(
                self.memory as *mut _,
                self.size,
                libc::PROT_READ | libc::PROT_EXEC,
            ) != 0
            {
                libc::munmap(self.memory as *mut _, self.size);
                return Err(anyhow::anyhow!("Failed to set memory permissions"));
            }
        }
        Ok(())
    }

    /// Check if the executable is loaded
    pub fn loaded(&self) -> bool {
        !self.memory.is_null()
    }

    /// Allocate pre-sized memory for the executable
    fn allocate_pre<'d>(&mut self, obj: &object::File<'d>) -> Result<()> {
        let mut total_size = 0u64;
        for section in obj.sections() {
            if section.size() > 0 {
                total_size = total_size.max(section.address() + section.size());
            }
        }

        // Round up to page size
        self.size = (total_size.div_ceil(PAGE_SIZE) * PAGE_SIZE) as usize;
        self.memory = unsafe {
            libc::mmap(
                ptr::null_mut(),
                self.size,
                libc::PROT_READ | libc::PROT_WRITE | libc::PROT_EXEC,
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS,
                -1,
                0,
            ) as *mut u8
        };

        if self.memory.is_null() {
            return Err(anyhow::anyhow!("Failed to allocate memory"));
        }

        Ok(())
    }

    /// Load sections into the executable
    fn load_sections<'d>(&mut self, obj: &object::File<'d>) -> Result<()> {
        for section in obj.sections() {
            if section.size() == 0 {
                continue;
            }

            let data = section.data()?;
            let offset = section.address() as usize;
            let Ok(name) = section.name() else {
                continue;
            };

            let should_load = match name {
                ".text" | ".data" | ".rodata" | ".bss" => true,
                ".symtab" | ".strtab" | ".shstrtab" => false,
                _ if name.starts_with(".debug") => false,
                _ if name.starts_with(".note") => false,
                _ => true,
            };

            if should_load && offset + data.len() <= self.size {
                unsafe {
                    ptr::copy_nonoverlapping(data.as_ptr(), self.memory.add(offset), data.len());
                }
            }

            // process relocations
            for (offset, reloc) in section.relocations() {
                let sym_name = match reloc.target() {
                    object::RelocationTarget::Symbol(sym_idx) => {
                        let symbol = obj.symbol_by_index(sym_idx)?;
                        symbol.name()?.to_string()
                    }
                    _ => continue,
                };

                let sym = *self
                    .symbols
                    .get(&sym_name)
                    .with_context(|| format!("Unresolved symbol: {}", sym_name))?;

                let place = section.address() as usize + offset as usize;
                let patch_addr = self.memory as usize + place;
                let value = match reloc.kind() {
                    RelocationKind::Relative => (sym as i64)
                        .wrapping_add(reloc.addend())
                        .wrapping_sub(patch_addr as i64),
                    RelocationKind::Absolute => (sym as i64).wrapping_add(reloc.addend()),
                    _ => return Err(anyhow::anyhow!("Unsupported relocation kind")),
                };

                // write the relocation (64-bit only)
                unsafe {
                    if reloc.size() != 64 {
                        return Err(anyhow::anyhow!("Only 64-bit relocations are supported"));
                    }
                    let ptr = self.memory.add(place) as *mut i64;
                    *ptr = value;
                }
            }
        }

        Ok(())
    }

    /// Load the symbols into the executable
    fn load_symbols<'d>(&mut self, obj: &object::File<'d>) {
        for symbol in obj.symbols() {
            if let Ok(name) = symbol.name() {
                let section_offset = symbol.address() as usize;
                let final_address = if let object::SymbolSection::Section(idx) = symbol.section() {
                    obj.section_by_index(idx)
                        .ok()
                        .map(|s| s.address() as usize + section_offset)
                        .unwrap_or(section_offset)
                } else {
                    section_offset
                };

                self.symbols.insert(name.to_string(), final_address);
            }
        }
    }
}

impl Drop for Executable {
    fn drop(&mut self) {
        unsafe {
            libc::munmap(self.memory as *mut _, self.size);
        }
    }
}
