use core_derive::Json;
use scale::{Decode, Encode};

#[derive(Debug, Encode, Decode, Json)]
pub struct AvailAssurance {
    /// The anchor of the assurance
    pub anchor: [u8; 32],
    /// The bitfield of the assurance
    pub bitfield: [u8; 1],
    /// The validator index of the assurance
    pub validator_index: u16,
    /// The signature of the assurance
    pub signature: [u8; 64],
}
