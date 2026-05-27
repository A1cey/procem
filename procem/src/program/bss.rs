use crate::word::Word;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Bss<W> {
    base_addr: W,
    size: W,
}

impl<W: Word> Bss<W> {
    /// Create a new BSS section.
    ///
    /// `base_addr` is the start address; `size` is the number of words in this memory region.
    #[inline]
    #[must_use]
    pub const fn new(base_addr: W, size: W) -> Self {
        Self { base_addr, size }
    }

    /// Get the base address of the BSS section.
    #[inline]
    #[must_use]
    pub const fn base_addr(&self) -> W {
        self.base_addr
    }

    /// Get the size of the BSS section.
    #[inline]
    #[must_use]
    pub const fn size(&self) -> W {
        self.size
    }

    /// Compute the end address of the BSS region (exclusive) by adding `base_addr` and `size`.
    ///
    /// Note: Wrapping behaviour is defined on the [`Word`] implementation.
    #[must_use]
    #[inline]
    pub fn end_addr(&self) -> W {
        self.base_addr + self.size
    }

    /// Whether the BSS region is empty (size == 0).
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == W::from(0)
    }
}
