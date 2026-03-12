//! The [`Program`] definition.
mod code;
mod sections;

pub use code::Code;
pub use sections::{Bss, Data, Header};

use crate::instruction::Instruction;
use crate::processor::ProcessorError;
use crate::word::Word;
use core::ops::Deref;

// TODO: Program builder + validation pc in range of program or when loading program into processor?
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
    pub fn try_fetch(&self, pc: W) -> Result<Inst, ProcessorError> {
        let pc: usize = pc.into();

        self.code.get(pc).map_or_else(
            || {
                Err(ProcessorError::PCOutOfBounds {
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
