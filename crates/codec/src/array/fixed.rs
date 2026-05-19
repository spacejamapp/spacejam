//! Stack-backed fixed-size array.

use crate::Vec;
use core::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut, Index, IndexMut},
};
use serde::{
    de::{self, Error, SeqAccess, Visitor},
    ser::{self, SerializeTuple},
};

/// A fixed-size array backed by `[T; N]` on the stack.
///
/// Wire format: N elements concatenated with no length prefix.
#[repr(transparent)]
pub struct FixedArray<T, const N: usize>(pub [T; N]);

impl<T: ser::Serialize, const N: usize> ser::Serialize for FixedArray<T, N> {
    fn serialize<S: ser::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut tuple = serializer.serialize_tuple(N)?;
        for item in self.as_slice() {
            tuple.serialize_element(item)?;
        }
        tuple.end()
    }
}

impl<'de, T: de::Deserialize<'de>, const N: usize> de::Deserialize<'de> for FixedArray<T, N> {
    fn deserialize<D: de::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

        impl<'de, T: de::Deserialize<'de>, const N: usize> Visitor<'de> for ArrayVisitor<T, N> {
            type Value = FixedArray<T, N>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "an array of {N} elements")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                let mut v: Vec<T> = Vec::with_capacity(N);
                for i in 0..N {
                    v.push(
                        seq.next_element()?
                            .ok_or_else(|| A::Error::invalid_length(i, &self))?,
                    );
                }
                FixedArray::try_from_vec(v).map_err(|_| A::Error::custom("length mismatch"))
            }
        }

        deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
}

impl<T, const N: usize> FixedArray<T, N> {
    /// Slice view into the backing store.
    pub fn as_slice(&self) -> &[T] {
        &self.0
    }

    /// Mutable slice view into the backing store.
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        &mut self.0
    }

    /// Try to construct from a Vec. Fails if length != N.
    pub fn try_from_vec(v: Vec<T>) -> Result<Self, Vec<T>> {
        let arr: [T; N] = v.try_into()?;
        Ok(Self(arr))
    }
}

impl<T: Default, const N: usize> Default for FixedArray<T, N> {
    fn default() -> Self {
        Self(core::array::from_fn(|_| T::default()))
    }
}

impl<T: fmt::Debug, const N: usize> fmt::Debug for FixedArray<T, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_slice().fmt(f)
    }
}

impl<T: Clone, const N: usize> Clone for FixedArray<T, N> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T: PartialEq, const N: usize> PartialEq for FixedArray<T, N> {
    fn eq(&self, other: &Self) -> bool {
        self.as_slice() == other.as_slice()
    }
}

impl<T: Eq, const N: usize> Eq for FixedArray<T, N> {}

impl<T: PartialOrd, const N: usize> PartialOrd for FixedArray<T, N> {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        self.as_slice().partial_cmp(other.as_slice())
    }
}

impl<T: Ord, const N: usize> Ord for FixedArray<T, N> {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.as_slice().cmp(other.as_slice())
    }
}

impl<T: core::hash::Hash, const N: usize> core::hash::Hash for FixedArray<T, N> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.as_slice().hash(state);
    }
}

impl<T, const N: usize> Deref for FixedArray<T, N> {
    type Target = [T];

    fn deref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> DerefMut for FixedArray<T, N> {
    fn deref_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, const N: usize> AsRef<[T]> for FixedArray<T, N> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}

impl<T, const N: usize> AsMut<[T]> for FixedArray<T, N> {
    fn as_mut(&mut self) -> &mut [T] {
        self.as_mut_slice()
    }
}

impl<T, I, const N: usize> Index<I> for FixedArray<T, N>
where
    [T]: Index<I>,
{
    type Output = <[T] as Index<I>>::Output;

    fn index(&self, index: I) -> &Self::Output {
        &self.as_slice()[index]
    }
}

impl<T, I, const N: usize> IndexMut<I> for FixedArray<T, N>
where
    [T]: IndexMut<I>,
{
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}

impl<T, const N: usize> From<[T; N]> for FixedArray<T, N> {
    fn from(arr: [T; N]) -> Self {
        Self(arr)
    }
}

impl<T, const N: usize> From<FixedArray<T, N>> for [T; N] {
    fn from(arr: FixedArray<T, N>) -> Self {
        arr.0
    }
}

impl<T, const N: usize> TryFrom<Vec<T>> for FixedArray<T, N> {
    type Error = Vec<T>;

    fn try_from(v: Vec<T>) -> Result<Self, Self::Error> {
        Self::try_from_vec(v)
    }
}

impl<T, const N: usize> IntoIterator for FixedArray<T, N> {
    type Item = T;
    type IntoIter = core::array::IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a FixedArray<T, N> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut FixedArray<T, N> {
    type Item = &'a mut T;
    type IntoIter = core::slice::IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_mut_slice().iter_mut()
    }
}

impl<T: Copy, const N: usize> Copy for FixedArray<T, N> {}
