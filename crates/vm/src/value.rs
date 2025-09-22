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

        fn to_vec(&self) -> Vec<u8> {
            self.to_le_bytes().to_vec()
        }
    };
}

impl Value for i8 {
    const SIZE: usize = 1;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as i64 as u64
    }

    impl_bytes!();
}

impl Value for u8 {
    const SIZE: usize = 1;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    impl_bytes!();
}

impl Value for i16 {
    const SIZE: usize = 2;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as i64 as u64
    }

    impl_bytes!();
}

impl Value for u16 {
    const SIZE: usize = 2;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    impl_bytes!();
}

impl Value for i32 {
    const SIZE: usize = 4;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as i64 as u64
    }

    impl_bytes!();
}

impl Value for u32 {
    const SIZE: usize = 4;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    impl_bytes!();
}

impl Value for i64 {
    const SIZE: usize = 8;
    const SIGNED: bool = true;

    fn as_u64(&self) -> u64 {
        *self as u64
    }

    impl_bytes!();
}

impl Value for u64 {
    const SIZE: usize = 8;
    const SIGNED: bool = false;

    fn as_u64(&self) -> u64 {
        *self
    }

    impl_bytes!();
}

/// Convert a slice of bytes to a i64
pub fn as_i64(bytes: &[u8]) -> Option<i64> {
    match bytes.len() {
        1 => Some(bytes[0] as i64),
        2 => Some(bytes[0] as i64 | (bytes[1] as i64) << 8),
        4 => Some(
            bytes[0] as i64
                | (bytes[1] as i64) << 8
                | (bytes[2] as i64) << 16
                | (bytes[3] as i64) << 24,
        ),
        8 => Some(
            bytes[0] as i64
                | (bytes[1] as i64) << 8
                | (bytes[2] as i64) << 16
                | (bytes[3] as i64) << 24
                | (bytes[4] as i64) << 32
                | (bytes[5] as i64) << 40
                | (bytes[6] as i64) << 48
                | (bytes[7] as i64) << 56,
        ),
        _ => None,
    }
}
