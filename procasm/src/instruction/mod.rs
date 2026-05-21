pub(crate) mod asm_instruction;
pub mod jump_condition;
pub mod operand;
pub mod unlinked;

use core::cmp::Ordering;
use std::mem::offset_of;
// TODO: Stack grows down not up
use procem::{
    instruction::Instruction as InstructionTrait,
    processor::Processor,
    register::{Flag, Register},
};

use crate::{
    instruction::{
        asm_instruction::{
            ASMJumpInstruction, ASMRegOperandInstruction, ASMRotateInstruction, ASMShiftInstruction,
            ASMSingleOperandInstruction, ASMSingleRegInstruction, ASMTwoOperandInstruction,
        },
        jump_condition::JumpCondition,
        operand::Operand,
    },
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
    pub(crate) fn resolve<const MEM_SIZE: usize, Insts, Words>(
        self,
        processor: &Processor<MEM_SIZE, Instruction<W>, Insts, W, Words>,
    ) -> W {
        match self {
            Self::Direct(addr) => addr,
            Self::Indirect { base, offset } => processor.registers.get_reg(base) + offset.resolve(processor),
        }
    }
}

/// A default instruction set implementation, that can be used for the [procem](../../procem/index.html) crate.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum Instruction<W> {
    /// No operation. (NOP)
    Nop,
    /// Copy a value from the operand to the register. (MOV)
    Mov { to: Register, from: Operand<W> },
    /// Store a value from a register to a memory location. (STR)
    Str { from: Register, to: MemoryLocation<W> },
    /// Load a value from a memory location into a register. (LDR)
    Ldr { to: Register, from: MemoryLocation<W> },
    /// Push a value from the operand to the stack. (PUSH)
    Push { from: Operand<W> },
    /// Pop a value from the stack to the register. (POP)
    Pop { to: Register },
    /// Call a subroutine at the program address specified by the operand.
    /// Pushes the current program counter onto the stack and sets the program counter to the address of the subroutine. (CALL)
    Call { addr: Operand<W> },
    /// Return from a subroutine.
    /// Pops the return address from the stack and sets the program counter to the popped value. (RET)
    Ret,
    /// Add the value of the operand (rhs) to the register (acc).
    /// The result is stored in acc. (ADD\[S\])
    Add {
        acc: Register,
        rhs: Operand<W>,
        signed: bool,
    },
    /// Subtract the value of the operand (rhs) from the register (acc).
    /// The result is stored in acc. (SUB\[S\])
    Sub {
        acc: Register,
        rhs: Operand<W>,
        signed: bool,
    },
    /// Multiply the value of the operand (rhs) with the value of the register (acc).
    /// The result is stored in acc. (MUL\[S\])
    Mul {
        acc: Register,
        rhs: Operand<W>,
        signed: bool,
    },
    /// Divide the value of the register (acc) by the value of the operand (rhs).
    /// The result is stored in acc. (DIV\[S\])
    Div {
        acc: Register,
        rhs: Operand<W>,
        signed: bool,
    },
    /// Increment the value in a register by one. (INC\[S\])
    Inc { reg: Register, signed: bool },
    /// Decrement the value in a register by one. (DEC\[S\])
    Dec { reg: Register, signed: bool },
    /// Set program counter to a value, effectively jumping to the instruction at this point in the program.
    /// The condition is checked before jumping and the jump is performed if the condition is met.
    /// See the assembly instruction at `JumpCondition`.
    Jump { to: W, condition: JumpCondition },
    /// Compare the values of two operands and set the flags accordingly. This is the same as `SUBS` but disregards the result of the subtraction. (CMP)
    Cmp { lhs: Operand<W>, rhs: Operand<W> },
    /// Perform an xor operation on the value in the register with the value of the operand. (XOR)
    Xor { reg: Register, rhs: Operand<W> },
    /// Perform an and operation on the value in the register with the value of the operand. (AND)
    And { reg: Register, rhs: Operand<W> },
    /// Perform an or operation on the value in the register with the value of the operand. (OR)
    Or { reg: Register, rhs: Operand<W> },
    /// Perform a not operation on the value in the register. (NOT)
    Not { reg: Register },
    /// Shift the value in the register left by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Shl { reg: Register, val: W },
    /// Shift the value in the register right by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Shr { reg: Register, val: W },
    /// Rotate the value in the register left by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Rol { reg: Register, val: u32 },
    /// Rotate the value in the register right by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Ror { reg: Register, val: u32 },
}

impl<W: ProcasmWord> InstructionTrait<W> for Instruction<W> {
    /// Execute an instruction on a processor.
    fn execute<const MEM_SIZE: usize, Insts, Words>(
        instruction: Self,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        match instruction {
            Self::Nop => (),
            Self::Mov { to, from } => Self::mov(to, from, processor),
            Self::Str { from, to } => Self::str(from, to, processor),
            Self::Ldr { to, from } => Self::ldr(to, from, processor),
            Self::Push { from } => Self::push(from, processor),
            Self::Pop { to } => Self::pop(to, processor),
            Self::Call { addr } => Self::call(addr, processor),
            Self::Ret => Self::ret(processor),
            Self::Add { acc, rhs, signed } => Self::add(acc, rhs, signed, processor),
            Self::Sub { acc, rhs, signed } => Self::sub(acc, rhs, signed, processor),
            Self::Mul { acc, rhs, signed } => Self::mul(acc, rhs, signed, processor),
            Self::Div { acc, rhs, signed } => Self::div(acc, rhs, signed, processor),
            Self::Inc { reg, signed } => Self::inc(reg, signed, processor),
            Self::Dec { reg, signed } => Self::dec(reg, signed, processor),
            Self::Jump { to, condition } => Self::jmp(to, condition, processor),
            Self::Cmp { lhs, rhs } => Self::cmp(lhs, rhs, processor),
            Self::Xor { reg, rhs } => Self::xor(reg, rhs, processor),
            Self::Or { reg, rhs } => Self::or(reg, rhs, processor),
            Self::And { reg, rhs } => Self::and(reg, rhs, processor),
            Self::Not { reg } => Self::not(reg, processor),
            Self::Shl { reg, val } => Self::shl(reg, val, processor),
            Self::Shr { reg, val } => Self::shr(reg, val, processor),
            Self::Rol { reg, val } => Self::rol(reg, val, processor),
            Self::Ror { reg, val } => Self::ror(reg, val, processor),
        }
    }
}

impl<W: ProcasmWord> Instruction<W> {
    // skips forrmatting the match
    #[rustfmt::skip]
    pub(crate) const fn from_reg_operand_instruction(
        instr: ASMRegOperandInstruction,
        lhs: Register,
        rhs: Operand<W>
    ) -> Self {
        use ASMRegOperandInstruction::{Mov, Add, AddS, Sub, SubS, Mul, MulS, Div, DivS, Or, And, Xor};
        match instr {
            Mov => Self::Mov { to: lhs, from: rhs },
            Add => Self::Add { acc: lhs, rhs, signed: false },
            AddS => Self::Add { acc: lhs, rhs, signed: true },
            Sub => Self::Sub { acc: lhs, rhs, signed: false },
            SubS => Self::Sub { acc: lhs, rhs, signed: true },
            Mul => Self::Mul { acc: lhs, rhs, signed: false },
            MulS => Self::Mul { acc: lhs, rhs, signed: true },
            Div => Self::Div { acc: lhs, rhs, signed: false },
            DivS => Self::Div { acc: lhs, rhs, signed: true },
            Or => Self::Or { reg: lhs, rhs },
            And => Self::And { reg: lhs, rhs },
            Xor => Self::Xor { reg: lhs, rhs },
        }
    }

    pub(crate) const fn from_single_reg_instruction(instr: ASMSingleRegInstruction, reg: Register) -> Self {
        use ASMSingleRegInstruction::{Dec, DecS, Inc, IncS, Not, Pop};
        match instr {
            Inc => Self::Inc { reg, signed: false },
            IncS => Self::Inc { reg, signed: true },
            Dec => Self::Dec { reg, signed: false },
            DecS => Self::Dec { reg, signed: true },
            Not => Self::Not { reg },
            Pop => Self::Pop { to: reg },
        }
    }

    pub(crate) const fn from_single_operand_instruction(
        instr: ASMSingleOperandInstruction,
        operand: Operand<W>,
    ) -> Self {
        use ASMSingleOperandInstruction::{Call, Push};

        match instr {
            Call => Self::Call { addr: operand },
            Push => Self::Push { from: operand },
        }
    }

    pub(crate) const fn from_two_operand_instruction(
        instr: ASMTwoOperandInstruction,
        lhs: Operand<W>,
        rhs: Operand<W>,
    ) -> Self {
        use ASMTwoOperandInstruction::Cmp;

        match instr {
            Cmp => Self::Cmp { lhs, rhs },
        }
    }

    pub(crate) const fn from_shift_instruction(instr: ASMShiftInstruction, reg: Register, val: W) -> Self {
        use ASMShiftInstruction::{Shl, Shr};

        match instr {
            Shl => Self::Shl { reg, val },
            Shr => Self::Shr { reg, val },
        }
    }

    pub(crate) const fn from_rotate_instruction(instr: ASMRotateInstruction, reg: Register, val: u32) -> Self {
        use ASMRotateInstruction::{Rol, Ror};

        match instr {
            Ror => Self::Ror { reg, val },
            Rol => Self::Rol { reg, val },
        }
    }

    pub(crate) const fn from_jump_instruction(instr: ASMJumpInstruction, dest: W) -> Self {
        use ASMJumpInstruction::{Jc, Jg, Jge, Jl, Jle, Jmp, Jnc, Jns, Jnz, Js, Jz};
        let condition = match instr {
            Jmp => JumpCondition::Unconditional,
            Jz => JumpCondition::Zero,
            Jnz => JumpCondition::NotZero,
            Jc => JumpCondition::Carry,
            Jnc => JumpCondition::NotCarry,
            Js => JumpCondition::Signed,
            Jns => JumpCondition::NotSigned,
            Jg => JumpCondition::Greater,
            Jl => JumpCondition::Less,
            Jge => JumpCondition::GreaterOrEq,
            Jle => JumpCondition::LessOrEq,
        };

        Self::Jump { to: dest, condition }
    }

    /// Copy a value from an operand to a register.
    #[inline]
    fn mov<const MEM_SIZE: usize, Insts, Words>(
        to: Register,
        from: Operand<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        processor.registers.set_reg(to, from.resolve(processor));
    }

    /// Store a value from a register into a memory location.
    #[inline]
    fn str<const MEM_SIZE: usize, Insts, Words>(
        from: Register,
        to: MemoryLocation<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let to_addr = to.resolve(processor);
        let val = processor.registers.get_reg(from);
        processor.mem.write(to_addr, val);
    }

    /// Load a value from a memory location into a register.
    #[inline]
    fn ldr<const MEM_SIZE: usize, Insts, Words>(
        to: Register,
        from: MemoryLocation<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let from_addr = from.resolve(processor);
        let val = processor.mem.read(from_addr);
        processor.registers.set_reg(to, val);
    }

    /// Push a value from the operand to the stack.
    #[inline]
    fn push<const MEM_SIZE: usize, Insts, Words>(
        from: Operand<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        processor.registers.dec(Register::SP);
        let sp = processor.registers.sp();

        processor.mem.write(sp, from.resolve(processor));
    }

    /// Pop a value from the stack to the register.
    #[inline]
    fn pop<const MEM_SIZE: usize, Insts, Words>(
        to: Register,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let sp = processor.registers.sp();
        let val = processor.mem.read(sp);

        processor.registers.inc(Register::SP);
        processor.registers.set_reg(to, val);
    }

    /// Call a subroutine at the program address specified by the operand.
    /// Pushes the current program counter onto the stack and sets the program counter to the address of the subroutine.
    #[inline]
    fn call<const MEM_SIZE: usize, Insts, Words>(
        addr: Operand<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        Self::push(Operand::Value(processor.registers.pc()), processor);
        processor.registers.set_reg(Register::PC, addr.resolve(processor));
    }

    /// Return from a subroutine.
    /// Pops the return address from the stack and sets the program counter to the popped value.
    #[inline]
    fn ret<const MEM_SIZE: usize, Insts, Words>(processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>) {
        Self::pop(Register::PC, processor);
    }

    /// Set program pointer to value, effectively jumping to the instruction at this point in the program.
    /// The condition is checked before jumping and the jump is performed if the condition is met.
    #[inline]
    fn jmp<const MEM_SIZE: usize, Insts, Words>(
        to: W,
        condition: JumpCondition,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        if condition.check(processor) {
            processor.registers.set_reg(Register::PC, to);
        }
    }

    /// Add the value of an operand (rhs) to a register (acc).
    #[inline]
    fn add<const MEM_SIZE: usize, Insts, Words>(
        acc: Register,
        rhs: Operand<W>,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        if signed {
            let (result, overflow) = a.overflowing_add(b);
            let carry = a.check_carry_add(b);

            processor.registers.set_reg(acc, result);
            processor.registers.set_flag(Flag::V, overflow);
            processor.registers.set_flag(Flag::C, carry);

            Self::set_signed_zero_flags(result, processor);
        } else {
            processor.registers.set_reg(acc, a + b);
        }
    }

    /// Subtract the value of an operand (rhs) from a register (acc).
    #[inline]
    fn sub<const MEM_SIZE: usize, Insts, Words>(
        acc: Register,
        rhs: Operand<W>,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        if signed {
            let (result, overflow) = a.overflowing_sub(b);
            let carry = a.check_carry_sub(b);

            processor.registers.set_reg(acc, result);
            processor.registers.set_flag(Flag::V, overflow);
            processor.registers.set_flag(Flag::C, carry);

            Self::set_signed_zero_flags(result, processor);
        } else {
            processor.registers.set_reg(acc, a - b);
        }
    }

    /// Multiply the value of an operand (acc) with the value of a register (rhs).
    /// The result is stored in acc.
    #[inline]
    fn mul<const MEM_SIZE: usize, Insts, Words>(
        acc: Register,
        rhs: Operand<W>,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        if signed {
            let (result, overflow) = a.overflowing_mul(b);
            let carry = a.check_carry_mul(b);

            processor.registers.set_reg(acc, result);
            processor.registers.set_flag(Flag::V, overflow);
            processor.registers.set_flag(Flag::C, carry);

            Self::set_signed_zero_flags(result, processor);
        } else {
            processor.registers.set_reg(acc, a * b);
        }
    }

    /// Divide the value of an operand (acc) by the value of a register (rhs).
    /// The result is stored in acc.
    #[inline]
    fn div<const MEM_SIZE: usize, Insts, Words>(
        acc: Register,
        rhs: Operand<W>,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        if signed {
            let (result, overflow) = a.overflowing_div(b);
            let carry = overflow; // this is the same as a.carry_div(b)

            processor.registers.set_reg(acc, result);
            processor.registers.set_flag(Flag::V, overflow);
            processor.registers.set_flag(Flag::C, carry);

            Self::set_signed_zero_flags(result, processor);
        } else {
            processor.registers.set_reg(acc, a / b);
        }
    }

    /// Increment the value in a register by one.
    #[inline]
    fn inc<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        if signed {
            Self::add(reg, Operand::Value(1.into()), true, processor);
        } else {
            processor.registers.inc(reg);
        }
    }

    /// Decrement the value in a register by one.
    #[inline]
    fn dec<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        if signed {
            Self::sub(reg, Operand::Value(1.into()), true, processor);
        } else {
            processor.registers.dec(reg);
        }
    }

    /// Sets the signed and zero flags.
    #[inline]
    fn set_signed_zero_flags<const MEM_SIZE: usize, Insts, Words>(
        val: W,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        match val.cmp(&(0.into())) {
            Ordering::Less => {
                processor.registers.set_flag(Flag::S, true);
                processor.registers.set_flag(Flag::Z, false);
            }
            Ordering::Equal => {
                processor.registers.set_flag(Flag::S, false);
                processor.registers.set_flag(Flag::Z, true);
            }
            Ordering::Greater => {
                processor.registers.set_flag(Flag::S, false);
                processor.registers.set_flag(Flag::Z, false);
            }
        }
    }

    /// Compares two operands and sets the flags accordingly.
    #[inline]
    fn cmp<const MEM_SIZE: usize, Insts, Words>(
        lhs: Operand<W>,
        rhs: Operand<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = lhs.resolve(processor);
        let b = rhs.resolve(processor);

        let (result, overflow) = a.overflowing_sub(b);
        let carry = a.check_carry_sub(b);

        processor.registers.set_flag(Flag::V, overflow);
        processor.registers.set_flag(Flag::C, carry);
        Self::set_signed_zero_flags(result, processor);
    }

    /// Perform an xor operation on the value in the register with the value of the operand. (XOR)
    #[inline]
    fn xor<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        rhs: Operand<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(reg);
        let b = rhs.resolve(processor);

        processor.registers.set_reg(reg, a ^ b);
    }

    /// Perform an and operation on the value in the register with the value of the operand. (AND)
    #[inline]
    fn and<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        rhs: Operand<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(reg);
        let b = rhs.resolve(processor);

        processor.registers.set_reg(reg, a & b);
    }

    /// Perform an or operation on the value in the register with the value of the operand. (OR)
    #[inline]
    fn or<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        rhs: Operand<W>,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(reg);
        let b = rhs.resolve(processor);

        processor.registers.set_reg(reg, a | b);
    }

    /// Perform a not operation on the value in the register. (NOT)
    #[inline]
    fn not<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(reg);

        processor.registers.set_reg(reg, !a);
    }

    /// Shift the value in the register left by the specified number of bits.
    #[inline]
    fn shl<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        val: W,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a << val);
    }

    /// Shift the value in the register right by the specified number of bits.
    #[inline]
    fn shr<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        val: W,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a >> val);
    }

    /// Rotate the value in the register left by the specified number of bits.
    #[inline]
    fn rol<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        val: u32,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a.rotate_left(val));
    }

    /// Rotate the value in the register right by the specified number of bits.
    #[inline]
    fn ror<const MEM_SIZE: usize, Insts, Words>(
        reg: Register,
        val: u32,
        processor: &mut Processor<MEM_SIZE, Self, Insts, W, Words>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a.rotate_right(val));
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use procem::word::*;

    const MEM_SIZE: usize = 32;
    type IS = Instruction<W>;
    type P = Vec<IS>;
    type W = I8;
    type Words = Vec<W>;

    mod mov {
        use super::*;

        #[test]
        fn test_move_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 10isize.into());
            let _ = IS::execute(
                Instruction::Mov {
                    from: Operand::Register(Register::R0),
                    to: Register::R1,
                },
                &mut processor,
            );
            assert_eq!(
                processor.registers.get_reg(Register::R1),
                processor.registers.get_reg(Register::R0)
            );
        }

        #[test]
        fn test_move_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            let _ = IS::execute(
                Instruction::Mov {
                    to: Register::R0,
                    from: Operand::Value(10isize.into()),
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 10isize.into());
        }
    }

    mod ldr_str {
        use super::*;

        #[test]
        fn test_str_direct_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 42i8.into());

            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Direct(0i8.into()),
                },
                &mut processor,
            );

            assert_eq!(processor.mem.read(0i8.into()), I8::from(42i8));
        }

        #[test]
        fn test_str_indirect_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 42i8.into());
            processor.registers.set_reg(Register::R1, 1i8.into());

            // Positive value offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Indirect {
                        base: Register::R1,
                        offset: Operand::Value(0i8.into()),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(1i8.into()), I8::from(42i8));

            // Negative value offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Indirect {
                        base: Register::R1,
                        offset: Operand::Value((-1i8).into()),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(0i8.into()), I8::from(42i8));

            // Register offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Indirect {
                        base: Register::R1,
                        offset: Operand::Register(Register::R1),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(2i8.into()), I8::from(42i8));
        }

        #[test]
        #[should_panic]
        fn test_str_invalid_memory_location_panics() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();

            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Direct((MEM_SIZE as i8).into()),
                },
                &mut processor,
            );

            unreachable!("Panic should happen before")
        }

        #[test]
        fn test_ldr_direct_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.mem.write(0i8.into(), 42i8.into());

            let _ = IS::execute(
                Instruction::Ldr {
                    to: Register::R0,
                    from: MemoryLocation::Direct(0i8.into()),
                },
                &mut processor,
            );

            assert_eq!(processor.registers.get_reg(Register::R0), I8::from(42i8));
        }

        #[test]
        fn test_ldr_indirect_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R1, 1i8.into());
            processor.mem.write(0i8.into(), 42i8.into());
            processor.mem.write(1i8.into(), 43i8.into());
            processor.mem.write(2i8.into(), 44i8.into());

            // Positive value offset
            let _ = IS::execute(
                Instruction::Ldr {
                    to: Register::R0,
                    from: MemoryLocation::Indirect {
                        base: Register::R1,
                        offset: Operand::Value(0i8.into()),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), I8::from(43i8));

            // Negative value offset
            let _ = IS::execute(
                Instruction::Ldr {
                    to: Register::R0,
                    from: MemoryLocation::Indirect {
                        base: Register::R1,
                        offset: Operand::Value((-1i8).into()),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), I8::from(42i8));

            // Register offset
            let _ = IS::execute(
                Instruction::Ldr {
                    to: Register::R0,
                    from: MemoryLocation::Indirect {
                        base: Register::R1,
                        offset: Operand::Register(Register::R1),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), I8::from(44i8));
        }

        #[test]
        #[should_panic]
        fn test_ldr_invalid_memory_location_panics() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();

            let _ = IS::execute(
                Instruction::Ldr {
                    from: MemoryLocation::Direct((MEM_SIZE as i8).into()),
                    to: Register::R0,
                },
                &mut processor,
            );

            unreachable!("Panic should happen before")
        }
    }

    mod inc {
        use super::*;

        #[test]
        fn test_inc() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 10isize.into());
            let _ = IS::execute(
                Instruction::Inc {
                    reg: Register::R0,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 11isize.into());
        }

        #[test]
        fn test_inc_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, i8::MAX.into());
            let _ = IS::execute(
                Instruction::Inc {
                    reg: Register::R0,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), i8::MIN.into());
        }
    }

    mod dec {
        use super::*;

        #[test]
        fn test_dec() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 10isize.into());
            let _ = IS::execute(
                Instruction::Dec {
                    reg: Register::R0,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 9isize.into());
        }

        #[test]
        fn test_dec_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, i8::MIN.into());
            let _ = IS::execute(
                Instruction::Dec {
                    reg: Register::R0,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), i8::MAX.into());
        }
    }

    mod add {
        use super::*;

        #[test]
        fn test_add_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 5isize.into());
            processor.registers.set_reg(Register::R1, 10isize.into());
            let _ = IS::execute(
                Instruction::Add {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 15isize.into());
        }

        #[test]
        fn test_add_reg_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, i8::MAX.into());
            processor.registers.set_reg(Register::R1, 1isize.into());
            let _ = IS::execute(
                Instruction::Add {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), i8::MIN.into());
        }

        #[test]
        fn test_add_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 5isize.into());
            let _ = IS::execute(
                Instruction::Add {
                    acc: Register::R0,
                    rhs: Operand::Value(10isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 15isize.into());
        }

        #[test]
        fn test_add_val_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, i8::MAX.into());
            let _ = IS::execute(
                Instruction::Add {
                    acc: Register::R0,
                    rhs: Operand::Value(1isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), i8::MIN.into());
        }
    }

    mod sub {
        use super::*;

        #[test]
        fn test_sub_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 5isize.into());
            processor.registers.set_reg(Register::R1, 10isize.into());
            let _ = IS::execute(
                Instruction::Sub {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (-5isize).into());
        }

        #[test]
        fn test_sub_reg_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, i8::MIN.into());
            processor.registers.set_reg(Register::R1, 1isize.into());
            let _ = IS::execute(
                Instruction::Sub {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), i8::MAX.into());
        }

        #[test]
        fn test_sub_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 5isize.into());
            let _ = IS::execute(
                Instruction::Sub {
                    acc: Register::R0,
                    rhs: Operand::Value(10isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (-5isize).into());
        }

        #[test]
        fn test_sub_val_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, (-128isize).into());
            let _ = IS::execute(
                Instruction::Sub {
                    acc: Register::R0,
                    rhs: Operand::Value(1isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 127isize.into());
        }
    }

    mod mul {
        use super::*;

        #[test]
        fn test_mul_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 5isize.into());
            processor.registers.set_reg(Register::R1, 10isize.into());
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 50isize.into());

            processor.registers.set_reg(Register::R0, (-5isize).into());
            processor.registers.set_reg(Register::R1, 10isize.into());
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (-50isize).into());
        }

        #[test]
        fn test_mul_reg_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 80isize.into());
            processor.registers.set_reg(Register::R1, 2isize.into());
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (-96isize).into());
        }

        #[test]
        fn test_mul_reg_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, (-80isize).into());
            processor.registers.set_reg(Register::R1, 2isize.into());
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 96isize.into());
        }

        #[test]
        fn test_mul_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 5isize.into());
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Value(10isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 50isize.into());

            processor.registers.set_reg(Register::R0, (-5isize).into());
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Value(10isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (-50isize).into());
        }

        #[test]
        fn test_mul_val_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 80isize.into());
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Value(2isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (-96isize).into());
        }

        #[test]
        fn test_mul_val_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, (-80isize).into());
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Value(2isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 96isize.into());
        }
    }

    mod div {
        use super::*;

        #[test]
        fn test_div_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 10isize.into());
            processor.registers.set_reg(Register::R1, 5isize.into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 2isize.into());

            processor.registers.set_reg(Register::R0, (-10isize).into());
            processor.registers.set_reg(Register::R1, 5isize.into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (-2isize).into());
        }

        #[test]
        fn test_div_reg_truncate() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 3isize.into());
            processor.registers.set_reg(Register::R1, 2isize.into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 1isize.into());
        }

        #[test]
        fn test_div_reg_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, i8::MIN.into());
            processor.registers.set_reg(Register::R1, (-1isize).into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (i8::MIN).into());
        }

        #[test]
        fn test_div_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 10isize.into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(5isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 2isize.into());

            processor.registers.set_reg(Register::R0, (-10isize).into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(5isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (-2isize).into());
        }

        #[test]
        fn test_div_val_truncate() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, 3isize.into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(4isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 0isize.into());

            processor.registers.set_reg(Register::R0, 3isize.into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(2isize.into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 1isize.into());
        }

        #[test]
        fn test_div_val_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            processor.registers.set_reg(Register::R0, i8::MIN.into());
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value((-1isize).into()),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), (i8::MIN).into());
        }
    }

    mod jmp {
        use super::*;

        #[test]
        fn test_jmp() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0isize.into());
            let _ = IS::execute(
                Instruction::Jump {
                    to: 2isize.into(),
                    condition: JumpCondition::Unconditional,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), 2isize.into());
        }

        #[test]
        fn test_jmp_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0isize.into());
            let _ = IS::execute(
                Instruction::Jump {
                    to: i8::MAX.into(),
                    condition: JumpCondition::Unconditional,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), i8::MAX.into());
            let _ = IS::execute(
                Instruction::Inc {
                    reg: Register::PC,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), i8::MIN.into());
        }

        #[test]
        fn test_jmp_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0isize.into());
            let _ = IS::execute(
                Instruction::Jump {
                    to: i8::MIN.into(),
                    condition: JumpCondition::Unconditional,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), i8::MIN.into());
            let _ = IS::execute(
                Instruction::Dec {
                    reg: Register::PC,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), i8::MAX.into());
        }
    }

    mod cmp {
        use super::*;

        #[test]
        fn test_cmp_eq_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();

            processor.registers.set_reg(Register::R0, 1isize.into());
            processor.registers.set_reg(Register::R1, 1isize.into());

            let _ = IS::execute(
                Instruction::Cmp {
                    lhs: Operand::Register(Register::R0),
                    rhs: Operand::Register(Register::R1),
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_flag(Flag::C), false);
            assert_eq!(processor.registers.get_flag(Flag::S), false);
            assert_eq!(processor.registers.get_flag(Flag::V), false);
            assert_eq!(processor.registers.get_flag(Flag::Z), true);
        }

        #[test]
        fn test_cmp_eq_reg_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();

            processor.registers.set_reg(Register::R0, 1isize.into());

            let _ = IS::execute(
                Instruction::Cmp {
                    lhs: Operand::Register(Register::R0),
                    rhs: Operand::Value(1isize.into()),
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_flag(Flag::C), false);
            assert_eq!(processor.registers.get_flag(Flag::S), false);
            assert_eq!(processor.registers.get_flag(Flag::V), false);
            assert_eq!(processor.registers.get_flag(Flag::Z), true);
        }

        #[test]
        fn test_cmp_eq_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();

            let _ = IS::execute(
                Instruction::Cmp {
                    lhs: Operand::Value(1isize.into()),
                    rhs: Operand::Value(1isize.into()),
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_flag(Flag::C), false);
            assert_eq!(processor.registers.get_flag(Flag::S), false);
            assert_eq!(processor.registers.get_flag(Flag::V), false);
            assert_eq!(processor.registers.get_flag(Flag::Z), true);
        }

        #[test]
        fn test_cmp_less() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();

            processor.registers.set_reg(Register::R0, 1isize.into());
            processor.registers.set_reg(Register::R1, 2isize.into());

            let _ = IS::execute(
                Instruction::Cmp {
                    lhs: Operand::Register(Register::R0),
                    rhs: Operand::Register(Register::R1),
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_flag(Flag::C), true);
            assert_eq!(processor.registers.get_flag(Flag::S), true);
            assert_eq!(processor.registers.get_flag(Flag::V), false);
            assert_eq!(processor.registers.get_flag(Flag::Z), false);
        }

        #[test]
        fn test_cmp_greater() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();

            processor.registers.set_reg(Register::R0, 2isize.into());
            processor.registers.set_reg(Register::R1, 1isize.into());

            let _ = IS::execute(
                Instruction::Cmp {
                    lhs: Operand::Register(Register::R0),
                    rhs: Operand::Register(Register::R1),
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_flag(Flag::C), false);
            assert_eq!(processor.registers.get_flag(Flag::S), false);
            assert_eq!(processor.registers.get_flag(Flag::V), false);
            assert_eq!(processor.registers.get_flag(Flag::Z), false);
        }
    }

    mod stack {
        use super::*;
        use procem::processor::Processor;

        #[test]
        fn test_push_pop() {
            let mut processor = Processor::<MEM_SIZE, IS, P, W, Words>::new();

            processor
                .registers
                .set_reg(Register::SP, W::try_from(MEM_SIZE - 1).unwrap());
            processor.registers.set_reg(Register::R0, 1isize.into());

            [
                Instruction::Push {
                    from: Operand::Register(Register::R0),
                },
                Instruction::Push {
                    from: Operand::Register(Register::R0),
                },
                Instruction::Pop { to: Register::R1 },
                Instruction::Push {
                    from: Operand::Register(Register::R0),
                },
                Instruction::Pop { to: Register::R2 },
                Instruction::Pop { to: Register::R3 },
            ]
            .iter()
            .for_each(|&inst| IS::execute(inst, &mut processor));

            assert_eq!(processor.registers.get_reg(Register::R1), W::from(1isize));
            assert_eq!(processor.registers.get_reg(Register::R2), W::from(1isize));
            assert_eq!(processor.registers.get_reg(Register::R3), W::from(1isize));
            assert_eq!(processor.registers.sp(), W::try_from(MEM_SIZE - 1).unwrap());
        }
    }
}
