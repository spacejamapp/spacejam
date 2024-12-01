//! JAMCodec encoding

/// JAM encode based on the parity scale codec
pub trait Encode: scale::Encode {
    /// Encode the value to a byte vector
    fn encode(&self) -> Vec<u8> {
        scale::Encode::encode(self)
    }
}
