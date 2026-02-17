//! The [`Processor`] and [`ProcessorBuilder`] structs.
use core::fmt::{Display, Formatter};
use core::ops::Deref;

use crate::instruction::Instruction;
use crate::program::{Program, ProgramError};
use crate::register::{Register, Registers};
use crate::stack::Stack;
use crate::word::Word;

/// The [`Processor`] is the main component of the emulator. It represents a simplified real world processor with a stack, registers and flags.
///
/// It can store a singular [`Program`].
/// It has [`GENERAL_REGISTER_COUNT`](crate::register::GENERAL_REGISTER_COUNT) general purpose [`register`](crate::register)s,
/// a program counter ([`pc`](crate::register::Registers::pc)), a stack pointer ([`sp`](crate::register::Registers::sp))
/// and 4 flags ([`C`](crate::register::Flag::C), [`S`](crate::register::Flag::S), [`V`](crate::register::Flag::V), [`Z`](crate::register::Flag::Z)).
/// It also has a stack of size `STACK_SIZE`.
///
/// The processor can be created by using the [`builder()`](Processor::builder()) method or the [`ProcessorBuilder`] directly or by using the [`new()`](Processor::new()) method.
/// Using the builder pattern allows specifying the initial registers, stack and program.
/// Any unspecifed values will be initialized to their default values.
/// Using the [`new()`](Processor::new()) method just creates a default processor.
/// The program is then loaded using the [`load_program()`](Processor::load_program()) method.
///
/// To run a loaded program two methods are provided:
/// - To run the entire program use [`run_program()`](Processor::run_program()).
/// - To run only the next instruction use [`execute_next_instruction()`](Processor::execute_next_instruction()).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Processor<'a, const STACK_SIZE: usize, I, P, W: Word> {
    pub registers: Registers<W>,
    pub stack: Stack<STACK_SIZE, W>,
    program: Option<&'a Program<I, P, W>>,
}

impl<'a, const STACK_SIZE: usize, I, P, W> Processor<'a, STACK_SIZE, I, P, W>
where
    I: Instruction<W>,
    P: Deref<Target = [I]>,
    W: Word,
{
    #[must_use]
    #[inline]
    pub const fn builder() -> ProcessorBuilder<'a, STACK_SIZE, I, P, W> {
        ProcessorBuilder::new()
    }

    /// Creates a new processor.
    #[must_use]
    #[inline]
    pub fn new() -> Self {
        Self {
            registers: Registers::new(),
            stack: Stack::new(),
            program: None,
        }
    }

    /// Loads a program into the processor.
    ///
    /// The program cannot be changed after being loaded. To make changes, an updated or entirely new program has to be loaded.
    #[inline]
    pub const fn load_program(&mut self, program: &'a Program<I, P, W>) {
        self.program = Some(program);
    }

    /// Runs the entire program.
    ///
    /// # Errors
    /// The execution of the program stops and a `ProgramError` is returned if an error occured during the fetching of an instruction.
    ///
    /// Note: The execution of an instruction will never return an error. If the instruction is valid it will not error.
    /// Invalid instructions are a major bug in the implementation of the instruction set that is used for the program.
    pub fn run_program(&mut self) -> Result<(), ProgramError> {
        loop {
            self.execute_next_instruction()?;
        }
    }

    /// Fetches the current instruction (where pc points to), increments the pc and then executes the instruction.
    ///
    /// # Errors
    /// Returns a `ProgramError` if an error occured during fetching.
    ///
    /// Note: The execution of an instruction will never return an error. If the instruction is valid it will not error.
    /// Invalid instructions are a major bug in the implementation of the instruction set that is used for the program.
    pub fn execute_next_instruction(&mut self) -> Result<(), ProgramError> {
        let program = self.program.as_ref().ok_or(ProgramError::NoProgramLoaded)?;

        let instruction = program.try_fetch(self.registers.pc())?;

        self.registers.inc(Register::PC);

        I::execute(instruction, self);

        Ok(())
    }
}

impl<const STACK_SIZE: usize, I, P, W> Display for Processor<'_, STACK_SIZE, I, P, W>
where
    I: Instruction<W>,
    P: Deref<Target = [I]>,
    W: Word,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "Registers: \n{}\nStack: \t\t{}", self.registers, self.stack)
    }
}

/// The [`ProcessorBuilder`] is used to create a [`Processor`].
#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Default)]
pub struct ProcessorBuilder<'a, const STACK_SIZE: usize, I, P, W> {
    registers: Option<Registers<W>>,
    stack: Option<Stack<STACK_SIZE, W>>,
    program: Option<&'a Program<I, P, W>>,
}

impl<'a, const STACK_SIZE: usize, I, P, W> ProcessorBuilder<'a, STACK_SIZE, I, P, W>
where
    I: Instruction<W>,
    P: Deref<Target = [I]>,
    W: Word,
{
    /// Creates a new `ProcessorBuilder` with registers, stack and program set to `None`.
    #[inline]
    const fn new() -> Self {
        Self {
            registers: None,
            stack: None,
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

    /// Sets the stack for the `ProcessorBuilder`.
    #[must_use]
    #[inline]
    pub const fn with_stack(mut self, stack: Stack<STACK_SIZE, W>) -> Self {
        self.stack = Some(stack);
        self
    }

    /// Sets the program for the `ProcessorBuilder`.
    #[must_use]
    #[inline]
    pub const fn with_program(mut self, program: &'a Program<I, P, W>) -> Self {
        self.program = Some(program);
        self
    }

    /// Builds the `Processor` with the given registers, stack and program.
    #[must_use]
    #[inline]
    pub fn build(self) -> Processor<'a, STACK_SIZE, I, P, W> {
        Processor {
            registers: self.registers.unwrap_or_default(),
            stack: self.stack.unwrap_or_default(),
            program: self.program,
        }
    }
}
