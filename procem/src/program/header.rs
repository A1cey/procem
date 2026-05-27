use crate::word::Word;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Header<W> {
    pub(crate) init_pc: W,
    pub(crate) init_sp: W,
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
