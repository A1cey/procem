//! The [`Instruction`] trait.

use core::fmt::Debug;
use core::ops::Deref;

use crate::{processor::Processor, word::Word};

/// The [`Instruction`] trait is implemented by all instructions or instruction sets that can be executed by the processor.
///
/// The [`procem_default`](../../procem_default/index.html) crate provides a default implementation of this trait using a custom instruction set.
/// Its [`execute`](Instruction::execute) method is used by the processor to execute the instruction.
pub trait Instruction<W: Word>: Debug + Copy {
    /// This function is called when an instruction is executed by the processor.
    fn execute<const MEM_SIZE: usize, P: Deref<Target = [Self]>>(
        instruction: Self,
        processor: &mut Processor<MEM_SIZE, Self, P, W>,
    );
}
