use procem::{processor::Processor, register::Register};

use crate::instruction::{Instruction, operand::Operand};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum MemoryLocation {
    Labeled(u64),
    Offset { base: Register, offset: Operand },
}

impl MemoryLocation {
    /// Resolve the memory location to a value.
    #[inline]
    pub fn resolve<const MEM_SIZE: usize, Insts, Bytes>(
        self,
        processor: &Processor<MEM_SIZE, Instruction, Insts, Bytes>,
    ) -> u64 {
        match self {
            Self::Labeled(addr) => addr,
            Self::Offset { base, offset } => processor.registers.get_reg(base) + offset.resolve(processor),
        }
    }
}
