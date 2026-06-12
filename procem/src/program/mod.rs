//! The [`Program`] definition.
mod bss;
mod code;
mod data;
mod header;

pub use bss::Bss;
pub use code::Code;
pub use data::Data;
pub use header::Header;

use crate::instruction::Instruction;
use crate::processor::ProcessorError;
use core::ops::Deref;

/// `Program` represents an executable image for the [`Processor`](crate::processor::Processor).
///
/// It contains:
/// - a [`Header`] with initial values for PC and SP,
/// - a [`Data`] section (initialized bytes with a base address),
/// - a [`BSS`](Bss) section (uninitialized memory region),
/// - the executable [`Code`] (the program instructions).
///
/// An instruction can be fetched from the program using the [`fetch`](Program::fetch), [`try_fetch`](Program::try_fetch), or [`fetch_unchecked`](Program::fetch_unchecked) methods.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Program<const MEM_SIZE: usize, Inst, Insts, Bytes> {
    header: Header,
    data: Data<Bytes>,
    bss: Bss,
    code: Code<Inst, Insts>,
}

impl<const MEM_SIZE: usize, Inst, Insts, Bytes> Program<MEM_SIZE, Inst, Insts, Bytes>
where
    Inst: Instruction,
    Insts: Deref<Target = [Inst]>,
    Bytes: Deref<Target = [u8]>,
{
    #[inline]
    #[must_use]
    pub const fn new(header: Header, data: Data<Bytes>, bss: Bss, code: Code<Inst, Insts>) -> Self {
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
    pub fn try_fetch(&self, pc: u64) -> Result<Inst, ProcessorError> {
        self.code
            .get(
                // Not more than usize values addressable
                pc as usize,
            )
            .map_or_else(
                || {
                    Err(ProcessorError::PCOutOfBounds {
                        pc,
                        // because pc is u64 and out of bounds (pc >= code.len()), this cast cannot loose data
                        program_len: self.code.len() as u64,
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
    pub fn fetch(&self, pc: u64) -> Inst {
        self.code[pc]
    }

    /// Returns the instruction at the provided index, without doing bounds checking.
    ///
    /// For a safe alternative see [`fetch`](Program::fetch).
    ///
    /// # Safety
    /// Calling this method with an out-of-bounds program counter value is undefined behavior even if the resulting value is not used.
    #[inline]
    #[must_use]
    pub unsafe fn fetch_unchecked(&self, pc: u64) -> Inst {
        // Not more than usize values addressable
        let addr = pc as usize;

        // SAFETY: The caller must uphold safety and provide an in-bounds program counter value.
        *unsafe { self.code.get_unchecked(addr) }
    }

    /// Get a reference to the header.
    #[inline]
    #[must_use]
    pub const fn header(&self) -> &Header {
        &self.header
    }

    /// Convenience: initial program counter from the header.
    #[inline]
    #[must_use]
    pub const fn init_pc(&self) -> u64 {
        self.header.init_pc()
    }

    /// Convenience: initial stack pointer from the header.
    #[inline]
    #[must_use]
    pub const fn init_sp(&self) -> u64 {
        self.header.init_sp()
    }

    /// Get a reference to the code.
    #[inline]
    #[must_use]
    pub const fn code(&self) -> &Code<Inst, Insts> {
        &self.code
    }

    /// Get a reference to the data section.
    #[inline]
    #[must_use]
    pub const fn data(&self) -> &Data<Bytes> {
        &self.data
    }

    /// Get a reference to the BSS section.
    #[inline]
    #[must_use]
    pub const fn bss(&self) -> &Bss {
        &self.bss
    }
}
