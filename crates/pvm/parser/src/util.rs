//! Utility functions.

use crate::{ProgramBlob, StandardProgramBlob};
use anyhow::Result;
use codec::compact::Numeric;
use std::collections::BTreeMap;

/// The `skip` function
///
/// provides the number of octets, minus one, to the next instruction's opcode,
/// given the index of instruction's opcode index into c (and by extension k):
///
/// ```text
/// i ↦ min(24, j∈ N ∶(k⌢ [1,1,...])i+1+j = 1)
/// ```
pub fn skip(pc: usize, bitmask: &[u8]) -> usize {
    let byte = pc / 8;
    let mut distance = 0;
    let mut bit = pc % 8;

    // search for the next instruction
    for byte in bitmask[byte..].iter() {
        for bit_idx in bit..8 {
            if (byte >> bit_idx) & 1 == 1 {
                return distance;
            }

            distance += 1;
        }
        bit = 0;
    }

    distance.min(24)
}

/// The `deblob` function.
///
/// The program blob `p` is split into as series of octets which make
/// up the instruction data `c` and the opcode bitmask `k` as well as
/// the jump table `j`.
///
/// The latter, dynamic jump table, is a sequence of indices into the
/// instruction data blob and is indexed into when dynamically-computed
/// jumps are taken. It is encoded as a sequence of natural numbers
/// (i.e. non-negative integers) each encoded with the same length in
/// octets. This length, term z above, is itself encoded prior.
///
/// `p` = E(∣j∣)⌢ E1(z)⌢ E(∣c∣)⌢ Ez(j)⌢ E(c)⌢ E(k), ∣k∣= ∣c∣
#[allow(clippy::type_complexity)]
pub fn deblob(blob: &[u8]) -> Result<ProgramBlob> {
    let mut pos = 0;

    // decode the jump table length
    //
    // E(|j|)
    let (len, next) = codec::compact::decode_from(blob);
    let jump_table_len = len as usize;
    pos += next;

    // decode the jump table entry size
    //
    // E₁(z)
    let jump_table_entry_size = blob[pos] as usize;
    pos += 1;

    // decode the instruction data length
    //
    // E(|c|)
    let (len, next) = codec::compact::decode_from(&blob[pos..]);
    let instruction_len = len as usize;
    pos += next;

    // decode the jump table
    //
    // E_z(j)
    let jump = if jump_table_entry_size > 0 {
        let length = jump_table_len * jump_table_entry_size;
        let table = blob[pos..pos + length].to_vec();
        let jump = table
            .chunks(jump_table_entry_size)
            .map(u64::decode)
            .collect();

        pos += length;
        jump
    } else {
        vec![]
    };

    // decode the instruction data
    //
    // E(c)
    let instructions = blob[pos..pos + instruction_len].to_vec();
    pos += instruction_len;

    // check that the program blob is not empty
    if instructions.is_empty() {
        anyhow::bail!("empty program blob");
    }

    // decode the bitmask
    //
    // E(k)
    let bitmask = blob[pos..].to_vec();
    // TODO: bitmask length check
    //
    // if bitmask.len() * 8 != instructions.len() {
    //     return Err("bitmask length does not match instruction length");
    // }

    Ok(ProgramBlob {
        instructions,
        bitmask,
        jump_table: jump,
    })
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
/// let E3(∣o∣)⌢ E3(∣w∣)⌢ E2(z)⌢ E3(s)⌢ o⌢ w⌢ E4(∣c∣)⌢ c = p
pub fn standard(format: &[u8], _args: &[u8]) -> anyhow::Result<StandardProgramBlob> {
    let len = format.len();
    if len < 15 {
        anyhow::bail!("Invalid format length")
    }

    // decode the length of o, w, and c, s
    let olen = u64::decode(&format[..3]) as usize;
    let wlen = u64::decode(&format[3..6]) as usize;
    let z = u64::decode(&format[6..8]);
    let s = u64::decode(&format[8..11]);

    // pre-calculate offset of o and w
    let oend = 11 + olen;
    let wend = oend + wlen;
    if oend > len || wend > len {
        anyhow::bail!("Failed to decode memory, invalid format length")
    }

    // extract o, w, c
    let o = format[11..oend].to_vec();
    let w = format[oend..wend].to_vec();

    // decode code
    let code_start = oend + wlen + 4;
    let code_len = u64::decode(&format[code_start..code_start + 4]) as usize;
    if code_start + code_len > len {
        anyhow::bail!("Failed to decode program blob, invalid format length")
    }

    let code = format[code_start..code_start + code_len].to_vec();

    // decode a
    let a = format[code_start + code_len..].to_vec();
    let alen = a.len();

    // with o, w, decode the memory and registers
    let funp = |x: u64| x.div_ceil(crate::PAGE_SIZE) * crate::PAGE_SIZE;
    let funz = |x: u64| x.div_ceil(crate::ZONE_SIZE) * crate::ZONE_SIZE;
    if (5 * crate::ZONE_SIZE
        + funz(olen as u64)
        + funz(wlen as u64 + z * crate::ZONE_SIZE)
        + funz(s)
        + crate::PVM_INIT_DATA_SIZE)
        > crate::PVM_MEMORY_SIZE
    {
        anyhow::bail!("Failed to decode memory, invalid format length")
    }

    // decode the registers
    let mut registers = [0u64; 13];
    registers[0] = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE;
    registers[1] = crate::PVM_MEMORY_SIZE - 2 * crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE;
    registers[7] = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE;
    registers[8] = alen as u64;

    // decode the memory
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
    insert_page(o, start, false);

    // insert pages from Z_Z + |o| to Z_Z + P(|o|)
    let len = funp(olen as u64) as usize - olen;
    start += olen as u64;
    insert_page(vec![0; len], start, true);

    // insert pages between 2Z_Z + Z(|o|) and 2Z_Z + Z(|o|) + Z(|w|)
    start += crate::ZONE_SIZE;
    insert_page(w, start, true);

    // insert pages between 2Z_Z + Z(|o|) + Z(|w|) and 2Z_Z + Z(|o|) + P(|w|) + z * Z_P
    let len = (funp(wlen as u64) + z * crate::PAGE_SIZE) as usize - wlen;
    start += wlen as u64;
    insert_page(vec![0; len], start, true);

    // insert pages between 2^32 - Z-Z - Z_I - P(s) and 2^32 - 2Z_Z - Z_I
    let len = funp(s);
    start = crate::PVM_MEMORY_SIZE - crate::ZONE_SIZE - crate::PVM_INIT_DATA_SIZE - len;
    insert_page(vec![0; len as usize], start, true);

    // insert pages between 2^32 - Z-Z - Z_I and 2^32 - Z_Z - Z_I + |a|
    start += len;
    insert_page(a, start, false);

    // insert pages between 2^32 - Z_Z - Z_I + |a| and 2^32 - Z_Z - Z_I + P(|a|)
    start += alen as u64;
    let len = funp(alen as u64) as usize - alen;
    insert_page(vec![0; len], start, false);

    // E(c)
    Ok(StandardProgramBlob {
        code,
        registers,
        memory,
    })
}
