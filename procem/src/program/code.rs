use core::{
    marker::PhantomData,
    ops::{Deref, Index},
};

use crate::instruction::Instruction;

/// `Code` represents executable instructions within a [`Program`](crate::program::Program).
///
/// This generic container can wrap any type that dereferences to a slice of [`Instruction`]s.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Code<Inst, Insts>(Insts, PhantomData<Inst>);

impl<Inst, Insts> Code<Inst, Insts>
where
    Inst: Instruction,
    Insts: Deref<Target = [Inst]>,
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

impl<Inst, Insts> Deref for Code<Inst, Insts>
where
    Inst: Instruction,
    Insts: Deref<Target = [Inst]>,
{
    type Target = [Inst];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<Inst, Insts> From<Insts> for Code<Inst, Insts>
where
    Inst: Instruction,
    Insts: Deref<Target = [Inst]>,
{
    fn from(instructions: Insts) -> Self {
        Self(instructions, PhantomData)
    }
}

impl<Inst, Insts> Index<u64> for Code<Inst, Insts>
where
    Inst: Instruction,
    Insts: Deref<Target = [Inst]>,
{
    type Output = Inst;

    /// Get a reference to the instruction at the given program counter.
    ///
    /// # Panics
    /// Panics if the program counter is out of bounds.
    #[inline]
    fn index(&self, pc: u64) -> &Self::Output {
        // Not more than usize addressable
        let addr = pc as usize;

        self.get(addr).unwrap_or_else(|| {
            panic!(
                "Program counter out of bounds. Program length: {}, Program counter: {}",
                self.len(),
                pc
            )
        })
    }
}
