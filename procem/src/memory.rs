//! The processor's [`Memory`].

use ars::fmt::slice::FmtSlice;
use core::fmt::{Debug, Display, Formatter};
use core::ops::{Index, IndexMut, Range, RangeFull};

use core::range::RangeInclusive;

use crate::processor::ProcessorError;

/// The [`Memory`] is a wrapper around a fixed-size array of bytes.
///
/// It can be read with the [`read`](Memory::read) or [`try_read`](Memory::try_read) methods. It can also be written to with the [`write`](Memory::write) or [`try_write`](Memory::try_write) methods.
/// For both reading and writing, the address needs to be provided.
/// ```
/// # use procem::register::{Flag, Register};
/// # use procem::processor::Processor;
/// # use procem::instruction::{Instruction, InstructionResult};
/// # use core::marker::PhantomData;
/// # use core::ops::Deref;
/// #
/// # #[derive(Debug, PartialEq, Eq, Clone, Copy, Ord, PartialOrd, Hash)]
/// # struct Inst;
/// #
/// # impl Instruction for Inst {
/// #     fn execute<const MEM_SIZE: usize, Insts, Bytes>(
/// #         instruction: Self,
/// #         processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>
/// #     ) -> InstructionResult { Ok(()) }
/// # }
/// # let mut processor = Processor::<4, _,  Vec<Inst>, Vec<_>>::new();
/// // Default memory values are all zero.
/// assert_eq!(processor.mem.read(processor.registers.get_reg(Register::SP)), 0);
///
/// processor.mem.write(processor.registers.get_reg(Register::SP), 1);
/// assert_eq!(processor.mem.read(processor.registers.get_reg(Register::SP)), 1);
///
/// processor.registers.inc(Register::SP);
/// processor.mem.write(processor.registers.get_reg(Register::SP), 10);
/// assert_eq!(processor.mem.read(processor.registers.get_reg(Register::SP)), 10);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(transparent)]
pub struct Memory<const MEM_SIZE: usize>([u8; MEM_SIZE]);

impl<const MEM_SIZE: usize> Default for Memory<MEM_SIZE> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const MEM_SIZE: usize> Display for Memory<MEM_SIZE> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "{}", FmtSlice(self.0.as_slice()))
    }
}

impl<const MEM_SIZE: usize> Index<u64> for Memory<MEM_SIZE> {
    type Output = u8;

    /// Get a reference to the value in memory at the provided address.
    ///
    /// # Panics
    /// Panics if the address is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    fn index(&self, addr: u64) -> &Self::Output {
        self.0
            .get(
                // Not more than usize is addressable
                addr as usize,
            )
            .unwrap_or_else(|| {
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

impl<const MEM_SIZE: usize> IndexMut<u64> for Memory<MEM_SIZE> {
    /// Get a mutable reference to the value in memory at the provided address.
    ///
    /// # Panics
    /// Panics if the address is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    fn index_mut(&mut self, addr: u64) -> &mut Self::Output {
        self.0
            .get_mut(
                // Not more than usize is addressable
                addr as usize,
            )
            .unwrap_or_else(|| {
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

impl<const MEM_SIZE: usize> Index<Range<u64>> for Memory<MEM_SIZE> {
    type Output = [u8];

    /// Get a reference to the slice of memory at the provided address range.
    ///
    /// # Panics
    /// Panics if the address range is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    fn index(&self, addr_range: Range<u64>) -> &Self::Output {
        self.0
            .get(
                // Not more than usize is addressable
                Range {
                    start: addr_range.start as usize,
                    end: addr_range.end as usize,
                },
            )
            .unwrap_or_else(|| {
                panic!(
                    "{}",
                    ProcessorError::OutOfBoundsRangeMemoryAccess {
                        mem_size: MEM_SIZE,
                        addr_range,
                    }
                )
            })
    }
}

impl<const MEM_SIZE: usize> IndexMut<Range<u64>> for Memory<MEM_SIZE> {
    /// Get a mutable reference to the slice of memory at the provided address range.
    ///
    /// # Panics
    /// Panics if the address range is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    fn index_mut(&mut self, addr_range: Range<u64>) -> &mut Self::Output {
        self.0
            .get_mut(
                // Not more than usize is addressable
                Range {
                    start: addr_range.start as usize,
                    end: addr_range.end as usize,
                },
            )
            .unwrap_or_else(|| {
                panic!(
                    "{}",
                    ProcessorError::OutOfBoundsRangeMemoryAccess {
                        mem_size: MEM_SIZE,
                        addr_range,
                    }
                )
            })
    }
}

impl<const MEM_SIZE: usize> Index<RangeFull> for Memory<MEM_SIZE> {
    type Output = [u8];

    /// Get a reference to the entire slice of memory.
    fn index(&self, index: RangeFull) -> &Self::Output {
        &self.0[index]
    }
}

impl<const MEM_SIZE: usize> IndexMut<RangeFull> for Memory<MEM_SIZE> {
    /// Get a mutable reference to the entire slice of memory.
    fn index_mut(&mut self, index: RangeFull) -> &mut Self::Output {
        &mut self.0[index]
    }
}

impl<const MEM_SIZE: usize> Index<RangeInclusive<u64>> for Memory<MEM_SIZE> {
    type Output = [u8];

    /// Get a reference to the slice of memory at the provided address range.
    ///
    /// # Panics
    /// Panics if the address range is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    fn index(&self, addr_range: RangeInclusive<u64>) -> &Self::Output {
        self.0
            .get(
                // Not more than usize is addressable
                RangeInclusive {
                    start: addr_range.start as usize,
                    last: addr_range.last as usize,
                },
            )
            .unwrap_or_else(|| {
                panic!(
                    "{}",
                    ProcessorError::OutOfBoundsRangeMemoryAccess {
                        mem_size: MEM_SIZE,
                        addr_range: addr_range.start..addr_range.last + 1
                    }
                )
            })
    }
}

impl<const MEM_SIZE: usize> IndexMut<RangeInclusive<u64>> for Memory<MEM_SIZE> {
    /// Get a mutable reference to the slice of memory at the provided address range.
    ///
    /// # Panics
    /// Panics if the address range is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    fn index_mut(&mut self, addr_range: RangeInclusive<u64>) -> &mut Self::Output {
        self.0
            .get_mut(
                // Not more than usize is addressable
                RangeInclusive {
                    start: addr_range.start as usize,
                    last: addr_range.last as usize,
                },
            )
            .unwrap_or_else(|| {
                panic!(
                    "{}",
                    ProcessorError::OutOfBoundsRangeMemoryAccess {
                        mem_size: MEM_SIZE,
                        addr_range: addr_range.start..addr_range.last + 1,
                    }
                )
            })
    }
}

impl<const MEM_SIZE: usize> Memory<MEM_SIZE> {
    /// Create memory with all elements initialized to the default word value.
    #[must_use]
    pub fn new() -> Self {
        Self([u8::default(); MEM_SIZE])
    }

    /// Get the size of the memory
    #[must_use]
    pub const fn size(&self) -> u64 {
        // only 64bit address space with sp being u64
        MEM_SIZE as u64
    }

    /// Read a value from memory at the provided address.
    ///
    /// For a non-panicking alternative see [`try_read`](Memory::try_read).
    ///
    /// # Panics
    /// Panics if the address is out of bounds.
    #[must_use]
    pub fn read(&self, addr: u64) -> u8 {
        self[addr]
    }

    /// Read a value from memory at the provided address.
    ///
    /// # Errors
    /// Returns an [`OutOfBoundsMemoryAccess`](ProcessorError::OutOfBoundsMemoryAccess) error if the address is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    pub fn try_read(&self, addr: u64) -> Result<u8, ProcessorError> {
        self.0
            .get(
                // Not more than usize is addressable
                addr as usize,
            )
            .copied()
            .ok_or(ProcessorError::OutOfBoundsMemoryAccess {
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
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    #[must_use]
    pub unsafe fn read_unchecked(&self, addr: u64) -> u8 {
        // Not more than usize is addressable
        let addr = addr as usize;

        // SAFETY: The caller must uphold safety and provide an in-bounds address value.
        *unsafe { self.0.get_unchecked(addr) }
    }

    /// Read a slice of bytes from memory starting at the specified address.
    ///
    /// The memory range written to spans from `start_addr` to `start_addr + len` (exclusive).
    ///
    /// For a non-panicking alternative, see [`try_read_slice`](Memory::try_read_slice).
    ///
    /// # Panics
    /// Panics if the address range exceeds the memory bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    #[must_use]
    pub fn read_slice(&mut self, start_addr: u64, len: usize) -> &[u8] {
        // Not more than usize is addressable
        let start_addr = start_addr as usize;
        &self.0[start_addr..start_addr + len]
    }

    /// Read a slice of bytes to memory starting at the specified address.
    ///
    /// The memory range written to spans from `start_addr` to `start_addr + len` (exclusive).
    ///
    /// # Errors
    /// Returns an [`OutOfBoundsMemoryAccess`](ProcessorError::OutOfBoundsMemoryAccess) error if the address range is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    pub fn try_read_slice(&mut self, start_addr: u64, len: usize) -> Result<&[u8], ProcessorError> {
        self.0
            .get(
                // Not more than usize is addressable
                start_addr as usize..start_addr as usize + len,
            )
            .ok_or(ProcessorError::OutOfBoundsRangeMemoryAccess {
                mem_size: MEM_SIZE,
                addr_range: start_addr..start_addr + len as u64,
            })
    }

    /// Read a slice of bytes to memory starting at the specified address, without doing bounds
    /// checking.
    ///
    /// The memory range written to spans from `start_addr` to `start_addr + len` (exclusive).
    ///
    /// For a safe alternative see [`read_slice`](Memory::read_slice).
    ///
    /// # Safety
    /// Calling this method is undefined behavior if the `start_addr` or `start_addr + len` is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    pub unsafe fn write_read_unchecked(&mut self, start_addr: u64, len: usize) -> &[u8] {
        // Not more than usize is addressable
        let start_addr = start_addr as usize;

        // SAFETY: The caller must uphold safety and provide an in-bounds address range.
        unsafe { self.0.get_unchecked(start_addr..start_addr + len) }
    }

    /// Write a value to memory at the given address.
    ///
    /// For a non-panicking alternative see [`try_write`](Memory::try_write).
    ///
    /// # Panics
    /// Panics if the address is out of bounds.
    pub fn write(&mut self, addr: u64, value: u8) {
        self[addr] = value;
    }

    /// Write a value to memory at the given address.
    ///
    /// # Errors
    /// Returns an [`OutOfBoundsMemoryAccess`](ProcessorError::OutOfBoundsMemoryAccess) error if the address is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    pub fn try_write(&mut self, addr: u64, value: u8) -> Result<(), ProcessorError> {
        *self
            .0
            .get_mut(
                // Not more than usize is addressable
                addr as usize,
            )
            .ok_or(ProcessorError::OutOfBoundsMemoryAccess {
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
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    pub unsafe fn write_unchecked(&mut self, addr: u64, value: u8) {
        // Not more than usize is addressable
        let addr = addr as usize;

        // SAFETY: The caller must uphold safety and provide an in-bounds address value.
        *unsafe { self.0.get_unchecked_mut(addr) } = value;
    }

    /// Write a slice of bytes to memory starting at the specified address.
    ///
    /// The memory range written to spans from `start_addr` to `start_addr + values.len()` (exclusive).
    ///
    /// For a non-panicking alternative, see [`try_write_slice`](Memory::try_write_slice).
    ///
    /// # Panics
    /// Panics if the address range exceeds the memory bounds.
    pub fn write_slice(&mut self, start_addr: u64, values: &[u8]) {
        // slices with more than u64
        debug_assert!(values.len() as u128 <= u128::from(u64::MAX));
        let end_addr = start_addr + values.len() as u64;

        self[start_addr..end_addr].copy_from_slice(values);
    }

    /// Write a slice of bytes to memory starting at the specified address.
    ///
    /// The memory range written to spans from `start_addr` to `start_addr + values.len()` (exclusive).
    ///
    /// # Errors
    /// Returns an [`OutOfBoundsMemoryAccess`](ProcessorError::OutOfBoundsMemoryAccess) error if the address range is out of bounds.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    pub fn try_write_slice(&mut self, start_addr: u64, values: &[u8]) -> Result<(), ProcessorError> {
        let slice = self
            .0
            .get_mut(
                // Not more than usize is addressable
                start_addr as usize..start_addr as usize + values.len(),
            )
            .ok_or(ProcessorError::OutOfBoundsRangeMemoryAccess {
                mem_size: MEM_SIZE,
                addr_range: start_addr..start_addr + values.len() as u64,
            })?;

        if slice.len() != values.len() {
            return Err(ProcessorError::InvalidSliceSize {
                expected: slice.len(),
                got: values.len(),
            });
        }

        // Cannot panic due to check above
        slice.copy_from_slice(values);

        Ok(())
    }

    /// Write a slice of bytes to memory starting at the specified address, without doing bounds
    /// checking.
    ///
    /// The memory range written to spans from `start_addr` to `start_addr + values.len()` (exclusive).
    ///
    /// For a safe alternative see [`write_slice`](Memory::write_slice).
    ///
    /// # Safety
    /// Calling this method is undefined behavior if:
    /// * The `start_addr` or `start_addr + values.len()` is out of bounds.
    /// * The address range in memory overlaps with the address range of `values`.
    #[expect(clippy::cast_possible_truncation, reason = "Not more than usize is addressable")]
    pub unsafe fn write_slice_unchecked(&mut self, start_addr: u64, values: &[u8]) {
        // Not more than usize is addressable
        let start_addr = start_addr as usize;

        let mem_ptr = self.0.as_mut_ptr();

        // SAFETY: The caller must uphold safety and provide a valid `start_addr`.
        let start_addr_ptr = unsafe { mem_ptr.add(start_addr) };

        // SAFETY: The caller must uphold safety and provide a valid address range (`start_addr + values.len()` must be in bounds).
        // The address range of values is not allowed to overlap with the address range of the destination in memory.
        unsafe {
            core::ptr::copy_nonoverlapping(values.as_ptr(), start_addr_ptr, values.len());
        }
    }
}
