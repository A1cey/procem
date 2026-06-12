#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Bss {
    base_addr: u64,
    size: u64,
}

impl Bss {
    /// Create a new BSS section.
    ///
    /// `base_addr` is the start address; `size` is the number of bytes in this memory region.
    #[inline]
    #[must_use]
    pub const fn new(base_addr: u64, size: u64) -> Self {
        Self { base_addr, size }
    }

    /// Get the base address of the BSS section.
    #[inline]
    #[must_use]
    pub const fn base_addr(&self) -> u64 {
        self.base_addr
    }

    /// Get the size of the BSS section.
    #[inline]
    #[must_use]
    pub const fn size(&self) -> u64 {
        self.size
    }

    /// Compute the end address of the BSS region (exclusive) by adding `base_addr` and `size`.
    #[must_use]
    #[inline]
    pub fn end_addr(&self) -> u64 {
        self.base_addr + self.size
    }

    /// Whether the BSS region is empty (size == 0).
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}
