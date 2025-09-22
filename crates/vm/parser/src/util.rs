//! Utility functions.

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
