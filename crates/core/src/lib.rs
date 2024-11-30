use scale::{Decode, Encode};

#[derive(Debug, Encode, Decode)]
pub struct AvailAssurance {
    anchor: [u8; 32],
    bitfield: [u8; 1],
    validator_index: u16,
    signature: [u8; 64],
}
