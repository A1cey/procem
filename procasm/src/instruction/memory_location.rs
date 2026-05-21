use procem::{processor::Processor, register::Register};

use crate::{
    instruction::{Instruction, operand::Operand},
    word::ProcasmWord,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryLocation<W> {
    Direct(W),
    Indirect { base: Register, offset: Operand<W> },
}

impl<W: ProcasmWord> MemoryLocation<W> {
    /// Resolve the memory location to a value.
    #[inline]
    pub fn resolve<const MEM_SIZE: usize, Insts, Words>(
        self,
        processor: &Processor<MEM_SIZE, Instruction<W>, Insts, W, Words>,
    ) -> W {
        match self {
            Self::Direct(addr) => addr,
            Self::Indirect { base, offset } => processor.registers.get_reg(base) + offset.resolve(processor),
        }
    }
}
