//! The processor's [`Stack`].

use crate::helper;
use crate::program::ProgramError;
use crate::word::Word;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Deref, DerefMut, Index, IndexMut};

/// The [`Stack`] is a wrapper around a fixed-size array of values implementing the [`Word`] trait.
///
/// It can be read with the [`read`](Stack::read) or [`try_read`](Stack::try_read) methods. It can also be written to with the [`write`](Stack::write) or [`try_write`](Stack::try_write) methods.
/// For both reading and writing, the stack pointer needs to be provided.
/// ```
/// # use procem::register::{Flag, Register};
/// # use procem::processor::Processor;
/// # use procem::instruction::Instruction;
/// # use procem::word::{I64, Word};
/// # use core::marker::PhantomData;
/// # use core::ops::Deref;
/// #
/// # #[derive(Debug, PartialEq, Eq, Clone, Copy, Ord, PartialOrd, Hash)]
/// # struct Inst<W: Word> (PhantomData<W>);
/// #
/// # impl<W: Word> Instruction<W> for Inst<W> {
/// #     fn execute<const STACK_SIZE: usize, P: Deref<Target = [Self]>>(
/// #         instruction: Self,
/// #         processor: &mut Processor<STACK_SIZE, Self, P, W>
/// #     ) {}
/// # }
/// # let mut processor = Processor::<4, _,  Vec<Inst<I64>>,_>::new();
/// // Default stack values are all zero.
/// assert_eq!(processor.stack.read(processor.registers.get_reg(Register::SP)), 0.into());
///
/// processor.stack.write(processor.registers.get_reg(Register::SP), 1.into());
/// assert_eq!(processor.stack.read(processor.registers.get_reg(Register::SP)), 1.into());
///
/// processor.registers.inc(Register::SP);
/// processor.stack.write(processor.registers.get_reg(Register::SP), 10.into());
/// assert_eq!(processor.stack.read(processor.registers.get_reg(Register::SP)), 10.into());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Stack<const STACK_SIZE: usize, W>([W; STACK_SIZE]);

impl<const STACK_SIZE: usize, W: Word> Deref for Stack<STACK_SIZE, W> {
    type Target = [W; STACK_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const STACK_SIZE: usize, W: Word> DerefMut for Stack<STACK_SIZE, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const STACK_SIZE: usize, W: Word> Default for Stack<STACK_SIZE, W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const STACK_SIZE: usize, W: Word> Display for Stack<STACK_SIZE, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", helper::FmtArray(self.deref().as_slice()))
    }
}

impl<const STACK_SIZE: usize, W: Word> Index<usize> for Stack<STACK_SIZE, W> {
    type Output = W;

    /// Get a reference to the value on the stack at the given stack pointer.
    ///
    /// # Panics
    /// Panics if the stack pointer is out of bounds.
    fn index(&self, sp: usize) -> &Self::Output {
        self.get(sp)
            .unwrap_or_else(|| panic!("Out of bounds stack access. Stack size: {STACK_SIZE}, Stack pointer: {sp}"))
    }
}

impl<const STACK_SIZE: usize, W: Word> IndexMut<usize> for Stack<STACK_SIZE, W> {
    /// Get a mutable reference to the value on the stack at the given stack pointer.
    ///
    /// # Panics
    /// Panics if the stack pointer is out of bounds.
    fn index_mut(&mut self, sp: usize) -> &mut Self::Output {
        self.get_mut(sp)
            .unwrap_or_else(|| panic!("Out of bounds stack access. Stack size: {STACK_SIZE}, Stack pointer: {sp}"))
    }
}

impl<const STACK_SIZE: usize, W: Word> Stack<STACK_SIZE, W> {
    /// Create a new stack with all elements initialized to the default value.
    #[must_use]
    pub fn new() -> Self {
        Self([W::default(); STACK_SIZE])
    }

    /// Read a value from the stack at the given stack pointer.
    ///
    /// For a non-panicking alternative see [`try_read`](Stack::try_read).
    ///
    /// # Panics
    /// Panics if the stack pointer is out of bounds.
    pub fn read(&self, sp: W) -> W {
        self[sp.into()]
    }

    /// Read a value from the stack at the given stack pointer.
    ///
    /// # Errors
    /// Returns an [`OutOfBoundsStackAccess`](ProgramError::OutOfBoundsStackAccess) error if the stack pointer is out of bounds.
    pub fn try_read(&mut self, sp: W) -> Result<W, ProgramError> {
        let sp: usize = sp.into();

        self.get(sp).copied().ok_or(ProgramError::OutOfBoundsStackAccess {
            stack_size: STACK_SIZE,
            stack_pointer: sp,
        })
    }

    /// Read a value from the stack at the given stack pointer, without doing bounds checking.
    ///
    /// For a safe alternative see [`read`](Stack::read).
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds stack pointer value is undefined behavior.
    pub unsafe fn read_unchecked(&mut self, sp: W) -> W {
        // SAFETY: The caller must uphold safety and provide an in-bounds stack pointer value.
        *unsafe { self.get_unchecked(sp.into()) }
    }

    /// Write a value to the stack at the given stack pointer.
    ///
    /// For a non-panicking alternative see [`try_write`](Stack::try_write).
    ///
    /// # Panics
    /// Panics if the stack pointer is out of bounds.
    pub fn write(&mut self, sp: W, value: W) {
        self[sp.into()] = value;
    }

    /// Write a value to the stack at the given stack pointer.
    ///
    /// # Errors
    /// Returns an [`OutOfBoundsStackAccess`](ProgramError::OutOfBoundsStackAccess) error if the stack pointer is out of bounds.
    pub fn try_write(&mut self, sp: W, value: W) -> Result<(), ProgramError> {
        let sp: usize = sp.into();

        *self.get_mut(sp).ok_or(ProgramError::OutOfBoundsStackAccess {
            stack_size: STACK_SIZE,
            stack_pointer: sp,
        })? = value;

        Ok(())
    }

    /// Write a value to the stack at the given stack pointer, without doing bounds checking.
    ///
    /// For a safe alternative see [`write`](Stack::write).
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds stack pointer value is undefined behavior.
    pub unsafe fn write_unchecked(&mut self, sp: W, value: W) {
        // SAFETY: The caller must uphold safety and provide an in-bounds stack pointer value.
        *unsafe { self.get_unchecked_mut(sp.into()) } = value;
    }
}
