//! The [`Processor`] and [`ProcessorBuilder`] structs.
use core::fmt::{Display, Formatter};
use core::ops::{Deref, Range};

use thiserror::Error;

use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::program::Program;
use crate::register::{Register, Registers};
use crate::word::Word;

/// The [`Processor`] is the main component of the emulator. It represents a simplified real world processor with memory, registers and flags.
///
/// It can store a singular [`Program`].
/// It has [`GENERAL_REGISTER_COUNT`](crate::register::GENERAL_REGISTER_COUNT) general purpose [`register`](crate::register)s,
/// a program counter ([`pc`](crate::register::Registers::pc)), a stack pointer ([`sp`](crate::register::Registers::sp))
/// and 4 flags ([`C`](crate::register::Flag::C), [`S`](crate::register::Flag::S), [`V`](crate::register::Flag::V), [`Z`](crate::register::Flag::Z)).
/// It also has memory of size `MEM_SIZE`.
///
/// The processor can be created by using the [`builder()`](Processor::builder()) method or the [`ProcessorBuilder`] directly or by using the [`new()`](Processor::new()) method.
/// Using the builder pattern allows specifying the initial registers, memory and program.
/// Any unspecifed values will be initialized to their default values.
/// Using the [`new()`](Processor::new()) method just creates a default processor.
/// The program is then loaded using the [`load_program()`](Processor::load_program()) method.
///
/// To run a loaded program two methods are provided:
/// - To run the entire program use [`run_program()`](Processor::run_program()).
/// - To run only the next instruction use [`execute_next_instruction()`](Processor::execute_next_instruction()).
///
/// # Generic Type Parameters
/// - `Inst`: The instruction type; must implement [`Instruction<W>`](crate::instruction::Instruction)
/// - `Insts`: A container of instructions dereferencing to `[Inst]` (allows `Vec`, arrays, slices, etc.)
/// - `W`: The word type; must implement [`Word`]
/// - `Words`: A container of words dereferencing to `[W]`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Processor<'program, const MEM_SIZE: usize, Inst, Insts, W: Word, Words> {
    pub registers: Registers<W>,
    pub mem: Memory<MEM_SIZE, W>,
    program: Option<&'program Program<MEM_SIZE, Inst, Insts, W, Words>>,
}

impl<'program, const MEM_SIZE: usize, Inst, Insts, W, Words> Processor<'program, MEM_SIZE, Inst, Insts, W, Words>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
    Words: Deref<Target = [W]>,
{
    #[must_use]
    #[inline]
    pub const fn builder() -> ProcessorBuilder<'program, MEM_SIZE, Inst, Insts, W, Words> {
        ProcessorBuilder::new()
    }

    /// Creates a new processor.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            mem: Memory::new(),
            program: None,
        }
    }

    /// Loads a program into the processor.
    ///
    /// The program cannot be changed after being loaded. You cannot mutate the program through the processor; replace it with a different [`Program`] to change behavior
    #[inline]
    pub fn load_program(&mut self, program: &'program Program<MEM_SIZE, Inst, Insts, W, Words>) {
        self.program = Some(program);
        self.load_header();
        self.load_data();
        self.load_bss();
    }

    #[inline]
    fn load_data(&mut self) {
        match self.program {
            Some(program) => {
                let base_addr = program.data().base_addr().into();
                let data = program.data().data();
                self.mem[base_addr..data.len()].clone_from_slice(data);
            }
            None => unreachable!("This function is only called after program is loaded into processor."),
        }
    }

    #[inline]
    fn load_bss(&mut self) {
        match self.program {
            Some(program) => {
                if program.bss().size() == W::from(0) {
                    return;
                }

                let base_addr = program.bss().base_addr().into();
                let end_addr = program.bss().end_addr().into();
                self.mem[base_addr..end_addr + 1].fill(W::from(0));
            }
            None => unreachable!("This function is only called after program is loaded into processor."),
        }
    }

    #[inline]
    fn load_header(&mut self) {
        match self.program {
            Some(program) => {
                let init_pc = program.header().init_pc();
                let init_sp = program.header().init_sp();

                self.registers.set_reg(Register::PC, init_pc);
                self.registers.set_reg(Register::SP, init_sp);
            }
            None => unreachable!("This function is only called after program is loaded into processor."),
        }
    }

    /// Runs the entire program.
    ///
    /// # Errors
    /// The execution of the program stops and a `ProcessorError` is returned if an error occured during the fetching of an instruction.
    ///
    /// Note: The execution of an instruction will never return an error. If the instruction is valid it will not error.
    /// Invalid instructions are a major bug in the implementation of the instruction set that is used for the program.
    pub fn run_program(&mut self) -> Result<(), ProcessorError> {
        loop {
            self.execute_next_instruction()?;
        }
    }

    /// Fetches the current instruction (where pc points to), increments the pc and then executes the instruction.
    ///
    /// # Errors
    /// Returns a `ProcessorError` if an error occured during fetching.
    ///
    /// Note: The execution of an instruction will never return an error. If the instruction is valid it will not error.
    /// Invalid instructions are a major bug in the implementation of the instruction set that is used for the program.
    pub fn execute_next_instruction(&mut self) -> Result<(), ProcessorError> {
        let program = self.program.as_ref().ok_or(ProcessorError::NoProgramLoaded)?;

        let instruction = program.try_fetch(self.registers.pc())?;

        self.registers.inc(Register::PC);

        Inst::execute(instruction, self);

        Ok(())
    }
}

impl<const MEM_SIZE: usize, Inst, Insts, W, Words> Display for Processor<'_, MEM_SIZE, Inst, Insts, W, Words>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
    Words: Deref<Target = [W]>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "Registers: \n{}\nMemory: \t\t{}", self.registers, self.mem)
    }
}

/// The [`ProcessorBuilder`] is used to create a [`Processor`].
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub struct ProcessorBuilder<'program, const MEM_SIZE: usize, Inst, Insts, W, Words> {
    registers: Option<Registers<W>>,
    mem: Option<Memory<MEM_SIZE, W>>,
    program: Option<&'program Program<MEM_SIZE, Inst, Insts, W, Words>>,
}

impl<'program, const MEM_SIZE: usize, Inst, Insts, W: Word, Words>
    ProcessorBuilder<'program, MEM_SIZE, Inst, Insts, W, Words>
where
    Inst: Instruction<W>,
    Insts: Deref<Target = [Inst]>,
    W: Word,
    Words: Deref<Target = [W]>,
{
    /// Creates a new `ProcessorBuilder` with registers, memory and program set to `None`.
    #[inline]
    const fn new() -> Self {
        Self {
            registers: None,
            mem: None,
            program: None,
        }
    }

    /// Sets the registers for the `ProcessorBuilder`.
    #[must_use]
    #[inline]
    pub const fn with_registers(mut self, registers: Registers<W>) -> Self {
        self.registers = Some(registers);
        self
    }

    /// Sets the memory for the `ProcessorBuilder`.
    #[must_use]
    #[inline]
    pub const fn with_memory(mut self, mem: Memory<MEM_SIZE, W>) -> Self {
        self.mem = Some(mem);
        self
    }

    /// Sets the program for the `ProcessorBuilder`.
    #[must_use]
    #[inline]
    pub const fn with_program(mut self, program: &'program Program<MEM_SIZE, Inst, Insts, W, Words>) -> Self {
        self.program = Some(program);
        self
    }

    /// Builds the `Processor` with the given registers, memory and program.
    #[must_use]
    #[inline]
    pub fn build(self) -> Processor<'program, MEM_SIZE, Inst, Insts, W, Words> {
        let mut processor = Processor {
            registers: self.registers.unwrap_or_default(),
            mem: self.mem.unwrap_or_default(),
            program: None,
        };
        if let Some(program) = self.program {
            processor.load_program(program);
        }

        processor
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum ProcessorError {
    #[error("Program counter out of bounds. Program length: {program_len}, Program counter: {pc}")]
    PCOutOfBounds { pc: usize, program_len: usize },
    #[error("No program loaded")]
    NoProgramLoaded,
    #[error("Out of bounds memory access. Memory size: {mem_size}, Accessed address: {addr}")]
    OutOfBoundsMemoryAccess { mem_size: usize, addr: usize },
    #[error("Out of bounds memory access. Memory size: {mem_size}, Accessed addresses: {addr_range:?}")]
    OutOfBoundsRangeMemoryAccess { mem_size: usize, addr_range: Range<usize> },
}

#[cfg(test)]
mod test {
    use alloc::{vec, vec::Vec};

    use crate::{
        instruction::Instruction,
        program::{Bss, Code, Data, Header},
        word::I32,
    };

    use super::*;

    extern crate alloc;

    #[derive(Debug, Clone, Copy)]
    struct Inst {}

    impl Instruction<I32> for Inst {
        fn execute<const MEM_SIZE: usize, Insts, Words>(
            _instruction: Self,
            _processor: &mut Processor<MEM_SIZE, Self, Insts, I32, Words>,
        ) {
        }
    }

    #[test]
    fn load_data() {
        let data_base_addr = I32::from(0);

        let mut processor = Processor::new();
        let program = Program::<32, Inst, Vec<_>, I32, Vec<_>>::new(
            Header::new(0.into(), 31.into()),
            Data::new(data_base_addr, vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into()]),
            Bss::new(0.into(), 0.into()),
            Code::new(vec![]),
        );

        processor.program = Some(&program);

        processor.load_data();

        for offset in 0..5 {
            assert_eq!(processor.mem.read(data_base_addr + offset.into()), (offset + 1).into());
        }
    }

    #[test]
    fn load_program_only_data() {
        let data_base_addr = I32::from(0);

        let mut processor = Processor::new();
        let program = Program::<32, Inst, Vec<_>, I32, Vec<_>>::new(
            Header::new(0.into(), 31.into()),
            Data::new(data_base_addr, vec![1.into(), 2.into(), 3.into(), 4.into(), 5.into()]),
            Bss::new(0.into(), 0.into()),
            Code::new(vec![]),
        );

        processor.load_program(&program);

        for offset in 0..5 {
            assert_eq!(processor.mem.read(data_base_addr + offset.into()), (offset + 1).into());
        }
    }

    // TODO: Implement similar test for other parts
}
