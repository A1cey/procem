#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Header {
    pub(crate) init_pc: usize,
    pub(crate) init_sp: usize,
}

impl Header {
    #[inline]
    #[must_use]
    pub const fn new(init_pc: usize, init_sp: usize) -> Self {
        Self { init_pc, init_sp }
    }

    /// Get the initial program counter.
    #[inline]
    #[must_use]
    pub const fn init_pc(&self) -> usize {
        self.init_pc
    }

    /// Get the initial stack pointer.
    #[inline]
    #[must_use]
    pub const fn init_sp(&self) -> usize {
        self.init_sp
    }
}
