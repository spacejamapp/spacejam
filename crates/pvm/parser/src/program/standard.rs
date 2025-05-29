//! Standard program blob.

use crate::program::PreimageBlob;
use codec::compact::Numeric;
use std::collections::BTreeMap;

/// The standard program blob.
#[derive(Default)]
pub struct StandardProgramBlob {
    /// The program code (c).
    pub code: Vec<u8>,

    /// The registers (ω).
    pub registers: [u64; 13],

    /// The memory (µ).
    pub memory: BTreeMap<u32, (Vec<u8>, bool)>,
}

impl From<PreimageBlob<'_>> for StandardProgramBlob {
    fn from(blob: PreimageBlob<'_>) -> Self {
        tracing::trace!("converting PreimageBlob to StandardProgramBlob");
        tracing::trace!("ro data length: {}", blob.ro_data.len());
        tracing::trace!("rw data length: {}", blob.rw_data.len());
        tracing::trace!("code length: {}", blob.code_blob.len());
        tracing::trace!("rw data padding pages: {:?}", blob.rw_data_padding_pages);
        tracing::trace!("stack size: {}", blob.stack_size);

        // Extract data from the workaround format
        let ro_data = blob.ro_data.to_vec();
        let rw_data = blob.rw_data.to_vec();
        let code = blob.code_blob.to_vec();
        let z = blob.rw_data_padding_pages as u64; // padding pages
        let s = blob.stack_size as u64; // stack size

        // Encode in the standard format: E₃(|o|) ⌢ E₃(|w|) ⌢ E₂(z) ⌢ E₃(s) ⌢ o ⌢ w ⌢ E₄(|c|) ⌢ c
        let mut standard_blob = Vec::new();

        // E₃(|o|) - encode ro_data length as 3 bytes (little-endian)
        let ro_len = ro_data.len() as u64;
        standard_blob.extend_from_slice(&ro_len.to_le_bytes()[..3]);

        // E₃(|w|) - encode rw_data length as 3 bytes (little-endian)
        let rw_len = rw_data.len() as u64;
        standard_blob.extend_from_slice(&rw_len.to_le_bytes()[..3]);

        // E₂(z) - encode padding pages as 2 bytes (little-endian)
        standard_blob.extend_from_slice(&z.to_le_bytes()[..2]);

        // E₃(s) - encode stack size as 3 bytes (little-endian)
        standard_blob.extend_from_slice(&s.to_le_bytes()[..3]);

        // o - ro_data
        standard_blob.extend_from_slice(&ro_data);

        // w - rw_data
        standard_blob.extend_from_slice(&rw_data);

        // E₄(|c|) - encode code length as 4 bytes (little-endian)
        let code_len = code.len() as u64;
        standard_blob.extend_from_slice(&code_len.to_le_bytes()[..4]);

        // c - code
        standard_blob.extend_from_slice(&code);

        // Parse using the standard function (no args for this format)
        crate::program::standard(&standard_blob, &[]).unwrap_or_else(|e| {
            tracing::error!("Failed to parse standard format: {}", e);
            // Fallback to empty standard program blob
            StandardProgramBlob::default()
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
    let code_len_start = wend; // Code length starts right after w
    let code_len = u64::decode(&format[code_len_start..code_len_start + 4]) as usize;
    let code_start = code_len_start + 4; // Code data starts after the 4-byte length
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
