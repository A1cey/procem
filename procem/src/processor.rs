//! The [`Processor`] and [`ProcessorBuilder`] structs.
use core::fmt::{Display, Formatter};
use core::ops::Deref;
use thiserror::Error;

use crate::instruction::Instruction;
use crate::memory::Memory;
use crate::program::Program;
use crate::register::{Register, Registers};

/// The [`Processor`] is the main component of the emulator. It represents a simplified real world processor with memory, registers and flags.
///
/// It can store a singular [`Program`].
/// It has 16 general purpose [`register`](crate::register)s,
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
/// - `Inst`: The instruction type; must implement [`Instruction`](crate::instruction::Instruction)
/// - `Insts`: A container of instructions dereferencing to `[Inst]` (allows `Vec`, arrays, slices, etc.)
/// - `Bytes`: A container of bytes dereferencing to `[u8]`
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Processor<'program, const MEM_SIZE: usize, Inst, Insts, Bytes> {
    pub registers: Registers,
    pub mem: Memory<MEM_SIZE>,
    program: Option<&'program Program<MEM_SIZE, Inst, Insts, Bytes>>,
}

impl<'program, const MEM_SIZE: usize, Inst, Insts, Bytes> Processor<'program, MEM_SIZE, Inst, Insts, Bytes>
where
    Inst: Instruction,
    Insts: Deref<Target = [Inst]>,
    Bytes: Deref<Target = [u8]>,
{
    #[must_use]
    #[inline]
    pub const fn builder() -> ProcessorBuilder<'program, MEM_SIZE, Inst, Insts, Bytes> {
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
    pub fn load_program(&mut self, program: &'program Program<MEM_SIZE, Inst, Insts, Bytes>) {
        self.program = Some(program);
        self.load_header();
        self.load_data();
        self.load_bss();
    }

    #[inline]
    fn load_data(&mut self) {
        match self.program {
            Some(program) => {
                let base_addr = program.data().base_addr();
                let data = program.data();
                self.mem[base_addr..data.len()].clone_from_slice(data.data());
            }
            None => unreachable!("This function is only called after program is loaded into processor."),
        }
    }

    #[inline]
    fn load_bss(&mut self) {
        match self.program {
            Some(program) => {
                let base_addr = program.bss().base_addr();
                let end_addr = program.bss().end_addr();
                self.mem[base_addr..end_addr].fill(0);
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

        Inst::execute(instruction, self)?;

        Ok(())
    }
}

impl<const MEM_SIZE: usize, Inst, Insts, Bytes> Display for Processor<'_, MEM_SIZE, Inst, Insts, Bytes>
where
    Inst: Instruction,
    Insts: Deref<Target = [Inst]>,
    Bytes: Deref<Target = [u8]>,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "Registers: \n{}\nMemory: \t\t{}", self.registers, self.mem)
    }
}

/// The [`ProcessorBuilder`] is used to create a [`Processor`].
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub struct ProcessorBuilder<'program, const MEM_SIZE: usize, Inst, Insts, Bytes> {
    registers: Option<Registers>,
    mem: Option<Memory<MEM_SIZE>>,
    program: Option<&'program Program<MEM_SIZE, Inst, Insts, Bytes>>,
}

impl<'program, const MEM_SIZE: usize, Inst, Insts, Bytes> ProcessorBuilder<'program, MEM_SIZE, Inst, Insts, Bytes>
where
    Inst: Instruction,
    Insts: Deref<Target = [Inst]>,
    Bytes: Deref<Target = [u8]>,
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
    pub const fn with_registers(mut self, registers: Registers) -> Self {
        self.registers = Some(registers);
        self
    }

    /// Sets the memory for the `ProcessorBuilder`.
    #[must_use]
    #[inline]
    pub const fn with_memory(mut self, mem: Memory<MEM_SIZE>) -> Self {
        self.mem = Some(mem);
        self
    }

    /// Sets the program for the `ProcessorBuilder`.
    #[must_use]
    #[inline]
    pub const fn with_program(mut self, program: &'program Program<MEM_SIZE, Inst, Insts, Bytes>) -> Self {
        self.program = Some(program);
        self
    }

    /// Builds the `Processor` with the given registers, memory and program.
    #[must_use]
    #[inline]
    pub fn build(self) -> Processor<'program, MEM_SIZE, Inst, Insts, Bytes> {
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
    PCOutOfBounds { pc: u64, program_len: u64 },
    #[error("No program loaded")]
    NoProgramLoaded,
    #[error("Out of bounds memory access. Memory size: {mem_size}, Accessed address: {addr}")]
    OutOfBoundsMemoryAccess { mem_size: usize, addr: u64 },
    #[error("Out of bounds memory access. Memory size: {mem_size}, Accessed addresses: {addr_range:?}")]
    OutOfBoundsRangeMemoryAccess {
        mem_size: usize,
        addr_range: core::ops::Range<u64>,
    },
    #[error("Invalid slice size when trying to write to a memory range. Expected: {expected}, Got: {got}.")]
    InvalidSliceSize { expected: usize, got: usize },
    #[error("Attempted division by zero.")]
    DivisionByZero,
}

#[cfg(test)]
mod test {
    use alloc::{vec, vec::Vec};

    use crate::{
        instruction::{Instruction, InstructionResult},
        program::{Bss, Code, Data, Header},
    };

    use super::*;

    extern crate alloc;

    #[derive(Debug, Clone, Copy)]
    struct Inst {}

    impl Instruction for Inst {
        fn execute<const MEM_SIZE: usize, Insts, Bytes>(
            _instruction: Self,
            _processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
        ) -> InstructionResult {
            Ok(())
        }
    }

    #[test]
    fn load_data() {
        const DATA_BASE_ADDR: u64 = 0;

        let mut processor = Processor::new();
        let program = Program::<32, Inst, Vec<_>, Vec<_>>::new(
            Header::new(0, 31),
            Data::new(DATA_BASE_ADDR, vec![1, 2, 3, 4, 5]),
            Bss::new(0, 0),
            Code::new(vec![]),
        );

        processor.program = Some(&program);

        processor.load_data();

        for offset in 0..5 {
            assert_eq!(processor.mem.read(DATA_BASE_ADDR + offset), offset as u8 + 1);
        }
    }

    #[test]
    fn load_program_only_data() {
        const DATA_BASE_ADDR: u64 = 0;

        let mut processor = Processor::new();
        let program = Program::<32, Inst, Vec<_>, Vec<_>>::new(
            Header::new(0, 31),
            Data::new(DATA_BASE_ADDR, vec![1, 2, 3, 4, 5]),
            Bss::new(0, 0),
            Code::new(vec![]),
        );

        processor.load_program(&program);

        for offset in 0..5 {
            assert_eq!(processor.mem.read(DATA_BASE_ADDR + offset), offset as u8 + 1);
        }
    }

    #[test]
    fn load_bss() {
        const BSS_SIZE: u64 = 10;

        let mut processor = Processor::new();
        let program = Program::<32, Inst, Vec<_>, Vec<_>>::new(
            Header::new(0, 31),
            Data::new(0, vec![]),
            Bss::new(0, BSS_SIZE),
            Code::new(vec![]),
        );

        processor.program = Some(&program);
        processor.mem[..].fill(1); // Set memory to non-zero value
        processor.load_bss(); // Set memory to zero

        for offset in 0..BSS_SIZE {
            assert_eq!(processor.mem.read(offset), 0);
        }

        // check that not too many values where set to 0
        for addr in BSS_SIZE..processor.mem.size() {
            assert_eq!(
                processor.mem.read(addr),
                1,
                "Address {addr} was set to 0, even though bss has a size of {BSS_SIZE} ranging from address {} to address {}.",
                program.bss().base_addr(),
                program.bss().end_addr()
            );
        }
    }

    #[test]
    fn load_programm_only_bss() {
        const BSS_SIZE: u64 = 10;

        let mut processor = Processor::new();
        let program = Program::<32, Inst, Vec<_>, Vec<_>>::new(
            Header::new(0, 31),
            Data::new(0, vec![]),
            Bss::new(0, BSS_SIZE),
            Code::new(vec![]),
        );

        processor.mem[..].fill(1); // Set memory to non-zero value
        processor.load_program(&program); // Set memory to zero

        for offset in 0..BSS_SIZE {
            assert_eq!(processor.mem.read(offset), 0);
        }

        // check that not too many values where set to 0
        for addr in BSS_SIZE..processor.mem.size() {
            assert_eq!(
                processor.mem.read(addr),
                1,
                "Address {addr} was set to 0, even though bss has a size of {BSS_SIZE} ranging from address {} to address {}.",
                program.bss().base_addr(),
                program.bss().end_addr()
            );
        }
    }

    // TODO: Implement similar test for other parts
}
