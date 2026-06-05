use procem::{processor::Processor, register::Register};

use crate::instruction::Instruction;

/// Operand for the instruction set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operand {
    Register(Register),
    Value(usize),
}

impl Operand {
    /// Resolve the operand to a value.
    #[inline]
    pub fn resolve<const MEM_SIZE: usize, Insts, Bytes>(
        self,
        processor: &Processor<MEM_SIZE, Instruction, Insts, Bytes>,
    ) -> usize {
        match self {
            Self::Register(reg) => processor.registers.get_reg(reg),
            Self::Value(val) => val,
        }
    }
}
