//! The [`Program`] definition.
use core::marker::PhantomData;
use core::ops::{Deref, Index};
use thiserror::Error;

use crate::instruction::Instruction;
use crate::word::Word;

/// `Code` is a container for a sequence of instructions that is executed by the [`Processor`](crate::processor::Processor).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Code<Inst, Insts, W>(Insts, PhantomData<(Inst, W)>);

impl<Inst, Insts, W> Code<Inst, Insts, W>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
{
    /// Create a new code section.
    ///
    /// `base_addr` is the address where the first element of `data` will be loaded.
    #[inline]
    #[must_use]
    pub const fn new(code: Insts) -> Self {
        Self(code, PhantomData)
    }

    #[inline]
    #[must_use]
    pub fn num_of_instructions(&self) -> usize {
        self.0.deref().len()
    }

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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Header<W> {
    init_pc: W,
    init_sp: W,
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Data<W, D> {
    base_addr: W,
    data: D,
}

impl<W, Words> Data<W, Words>
where
    W: Word,
    Words: Deref<Target = [W]>,
{
    /// Create a new data section.
    ///
    /// `base_addr` is the address where the first element of `data` will be loaded.
    #[inline]
    #[must_use]
    pub const fn new(base_addr: W, data: Words) -> Self {
        Self { base_addr, data }
    }

    /// Get the base address of the data section.
    #[inline]
    #[must_use]
    pub const fn base_addr(&self) -> W {
        self.base_addr
    }

    /// Get a reference to the underlying data.
    #[inline]
    #[must_use]
    pub const fn data(&self) -> &Words {
        &self.data
    }

    /// Get the data as a slice of words.
    #[must_use]
    #[inline]
    pub fn as_slice(&self) -> &[W] {
        &self.data
    }

    /// Number of words in the data section.
    #[must_use]
    #[inline]
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }

    /// Whether the data region is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Bss<W> {
    base_addr: W,
    size: W,
}

impl<W: Word> Bss<W> {
    /// Create a new BSS section.
    ///
    /// `base_addr` is the start address; `size` is the number of words in this memory region.
    #[inline]
    #[must_use]
    pub const fn new(base_addr: W, size: W) -> Self {
        Self { base_addr, size }
    }

    /// Get the base address of the BSS section.
    #[inline]
    #[must_use]
    pub const fn base_addr(&self) -> W {
        self.base_addr
    }

    /// Get the size of the BSS section.
    #[inline]
    #[must_use]
    pub const fn size(&self) -> W {
        self.size
    }

    /// Compute the end address of the BSS region (exclusive) by adding `base_addr` and `size`.
    ///
    /// Note: Wrapping behaviour is defined on the [`Word`] implementation.
    #[must_use]
    #[inline]
    pub fn end_addr(&self) -> W {
        self.base_addr + self.size
    }

    /// Whether the BSS region is empty (size == 0).
    #[must_use]
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == W::from(0)
    }
}

// TODO: Program builder
/// `Program` represents an executable image for the [`Processor`](crate::processor::Processor).
///
/// It contains:
/// - a [`Header`] with initial values for PC and SP,
/// - a [`Data`] section (initialized words with a base address),
/// - a [`BSS`](Bss) section (uninitialized memory region),
/// - the executable [`Code`] (the program instructions).
///
/// An instruction can be fetched from the program using the [`fetch`](Program::fetch), [`try_fetch`](Program::try_fetch), or [`fetch_unchecked`](Program::fetch_unchecked) methods.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Program<Inst, Insts, W, Words> {
    header: Header<W>,
    data: Data<W, Words>,
    bss: Bss<W>,
    code: Code<Inst, Insts, W>,
}

impl<Inst, Insts, W, Words> Program<Inst, Insts, W, Words>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
    Words: Deref<Target = [W]>,
{
    #[inline]
    #[must_use]
    pub const fn new(header: Header<W>, data: Data<W, Words>, bss: Bss<W>, code: Code<Inst, Insts, W>) -> Self {
        Self {
            header,
            data,
            bss,
            code,
        }
    }

    /// Returns the instruction at the provided index.
    ///
    /// # Errors
    /// Returns `PCOutOfBounds` error if the program counter is not in bounds.
    #[inline]
    pub fn try_fetch(&self, pc: W) -> Result<Inst, ProgramError> {
        let pc: usize = pc.into();

        self.code.get(pc).map_or_else(
            || {
                Err(ProgramError::PCOutOfBounds {
                    pc,
                    program_len: self.code.len(),
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
    #[must_use]
    pub fn fetch(&self, pc: W) -> Inst {
        self.code[pc.into()]
    }

    /// Returns the instruction at the provided index, without doing bounds checking.
    ///
    /// For a safe alternative see [`fetch`](Program::fetch).
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds program counter value is undefined behavior even if the resulting value is not used.
    #[inline]
    #[must_use]
    pub unsafe fn fetch_unchecked(&self, pc: W) -> Inst {
        // SAFETY: The caller must uphold safety and provide an in-bounds program counter value.
        *unsafe { self.code.get_unchecked(pc.into()) }
    }

    /// Get a reference to the header.
    #[inline]
    #[must_use]
    pub const fn header(&self) -> &Header<W> {
        &self.header
    }

    /// Convenience: initial program counter from the header.
    #[inline]
    #[must_use]
    pub const fn init_pc(&self) -> W {
        self.header.init_pc()
    }

    /// Convenience: initial stack pointer from the header.
    #[inline]
    #[must_use]
    pub const fn init_sp(&self) -> W {
        self.header.init_sp()
    }

    /// Get a reference to the code.
    #[inline]
    #[must_use]
    pub const fn code(&self) -> &Code<Inst, Insts, W> {
        &self.code
    }

    /// Get a reference to the data section.
    #[inline]
    #[must_use]
    pub const fn data(&self) -> &Data<W, Words> {
        &self.data
    }

    /// Get a reference to the BSS section.
    #[inline]
    #[must_use]
    pub const fn bss(&self) -> &Bss<W> {
        &self.bss
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ProgramError {
    #[error("Program counter out of bounds. Program length: {program_len}, Program counter: {pc}")]
    PCOutOfBounds { pc: usize, program_len: usize },
    #[error("No program loaded")]
    NoProgramLoaded,
    #[error("Out of bounds memory access. Memory size: {mem_size}, Accessed address: {addr}")]
    OutOfBoundsMemoryAccess { mem_size: usize, addr: usize },
}
