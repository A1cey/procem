use procem::{processor::Processor, register::Register, word::Word};

use crate::instruction::Instruction;

/// Operand for the instruction set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operand<W> {
    Register(Register),
    Value(W),
}

impl<W: Word> Operand<W> {
    /// Resolve the operand to a value.
    #[inline]
    pub fn resolve<const MEM_SIZE: usize, Insts, Words>(
        self,
        processor: &Processor<MEM_SIZE, Instruction<W>, Insts, W, Words>,
    ) -> W {
        match self {
            Self::Register(reg) => processor.registers.get_reg(reg),
            Self::Value(val) => val,
        }
    }
}
