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
<<<<<<< HEAD:procasm/src/instruction/operand.rs
    pub(crate) const fn resolve<const MEM_SIZE: usize, Insts, Words>(
=======
    pub(crate) fn resolve<const STACK_SIZE: usize, P>(
>>>>>>> 70ae210 (Replaced casts with usize::from in register.rs):procem_default/src/instruction/operand.rs
        self,
        processor: &Processor<MEM_SIZE, Instruction<W>, Insts, W, Words>,
    ) -> W {
        match self {
            Self::Register(reg) => processor.registers.get_reg(reg),
            Self::Value(val) => val,
        }
    }
}
