use procem::{processor::Processor, register::Flag, word::Word};

use crate::instruction::Instruction;

/// Jump condition for the instruction set.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum JumpCondition {
    /// No condition. \[JMP\]
    Unconditional,
    /// If zero flag is set. \[JZ\]
    Zero,
    /// If zero flag is not set. \[JNZ\]
    NotZero,
    /// If carry flag is set. \[JC\]
    Carry,
    /// If carry flag is not set. \[JNC\]
    NotCarry,
    /// If signed flag is set. \[JS\]
    Signed,
    /// If signed flag is not set. \[JNS\]
    NotSigned,
    /// If zero flag and signed flag are not set. \[JG\]
    Greater,
    /// If zero flag is not set and signed flag is set. \[JL\]
    Less,
    /// If zero flag is set or signed flag is not set. \[JGE\]
    GreaterOrEq,
    /// If zero flag or signed flag is set. \[JLE\]
    LessOrEq,
}

impl JumpCondition {
    /// Check the jump condition.
    #[inline]
    pub(crate) const fn check<const MEM_SIZE: usize, W: Word, Insts, Words>(
        self,
        processor: &Processor<MEM_SIZE, Instruction<W>, Insts, W, Words>,
    ) -> bool {
        let flags = &processor.registers;
        match self {
            Self::Unconditional => true,
            Self::Zero => flags.get_flag(Flag::Z),
            Self::NotZero => !flags.get_flag(Flag::Z),
            Self::Carry => flags.get_flag(Flag::C),
            Self::NotCarry => !flags.get_flag(Flag::C),
            Self::Signed => flags.get_flag(Flag::S),
            Self::NotSigned => !flags.get_flag(Flag::S),
            Self::Greater => !flags.get_flag(Flag::Z) && !flags.get_flag(Flag::S),
            Self::Less => !flags.get_flag(Flag::Z) && flags.get_flag(Flag::S),
            Self::GreaterOrEq => flags.get_flag(Flag::Z) || !flags.get_flag(Flag::S),
            Self::LessOrEq => flags.get_flag(Flag::Z) || flags.get_flag(Flag::S),
        }
    }
}
