use core::ops::Deref;

use crate::word::Word;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Data<W, D> {
    base_addr: W,
    data: D,
}

impl<W, Words> Data<W, Words>
where
    W: Word,
    Words: Deref<Target = [W]>,
{
    /// Create a new data section.
    ///
    /// `base_addr` is the address where the first element of `data` will be loaded.
    #[inline]
    #[must_use]
    pub const fn new(base_addr: W, data: Words) -> Self {
        Self { base_addr, data }
    }

    /// Get the base address of the data section.
    #[inline]
    #[must_use]
    pub const fn base_addr(&self) -> W {
        self.base_addr
    }

    /// Get a reference to the underlying data.
    #[inline]
    #[must_use]
    pub const fn data(&self) -> &Words {
        &self.data
    }

    /// Get the data as a slice of words.
    #[must_use]
    #[inline]
    pub fn as_slice(&self) -> &[W] {
        &self.data
    }

    /// Number of words in the data section.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Whether the data region is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
