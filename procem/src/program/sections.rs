use core::ops::Deref;

use crate::word::Word;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Header<W> {
    init_pc: W,
    init_sp: W,
}

impl<W: Word> Header<W> {
    #[inline]
    #[must_use]
    pub const fn new(init_pc: W, init_sp: W) -> Self {
        Self { init_pc, init_sp }
    }

    /// Get the initial program counter.
    #[inline]
    #[must_use]
    pub const fn init_pc(&self) -> W {
        self.init_pc
    }

    /// Get the initial stack pointer.
    #[inline]
    #[must_use]
    pub const fn init_sp(&self) -> W {
        self.init_sp
    }
}

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
