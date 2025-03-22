//! Value type for the PVM

use pvm_parser::format::ISA;

/// The value type for the PVM
pub trait Value: Sized {
    /// The size of the value in bytes
    const SIZE: usize;

    /// Whether the value is signed
    const SIGNED: bool;

    /// Convert the value to a u64
    fn as_u64(&self) -> u64;

    /// Convert a slice of bytes to a value
    fn from_bytes(bytes: &[u8]) -> Option<Self>;

    /// Convert the value to a slice of bytes
    fn to_vec(&self) -> Vec<u8>;
}

macro_rules! impl_bytes {
    () => {
        fn from_bytes(source: &[u8]) -> Option<Self> {
            if source.len() < Self::SIZE {
                return None;
            }

            let mut bytes = [0u8; Self::SIZE];
            bytes[..source.len()].copy_from_slice(source);
            Some(Self::from_le_bytes(bytes))
        }
    };
}

impl Value for i8 {
    const SIZE: usize = 1;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as i64 as u64
    }

    fn to_vec(&self) -> Vec<u8> {
        (*self as i64 as u64).bytes()
    }

    impl_bytes!();
}

impl Value for u8 {
    const SIZE: usize = 1;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    fn to_vec(&self) -> Vec<u8> {
        // (*self as u64).bytes()
        self.to_le_bytes().to_vec()
    }

    impl_bytes!();
}

impl Value for i16 {
    const SIZE: usize = 2;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as i64 as u64
    }

    fn to_vec(&self) -> Vec<u8> {
        // (*self as i64 as u64).bytes()
        self.to_le_bytes().to_vec()
    }

    impl_bytes!();
}

impl Value for u16 {
    const SIZE: usize = 2;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    fn to_vec(&self) -> Vec<u8> {
        // (*self as u64).bytes()
        self.to_le_bytes().to_vec()
    }

    impl_bytes!();
}

impl Value for i32 {
    const SIZE: usize = 4;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as i64 as u64
    }

    fn to_vec(&self) -> Vec<u8> {
        // (*self as i64 as u64).bytes()
        self.to_le_bytes().to_vec()
    }

    impl_bytes!();
}

impl Value for u32 {
    const SIZE: usize = 4;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    fn to_vec(&self) -> Vec<u8> {
        // (*self as u64).bytes()
        self.to_le_bytes().to_vec()
    }

    impl_bytes!();
}

impl Value for i64 {
    const SIZE: usize = 8;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    fn to_vec(&self) -> Vec<u8> {
        // (*self as u64).bytes()
        self.to_le_bytes().to_vec()
    }

    impl_bytes!();
}

impl Value for u64 {
    const SIZE: usize = 8;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self
    }

    fn to_vec(&self) -> Vec<u8> {
        (*self as u64).bytes()
        // self.to_le_bytes().to_vec()
    }

    impl_bytes!();
}
