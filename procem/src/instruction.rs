//! The [`Instruction`] trait.

use core::fmt::Debug;

use crate::processor::{Processor, ProcessorError};

pub type InstructionResult = Result<(), ProcessorError>;

/// The [`Instruction`] trait is implemented by all instructions or instruction sets that can be executed by the processor.
///
/// The [`procasm`](../../procasm/index.html) crate provides a default implementation of this trait using a custom instruction set.
/// Its [`execute`](Instruction::execute) method is used by the processor to execute the instruction.
pub trait Instruction: Debug + Copy {
    /// This function is called when an instruction is executed by the processor.
    fn execute<const MEM_SIZE: usize, Insts, Words>(
        instruction: Self,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Words>,
    ) -> InstructionResult;
}
