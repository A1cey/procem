//! **`procem`** provides a [`Processor`](processor::Processor) structure, which emulates a real-world processor architecture.
//!
//! The design is loosely based on the ARM architecture.
//!
//! The processor operates by loading and executing an assembly [`Program`](program::Program).
//! A [`Program`](program::Program) is a collection of assembly instructions that the processor iterates over and executes.
//! The instruction set in use must implement the [`Instruction`](instruction::Instruction) trait.
//! A default instruction set is available in the [`procasm`](../procasm/index.html) crate.
//!
//! The [`Registers`](register::Registers) and [`Memory`](memory::Memory) use [`Word`](word::Word) as their data type.
//!
//! The processor’s [`Registers`](register::Registers), [`Flags`](register::Flag) and [`Memory`](memory::Memory)
//! are directly accessible and modifiable through the [`Processor`](processor::Processor) structure.
//!
//! ```
//! # use procem::register::{Flag, Register};
//! # use procem::processor::Processor;
//! # use procem::instruction::Instruction;
//! # use procem::word::{I32, Word};
//! # use core::marker::PhantomData;
//! # use core::ops::Deref;
//! #
//! # #[derive(Debug, PartialEq, Eq, Clone, Copy, Ord, PartialOrd, Hash)]
//! # struct Inst<W: Word> (PhantomData<W>);
//! #
//! # impl<W: Word> Instruction<W> for Inst<W> {
//! #     fn execute<const MEM_SIZE: usize, Insts, Words>(
//! #         instruction: Self,
//! #         processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>
//! #     ) {}
//! # }
//! #
//! # let mut processor = Processor::<2048, _, Vec<Inst<I32>>, _, Vec<I32>>::new();
//! let r0 = processor.registers.get_reg(Register::R0);
//! processor.registers.set_reg(Register::R1, r0);
//!
//! let overflow = processor.registers.get_flag(Flag::V);
//!
//! processor.mem.write(processor.registers.get_reg(Register::SP), 10.into());
//! let val = processor.mem.read(processor.registers.get_reg(Register::SP));
//! assert_eq!(val, 10.into());
//! ```

#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod instruction;
pub mod memory;
pub mod processor;
pub mod program;
pub mod register;
pub mod word;
