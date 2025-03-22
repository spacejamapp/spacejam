//! Value type for the PVM

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
}

macro_rules! impl_from_bytes {
    () => {
        fn from_bytes(source: &[u8]) -> Option<Self> {
            if source.len() < Self::SIZE {
                return None;
            }

            let mut bytes = [0u8; Self::SIZE];
            bytes.copy_from_slice(source);
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

    impl_from_bytes!();
}

impl Value for u8 {
    const SIZE: usize = 1;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    impl_from_bytes!();
}

impl Value for i16 {
    const SIZE: usize = 2;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as i64 as u64
    }

    impl_from_bytes!();
}

impl Value for u16 {
    const SIZE: usize = 2;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    impl_from_bytes!();
}

impl Value for i32 {
    const SIZE: usize = 4;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as i64 as u64
    }

    impl_from_bytes!();
}

impl Value for u32 {
    const SIZE: usize = 4;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    impl_from_bytes!();
}

impl Value for i64 {
    const SIZE: usize = 8;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    impl_from_bytes!();
}

impl Value for u64 {
    const SIZE: usize = 8;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self
    }

    impl_from_bytes!();
}
