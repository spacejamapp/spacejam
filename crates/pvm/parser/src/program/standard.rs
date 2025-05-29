//! Standard program blob.

use crate::program::Program;
use anyhow::Result;
use codec::{io, Reader};
use std::{borrow::Cow, collections::BTreeMap};

/// The standard program blob
pub struct StandardProgramBlob<'a> {
    /// (o) The read-only data
    pub ro_data: Cow<'a, [u8]>,
    /// (w) The read-write data
    pub rw_data: Cow<'a, [u8]>,
    /// (c) The blob of the code
    pub code_blob: Cow<'a, [u8]>,
    /// (z) Padding pages for read-write data
    pub rw_data_padding_pages: u16,
    /// (s) The size of stack
    pub stack_size: u32,
}

impl<'a> StandardProgramBlob<'a> {
    /// Initialize the program.
    ///
    /// decode the registers (ω) and the memory (µ)
    pub fn init(&self, args: &'a [u8]) -> Result<Program<'a>> {
        let funp = |x: u64| x.div_ceil(crate::PAGE_SIZE) * crate::PAGE_SIZE;
        let funz = |x: u64| x.div_ceil(crate::ZONE_SIZE) * crate::ZONE_SIZE;
        let (ro_len, rw_len, args_len) = (
            self.ro_data.len() as u64,
            self.rw_data.len() as u64,
            args.len() as u64,
        );

        // with o, w, decode the memory and registers
        if (5 * crate::ZONE_SIZE
            + funz(ro_len)
            + funz(rw_len + self.rw_data_padding_pages as u64 * crate::ZONE_SIZE)
            + funz(self.stack_size as u64)
            + crate::PVM_INIT_DATA_SIZE)
            > crate::PVM_MEMORY_SIZE
        {
            anyhow::bail!("Failed to decode memory, invalid format length")
        }

        // decode the registers (ω)
        let mut registers = [0u64; 13];
        registers[0] = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE;
        registers[1] = crate::PVM_MEMORY_SIZE - 2 * crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE;
        registers[7] = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE;
        registers[8] = args.len() as u64;

        // decode the memory (µ)
        let mut memory = BTreeMap::<u32, (Vec<u8>, bool)>::new();
        let mut insert_page = |data: Vec<u8>, start: u64, write: bool| {
            let pages = data
                .chunks(crate::PAGE_SIZE as usize)
                .map(|page| page.to_vec())
                .collect::<Vec<_>>();
            let pagenum = (start / crate::PAGE_SIZE) as u32;
            for (i, page) in pages.iter().enumerate() {
                memory.insert(pagenum + i as u32, (page.to_vec(), write));
            }
        };

        // insert o pages
        let mut start = crate::ZONE_SIZE;
        insert_page(self.ro_data.to_vec(), start, false);

        // insert pages from Z_Z + |o| to Z_Z + P(|o|)
        let len = funp(ro_len) as usize - ro_len as usize;
        start += ro_len;
        insert_page(vec![0; len], start, true);

        // insert pages between 2Z_Z + Z(|o|) and 2Z_Z + Z(|o|) + Z(|w|)
        start += crate::ZONE_SIZE;
        insert_page(self.rw_data.to_vec(), start, true);

        // insert pages between 2Z_Z + Z(|o|) + Z(|w|) and 2Z_Z + Z(|o|) + P(|w|) + z * Z_P
        let len = (funp(rw_len) + self.rw_data_padding_pages as u64 * crate::PAGE_SIZE) as usize
            - rw_len as usize;
        start += rw_len;
        insert_page(vec![0; len], start, true);

        // insert pages between 2^32 - Z-Z - Z_I - P(s) and 2^32 - 2Z_Z - Z_I
        let len = funp(self.stack_size as u64) as usize - self.stack_size as usize;
        start = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE - len as u64;
        insert_page(vec![0; len], start, true);

        // insert pages between 2^32 - Z-Z - Z_I and 2^32 - Z_Z - Z_I + |a|
        start += len as u64;
        insert_page(args.to_vec(), start, false);

        // insert pages between 2^32 - Z_Z - Z_I + |a| and 2^32 - Z_Z - Z_I + P(|a|)
        start += args_len;
        let len = funp(args_len) as usize - args_len as usize;
        insert_page(vec![0; len], start, false);

        Ok(Program {
            registers,
            memory,
            code: self.code_blob.clone(),
        })
    }
}

impl<'a> TryFrom<&'a [u8]> for StandardProgramBlob<'a> {
    type Error = anyhow::Error;

    fn try_from(mut blob: &'a [u8]) -> Result<Self, Self::Error> {
        if blob.len() < 15 {
            anyhow::bail!("Invalid format length")
        }

        // E₃(|o|) - decode the read-only data length
        let ro_data_len = blob
            .read_u24()
            .ok_or_else(|| anyhow::anyhow!("EOF while reading read-only data length"))?;

        // E₃(|w|) - decode the read-write data length
        let rw_data_len = blob
            .read_u24()
            .ok_or_else(|| anyhow::anyhow!("EOF while reading read-write data length"))?;

        // E₂(z) - decode the padding pages
        let rw_data_padding_pages = blob
            .read_u16()
            .ok_or_else(|| anyhow::anyhow!("EOF while reading padding pages"))?;

        // E₃(s) - decode the stack size
        let stack_size = blob
            .read_u24()
            .ok_or_else(|| anyhow::anyhow!("EOF while reading stack size"))?;

        // o - decode the read-only data
        let ro_data = io::read_cow(&mut blob, ro_data_len)
            .ok_or_else(|| anyhow::anyhow!("EOF while reading read-only data"))?;

        // w - decode the read-write data
        let rw_data = io::read_cow(&mut blob, rw_data_len)
            .ok_or_else(|| anyhow::anyhow!("EOF while reading read-write data"))?;

        // E₄(|c|) - decode the code length
        let code_blob_len = blob
            .read_u32()
            .ok_or_else(|| anyhow::anyhow!("EOF while reading code length"))?;

        // c - decode the code
        let code_blob = io::read_cow(&mut blob, code_blob_len)
            .ok_or_else(|| anyhow::anyhow!("EOF while reading code"))?;

        Ok(Self {
            rw_data_padding_pages,
            stack_size,
            ro_data,
            rw_data,
            code_blob,
        })
    }
}

/// The `Y` function.
///
/// We thus define the standard program code format p, which includes not only the
/// instructions and jump table (previ-ously represented by the term c), but also
/// information on the state of the ram at program start. Given some p which
/// is appropriately encoded together with some argument data a, we can define
/// program code c, registers ω and ram µ through the standard initialization
/// decoder function Y
///
/// * let E3(∣o∣)⌢ E3(∣w∣)⌢ E2(z)⌢ E3(s)⌢ o⌢ w⌢ E4(∣c∣)⌢ c = p
/// * (p, a) -> (c, ω, µ)
pub fn standard<'a>(format: &'a [u8], args: &'a [u8]) -> anyhow::Result<Program<'a>> {
    StandardProgramBlob::try_from(format)?.init(args)
}
