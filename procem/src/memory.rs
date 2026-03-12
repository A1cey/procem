//! The processor's [`Memory`].

use ars::fmt::slice::FmtSlice;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Deref, DerefMut, Index, IndexMut};

use crate::processor::ProcessorError;
use crate::word::Word;

/// The [`Memory`] is a wrapper around a fixed-size array of values implementing the [`Word`] trait.
///
/// It can be read with the [`read`](Memory::read) or [`try_read`](Memory::try_read) methods. It can also be written to with the [`write`](Memory::write) or [`try_write`](Memory::try_write) methods.
/// For both reading and writing, the address needs to be provided.
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
/// #     fn execute<const MEM_SIZE: usize, Insts, Words>(
/// #         instruction: Self,
/// #         processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>
/// #     ) {}
/// # }
/// # let mut processor = Processor::<4, _,  Vec<Inst<I64>>,_, Vec<I64>>::new();
/// // Default memory values are all zero.
/// assert_eq!(processor.mem.read(processor.registers.get_reg(Register::SP)), 0.into());
///
/// processor.mem.write(processor.registers.get_reg(Register::SP), 1.into());
/// assert_eq!(processor.mem.read(processor.registers.get_reg(Register::SP)), 1.into());
///
/// processor.registers.inc(Register::SP);
/// processor.mem.write(processor.registers.get_reg(Register::SP), 10.into());
/// assert_eq!(processor.mem.read(processor.registers.get_reg(Register::SP)), 10.into());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Memory<const MEM_SIZE: usize, W>([W; MEM_SIZE]);

impl<const MEM_SIZE: usize, W: Word> Deref for Memory<MEM_SIZE, W> {
    type Target = [W; MEM_SIZE];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<const MEM_SIZE: usize, W: Word> DerefMut for Memory<MEM_SIZE, W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<const MEM_SIZE: usize, W: Word> Default for Memory<MEM_SIZE, W> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MEM_SIZE: usize, W: Word> Display for Memory<MEM_SIZE, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", FmtSlice(self.deref().as_slice()))
    }
}

impl<const MEM_SIZE: usize, W: Word> Index<usize> for Memory<MEM_SIZE, W> {
    type Output = W;

    /// Get a reference to the value in memory at the provided address.
    ///
    /// # Panics
    /// Panics if the address is out of bounds.
    fn index(&self, addr: usize) -> &Self::Output {
        self.get(addr).unwrap_or_else(|| {
            panic!(
                "{}",
                ProcessorError::OutOfBoundsMemoryAccess {
                    mem_size: MEM_SIZE,
                    addr,
                }
            )
        })
    }
}

impl<const MEM_SIZE: usize, W: Word> IndexMut<usize> for Memory<MEM_SIZE, W> {
    /// Get a mutable reference to the value in memory at the provided address.
    ///
    /// # Panics
    /// Panics if the address is out of bounds.
    fn index_mut(&mut self, addr: usize) -> &mut Self::Output {
        self.get_mut(addr).unwrap_or_else(|| {
            panic!(
                "{}",
                ProcessorError::OutOfBoundsMemoryAccess {
                    mem_size: MEM_SIZE,
                    addr,
                }
            )
        })
    }
}

impl<const MEM_SIZE: usize, W: Word> Memory<MEM_SIZE, W> {
    /// Create memory with all elements initialized to the default word value.
    #[must_use]
    pub fn new() -> Self {
        Self([W::default(); MEM_SIZE])
    }

    /// Read a value from memory at the provided address.
    ///
    /// For a non-panicking alternative see [`try_read`](Memory::try_read).
    ///
    /// # Panics
    /// Panics if the address is out of bounds.
    pub fn read(&self, addr: W) -> W {
        self[addr.into()]
    }

    /// Read a value from memory at the provided address.
    ///
    /// # Errors
    /// Returns an [`OutOfBoundsMemoryAccess`](ProcessorError::OutOfBoundsMemoryAccess) error if the address is out of bounds.
    pub fn try_read(&self, addr: W) -> Result<W, ProcessorError> {
        let addr: usize = addr.into();

        self.get(addr).copied().ok_or(ProcessorError::OutOfBoundsMemoryAccess {
            mem_size: MEM_SIZE,
            addr,
        })
    }

    /// Read a value from the memory at the given address, without doing bounds checking.
    ///
    /// For a safe alternative see [`read`](Memory::read).
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds address value is undefined behavior.
    pub unsafe fn read_unchecked(&self, addr: W) -> W {
        // SAFETY: The caller must uphold safety and provide an in-bounds address value.
        *unsafe { self.get_unchecked(addr.into()) }
    }

    /// Write a value to memory at the given address.
    ///
    /// For a non-panicking alternative see [`try_write`](Memory::try_write).
    ///
    /// # Panics
    /// Panics if the address is out of bounds.
    pub fn write(&mut self, addr: W, value: W) {
        self[addr.into()] = value;
    }

    /// Write a value to memory at the given address.
    ///
    /// # Errors
    /// Returns an [`OutOfBoundsMemoryAccess`](ProcessorError::OutOfBoundsMemoryAccess) error if the address is out of bounds.
    pub fn try_write(&mut self, addr: W, value: W) -> Result<(), ProcessorError> {
        let addr: usize = addr.into();

        *self.get_mut(addr).ok_or(ProcessorError::OutOfBoundsMemoryAccess {
            mem_size: MEM_SIZE,
            addr,
        })? = value;

        Ok(())
    }

    /// Write a value to memory at the given address, without doing bounds checking.
    ///
    /// For a safe alternative see [`write`](Memory::write).
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds address value is undefined behavior.
    pub unsafe fn write_unchecked(&mut self, addr: W, value: W) {
        // SAFETY: The caller must uphold safety and provide an in-bounds address value.
        *unsafe { self.get_unchecked_mut(addr.into()) } = value;
    }
}
