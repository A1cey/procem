use core::{
    marker::PhantomData,
    ops::{Deref, Index},
};

use crate::{instruction::Instruction, word::Word};

/// `Code` represents executable instructions within a [`Program`](crate::program::Program).
///
/// This generic container can wrap any type that dereferences to a slice of [`Instruction`]s.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Code<Inst, Insts, W>(Insts, PhantomData<(Inst, W)>);

impl<Inst, Insts, W> Code<Inst, Insts, W>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
{
    /// Create a new code section from a container of instructions.
    #[inline]
    #[must_use]
    pub const fn new(code: Insts) -> Self {
        Self(code, PhantomData)
    }

    /// Returns whether this code section contains no instructions.
    #[inline]
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl<Inst, Insts, W> Deref for Code<Inst, Insts, W>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
{
    type Target = [Inst];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Inst, Insts, W> From<Insts> for Code<Inst, Insts, W>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
{
    fn from(instructions: Insts) -> Self {
        Self(instructions, PhantomData)
    }
}

impl<Inst, Insts, W> Index<usize> for Code<Inst, Insts, W>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
{
    type Output = Inst;

    /// Get a reference to the instruction at the given program counter.
    ///
    /// # Panics
    /// Panics if the program counter is out of bounds.
    #[inline]
    fn index(&self, pc: usize) -> &Self::Output {
        self.get(pc).unwrap_or_else(|| {
            panic!(
                "Program counter out of bounds. Program length: {}, Program counter: {}",
                self.len(),
                pc
            )
        })
    }
}
