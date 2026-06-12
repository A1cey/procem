#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Header {
    pub(crate) init_pc: u64,
    pub(crate) init_sp: u64,
}

impl Header {
    #[inline]
    #[must_use]
    pub const fn new(init_pc: u64, init_sp: u64) -> Self {
        Self { init_pc, init_sp }
    }

    /// Get the initial program counter.
    #[inline]
    #[must_use]
    pub const fn init_pc(&self) -> u64 {
        self.init_pc
    }

    /// Get the initial stack pointer.
    #[inline]
    #[must_use]
    pub const fn init_sp(&self) -> u64 {
        self.init_sp
    }
}
