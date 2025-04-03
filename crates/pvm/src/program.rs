//! Program related functions.

use codec::compact::Numeric;

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
pub fn deblob(blob: &[u8]) -> Result<(Vec<u8>, Vec<u8>, Vec<u64>), &str> {
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
        return Err("empty program blob");
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

    Ok((instructions, bitmask, jump))
}

/// The `skip` function
///
/// provides the number of octets, minus one, to the next instruction’s opcode,
/// given the index of instruction’s opcode index into c (and by extension k):
///
/// ```text
/// i ↦ min(24, j∈ N ∶(k⌢ [1,1,...])i+1+j = 1)
/// ```
pub fn skip(pc: usize, bitmask: &[u8]) -> usize {
    let byte = pc / 8;
    let mut distance = 1;
    let mut bit = pc % 8;

    // search for the next instruction
    for byte in bitmask[byte..].iter() {
        for bit_idx in bit..8 {
            if (byte >> bit_idx) & 1 == 1 {
                return distance;
            }
        }
        distance += 1;
        bit = 0;
    }

    distance.min(24)
}
