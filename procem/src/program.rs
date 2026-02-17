//! The [`Program`] struct.
use core::marker::PhantomData;
use core::ops::{Deref, Index};
use thiserror::Error;

use crate::instruction::Instruction;
use crate::word::Word;

/// [`Program`] is a container for a sequence of instructions that is executed by the [`Processor`](crate::processor::Processor).
///
/// An instruction can be fetched from the program using the [`fetch`](Program::fetch) or [`try_fetch`](Program::try_fetch) methods.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Program<I, T, W>(T, PhantomData<(I, W)>);

impl<T, I, W: Word> Deref for Program<I, T, W>
where
    I: Instruction<W>,
    T: Deref<Target = [I]>,
{
    type Target = [I];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<I, T, W> From<T> for Program<I, T, W>
where
    I: Instruction<W>,
    T: Deref<Target = [I]>,
    W: Word,
{
    fn from(instructions: T) -> Self {
        Self(instructions, PhantomData)
    }
}

impl<I, T, W> Index<usize> for Program<I, T, W>
where
    I: Instruction<W>,
    T: Deref<Target = [I]>,
    W: Word,
{
    type Output = I;

    /// Get a reference to the instruction at the given program counter.
    ///
    /// # Panics
    /// Panics if the program counter is out of bounds.
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

impl<T, I, W> Program<I, T, W>
where
    I: Instruction<W>,
    T: Deref<Target = [I]>,
    W: Word,
{
    /// Creates a new program from the provided instructions.
    #[must_use]
    pub fn new(instructions: T) -> Self {
        instructions.into()
    }

    /// Returns the instruction at the provided index.
    ///
    /// # Errors
    /// Returns `PCOutOfBounds` error if the program counter is not in bounds.
    #[inline]
    pub fn try_fetch(&self, pc: W) -> Result<I, ProgramError> {
        let pc: usize = pc.into();

        self.get(pc).map_or_else(
            || {
                Err(ProgramError::PCOutOfBounds {
                    pc,
                    program_len: self.len(),
                })
            },
            |instruction| Ok(*instruction),
        )
    }

    /// Returns the instruction at the provided index.
    ///
    /// For a non-panicking alternative see [`try_fetch`](Program::try_fetch).
    ///
    /// # Panics
    /// Panics if the program counter is not in bounds.
    #[inline]
    pub fn fetch(&self, pc: W) -> I {
        self[pc.into()]
    }

    /// Returns the instruction at the provided index, without doing bounds checking.
    ///
    /// For a safe alternative see [`fetch`](Program::fetch).
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds program counter value is undefined behavior even if the resulting value is not used.
    #[inline]
    pub unsafe fn fetch_unchecked(&self, pc: W) -> I {
        // SAFETY: The caller must uphold safety and provide an in-bounds program counter value.
        *unsafe { self.get_unchecked(pc.into()) }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ProgramError {
    #[error("Program counter out of bounds. Program length: {program_len}, Program counter: {pc}")]
    PCOutOfBounds { pc: usize, program_len: usize },
    #[error("No program loaded")]
    NoProgramLoaded,
    #[error("Out of bounds stack access. Stack size: {stack_size}, Stack pointer: {stack_pointer}")]
    OutOfBoundsStackAccess { stack_size: usize, stack_pointer: usize },
}
