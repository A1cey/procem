pub(crate) mod asm_instruction;
pub mod jump_condition;
pub mod memory_location;
pub mod operand;
pub mod unlinked;

use core::cmp::Ordering;
use procem::{
    instruction::Instruction as InstructionTrait,
    processor::Processor,
    register::{Flag, Register},
};

use crate::instruction::{
    asm_instruction::{
        ASMJumpInstruction, ASMRegOperandInstruction, ASMRotateInstruction, ASMShiftInstruction,
        ASMSingleOperandInstruction, ASMSingleRegInstruction, ASMTwoOperandInstruction,
    },
    jump_condition::JumpCondition,
    memory_location::MemoryLocation,
    operand::Operand,
};

/// A default instruction set implementation, that can be used for the [procem](../../procem/index.html) crate.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum Instruction {
    /// No operation. (NOP)
    Nop,
    /// Store the memory location of a label in a register. (ADR)
    Adr { reg: Register, addr: usize },
    /// Copy a value from the operand to the register. (MOV)
    Mov { to: Register, from: Operand },
    /// Store a value from a register to a memory location. (STR)
    Str { from: Register, to: MemoryLocation },
    /// Load a value from a memory location into a register. (LDR)
    Ldr { to: Register, from: MemoryLocation },
    /// Push a value from the operand to the stack. (PUSH)
    Push { from: Operand },
    /// Pop a value from the stack to the register. (POP)
    Pop { to: Register },
    /// Call a subroutine at the program address specified by the operand.
    /// Pushes the current program counter onto the stack and sets the program counter to the address of the subroutine. (CALL)
    Call { addr: Operand },
    /// Return from a subroutine.
    /// Pops the return address from the stack and sets the program counter to the popped value. (RET)
    Ret,
    /// Add the value of the operand (rhs) to the register (acc).
    /// The result is stored in acc. (ADD\[S\])
    Add { acc: Register, rhs: Operand, signed: bool },
    /// Subtract the value of the operand (rhs) from the register (acc).
    /// The result is stored in acc. (SUB\[S\])
    Sub { acc: Register, rhs: Operand, signed: bool },
    /// Multiply the value of the operand (rhs) with the value of the register (acc).
    /// The result is stored in acc. (MUL\[S\])
    Mul { acc: Register, rhs: Operand, signed: bool },
    /// Divide the value of the register (acc) by the value of the operand (rhs).
    /// The result is stored in acc. (DIV\[S\])
    Div { acc: Register, rhs: Operand, signed: bool },
    /// Increment the value in a register by one. (INC\[S\])
    Inc { reg: Register, signed: bool },
    /// Decrement the value in a register by one. (DEC\[S\])
    Dec { reg: Register, signed: bool },
    /// Set program counter to a value, effectively jumping to the instruction at this point in the program.
    /// The condition is checked before jumping and the jump is performed if the condition is met.
    /// See the assembly instruction at `JumpCondition`.
    Jump { to: usize, condition: JumpCondition },
    /// Compare the values of two operands and set the flags accordingly. This is the same as `SUBS` but disregards the result of the subtraction. (CMP)
    Cmp { lhs: Operand, rhs: Operand },
    /// Perform an xor operation on the value in the register with the value of the operand. (XOR)
    Xor { reg: Register, rhs: Operand },
    /// Perform an and operation on the value in the register with the value of the operand. (AND)
    And { reg: Register, rhs: Operand },
    /// Perform an or operation on the value in the register with the value of the operand. (OR)
    Or { reg: Register, rhs: Operand },
    /// Perform a not operation on the value in the register. (NOT)
    Not { reg: Register },
    /// Shift the value in the register left by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Shl { reg: Register, val: usize },
    /// Shift the value in the register right by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Shr { reg: Register, val: usize },
    /// Rotate the value in the register left by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Rol { reg: Register, val: u32 },
    /// Rotate the value in the register right by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Ror { reg: Register, val: u32 },
}

impl InstructionTrait for Instruction {
    /// Execute an instruction on a processor.
    fn execute<const MEM_SIZE: usize, Insts, Bytes>(
        instruction: Self,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        match instruction {
            Self::Nop => (),
            Self::Adr { reg, addr } => Self::adr(reg, addr, processor),
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

impl Instruction {
    // skips forrmatting the match
    #[rustfmt::skip]
    pub(crate) const fn from_reg_operand_instruction(
        instr: ASMRegOperandInstruction,
        lhs: Register,
        rhs: Operand
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

    pub(crate) const fn from_single_operand_instruction(instr: ASMSingleOperandInstruction, operand: Operand) -> Self {
        use ASMSingleOperandInstruction::{Call, Push};

        match instr {
            Call => Self::Call { addr: operand },
            Push => Self::Push { from: operand },
        }
    }

    pub(crate) const fn from_two_operand_instruction(
        instr: ASMTwoOperandInstruction,
        lhs: Operand,
        rhs: Operand,
    ) -> Self {
        use ASMTwoOperandInstruction::Cmp;

        match instr {
            Cmp => Self::Cmp { lhs, rhs },
        }
    }

    pub(crate) const fn from_shift_instruction(instr: ASMShiftInstruction, reg: Register, val: usize) -> Self {
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

    pub(crate) const fn from_jump_instruction(instr: ASMJumpInstruction, dest: usize) -> Self {
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

    /// Move a memory address into a register.
    #[inline]
    fn adr<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        addr: usize,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        processor.registers.set_reg(reg, addr);
    }

    /// Copy a value from an operand to a register.
    #[inline]
    fn mov<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        from: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        processor.registers.set_reg(to, from.resolve(processor));
    }

    /// Store a value from a register into a memory location.
    #[inline]
    fn str<const MEM_SIZE: usize, Insts, Bytes>(
        from: Register,
        to: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let to_addr = to.resolve(processor);
        let val = processor.registers.get_reg(from);
        processor.mem.write(to_addr, val as u8); // TODO: Writes from reg to memory how to handle? usize -> u8
    }

    /// Load a value from a memory location into a register.
    #[inline]
    fn ldr<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        from: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let from_addr = from.resolve(processor);
        let val = processor.mem.read(from_addr);
        processor.registers.set_reg(to, usize::from(val));
    }

    /// Push a value from the operand to the stack.
    #[inline]
    fn push<const MEM_SIZE: usize, Insts, Bytes>(
        from: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        processor.registers.dec(Register::SP);
        let sp = processor.registers.sp();

        processor.mem.write(sp, from.resolve(processor) as u8); // TODO: fix this conversion
    }

    /// Pop a value from the stack to the register.
    #[inline]
    fn pop<const MEM_SIZE: usize, Insts, Bytes>(to: Register, processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>) {
        let sp = processor.registers.sp();
        let val = processor.mem.read(sp);

        processor.registers.inc(Register::SP);
        processor.registers.set_reg(to, usize::from(val));
    }

    /// Call a subroutine at the program address specified by the operand.
    /// Pushes the current program counter onto the stack and sets the program counter to the address of the subroutine.
    #[inline]
    fn call<const MEM_SIZE: usize, Insts, Bytes>(
        addr: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        Self::push(Operand::Value(processor.registers.pc()), processor);
        processor.registers.set_reg(Register::PC, addr.resolve(processor));
    }

    /// Return from a subroutine.
    /// Pops the return address from the stack and sets the program counter to the popped value.
    #[inline]
    fn ret<const MEM_SIZE: usize, Insts, Bytes>(processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>) {
        Self::pop(Register::PC, processor);
    }

    /// Set program pointer to value, effectively jumping to the instruction at this point in the program.
    /// The condition is checked before jumping and the jump is performed if the condition is met.
    #[inline]
    fn jmp<const MEM_SIZE: usize, Insts, Bytes>(
        to: usize,
        condition: JumpCondition,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        if condition.check(processor) {
            processor.registers.set_reg(Register::PC, to);
        }
    }

    /// Add the value of an operand (rhs) to a register (acc).
    #[inline]
    fn add<const MEM_SIZE: usize, Insts, Bytes>(
        acc: Register,
        rhs: Operand,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        if signed {
            let (result, overflow) = a.overflowing_add(b);
            let carry = Self::check_carry_add(a, b); // TODO: Replace this function with equivalent

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
    fn sub<const MEM_SIZE: usize, Insts, Bytes>(
        acc: Register,
        rhs: Operand,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        if signed {
            let (result, overflow) = a.overflowing_sub(b);
            let carry = Self::check_carry_sub(a, b);

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
    fn mul<const MEM_SIZE: usize, Insts, Bytes>(
        acc: Register,
        rhs: Operand,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        if signed {
            let (result, overflow) = a.overflowing_mul(b);
            let carry = Self::check_carry_mul(a, b);

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
    fn div<const MEM_SIZE: usize, Insts, Bytes>(
        acc: Register,
        rhs: Operand,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
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
    fn inc<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        if signed {
            Self::add(reg, Operand::Value(1), true, processor);
        } else {
            processor.registers.inc(reg);
        }
    }

    /// Decrement the value in a register by one.
    #[inline]
    fn dec<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        signed: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        if signed {
            Self::sub(reg, Operand::Value(1), true, processor);
        } else {
            processor.registers.dec(reg);
        }
    }

    /// Sets the signed and zero flags.
    #[inline]
    fn set_signed_zero_flags<const MEM_SIZE: usize, Insts, Bytes>(
        val: usize,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        match val.cmp(&0) {
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

    //  TODO: lhs and rhs entirely depend on the word size, i think...
    fn check_carry_add(lhs: usize, rhs: usize) -> bool {
        let (lhs, rhs) = (lhs as u128, rhs as u128);
        lhs + rhs > usize::MAX as u128 // TODO: usize here depends on the word size used: byte --> u8, word --> u32, dword --> u64
    }

    // TODO: see above
    fn check_carry_sub(lhs: usize, rhs: usize) -> bool {
        let (lhs, rhs) = (lhs as u128, rhs as u128);
        lhs < rhs
    }

    // TODO: see above
    fn check_carry_mul(lhs: usize, rhs: usize) -> bool {
        let (lhs, rhs) = (lhs as u128, rhs as u128);
        lhs * rhs > usize::MAX as u128 // TODO: see above
    }

    // TODO: see above
    fn check_carry_div(lhs: usize, rhs: usize) -> bool {
        usize::overflowing_div(lhs, rhs).1 // TODO: see above
    }

    /// Compares two operands and sets the flags accordingly.
    #[inline]
    fn cmp<const MEM_SIZE: usize, Insts, Bytes>(
        lhs: Operand,
        rhs: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = lhs.resolve(processor);
        let b = rhs.resolve(processor);

        let (result, overflow) = a.overflowing_sub(b);
        let carry = Self::check_carry_sub(a, b);

        processor.registers.set_flag(Flag::V, overflow);
        processor.registers.set_flag(Flag::C, carry);
        Self::set_signed_zero_flags(result, processor);
    }

    /// Perform an xor operation on the value in the register with the value of the operand. (XOR)
    #[inline]
    fn xor<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        rhs: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        let b = rhs.resolve(processor);

        processor.registers.set_reg(reg, a ^ b);
    }

    /// Perform an and operation on the value in the register with the value of the operand. (AND)
    #[inline]
    fn and<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        rhs: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        let b = rhs.resolve(processor);

        processor.registers.set_reg(reg, a & b);
    }

    /// Perform an or operation on the value in the register with the value of the operand. (OR)
    #[inline]
    fn or<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        rhs: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        let b = rhs.resolve(processor);

        processor.registers.set_reg(reg, a | b);
    }

    /// Perform a not operation on the value in the register. (NOT)
    #[inline]
    fn not<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, !a);
    }

    /// Shift the value in the register left by the specified number of bits.
    #[inline]
    fn shl<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        val: usize, // TODO: Is this always the correct size?
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a << val);
    }

    /// Shift the value in the register right by the specified number of bits.
    #[inline]
    fn shr<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        val: usize, // TODO: Is this always the correct size?
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a >> val);
    }

    /// Rotate the value in the register left by the specified number of bits.
    #[inline]
    fn rol<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        val: u32,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a.rotate_left(val));
    }

    /// Rotate the value in the register right by the specified number of bits.
    #[inline]
    fn ror<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        val: u32,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a.rotate_right(val));
    }
}

#[cfg(test)]
mod test {

    use super::*;

    const MEM_SIZE: usize = 32;
    type IS = Instruction;
    type P = Vec<IS>;
    type Bytes = Vec<u8>;

    mod mov {
        use super::*;

        #[test]
        fn test_move_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 10);
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
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            let _ = IS::execute(
                Instruction::Mov {
                    to: Register::R0,
                    from: Operand::Value(10),
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 10);
        }
    }

    mod ldr_str {
        use super::*;

        #[test]
        fn test_str_direct_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 42);

            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Labeled(0),
                },
                &mut processor,
            );

            assert_eq!(processor.mem.read(0), 42);
        }

        #[test]
        fn test_str_indirect_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 42);
            processor.registers.set_reg(Register::R1, 1);

            // Positive value offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(0),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(1), 42);

            // Negative value offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(-1isize as usize),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(0), 42);

            // Register offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Register(Register::R1),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(2), 42);
        }

        #[test]
        #[should_panic]
        fn test_str_invalid_memory_location_panics() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Labeled(MEM_SIZE),
                },
                &mut processor,
            );

            unreachable!("Panic should happen before")
        }

        #[test]
        fn test_ldr_direct_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.mem.write(0, 42);

            let _ = IS::execute(
                Instruction::Ldr {
                    to: Register::R0,
                    from: MemoryLocation::Labeled(0),
                },
                &mut processor,
            );

            assert_eq!(processor.registers.get_reg(Register::R0), 42);
        }

        #[test]
        fn test_ldr_indirect_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R1, 1);
            processor.mem.write(0, 42);
            processor.mem.write(1, 43);
            processor.mem.write(2, 44);

            // Positive value offset
            let _ = IS::execute(
                Instruction::Ldr {
                    to: Register::R0,
                    from: MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(0),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 43);

            // Negative value offset
            let _ = IS::execute(
                Instruction::Ldr {
                    to: Register::R0,
                    from: MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(-1isize as usize),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 42);

            // Register offset
            let _ = IS::execute(
                Instruction::Ldr {
                    to: Register::R0,
                    from: MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Register(Register::R1),
                    },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 44);
        }

        #[test]
        #[should_panic]
        fn test_ldr_invalid_memory_location_panics() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            let _ = IS::execute(
                Instruction::Ldr {
                    from: MemoryLocation::Labeled(MEM_SIZE),
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
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 10);
            let _ = IS::execute(
                Instruction::Inc {
                    reg: Register::R0,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 11);
        }

        #[test]
        fn test_inc_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, usize::MAX);
            let _ = IS::execute(
                Instruction::Inc {
                    reg: Register::R0,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), usize::MIN);
        }
    }

    mod dec {
        use super::*;

        #[test]
        fn test_dec() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 10);
            let _ = IS::execute(
                Instruction::Dec {
                    reg: Register::R0,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 9);
        }

        #[test]
        fn test_dec_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, usize::MIN);
            let _ = IS::execute(
                Instruction::Dec {
                    reg: Register::R0,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), usize::MAX);
        }
    }

    mod add {
        use super::*;

        #[test]
        fn test_add_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 5);
            processor.registers.set_reg(Register::R1, 10);
            let _ = IS::execute(
                Instruction::Add {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 15);
        }

        #[test]
        fn test_add_reg_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, usize::MAX.into());
            processor.registers.set_reg(Register::R1, 1);
            let _ = IS::execute(
                Instruction::Add {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), usize::MIN);
        }

        #[test]
        fn test_add_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 5);
            let _ = IS::execute(
                Instruction::Add {
                    acc: Register::R0,
                    rhs: Operand::Value(10),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 15);
        }

        #[test]
        fn test_add_val_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, usize::MAX);
            let _ = IS::execute(
                Instruction::Add {
                    acc: Register::R0,
                    rhs: Operand::Value(1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), usize::MIN);
        }
    }

    mod sub {
        use super::*;

        #[test]
        fn test_sub_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 5);
            processor.registers.set_reg(Register::R1, 10);
            let _ = IS::execute(
                Instruction::Sub {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), -5isize as usize);
        }

        #[test]
        fn test_sub_reg_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, usize::MIN);
            processor.registers.set_reg(Register::R1, 1);
            let _ = IS::execute(
                Instruction::Sub {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), usize::MAX);
        }

        #[test]
        fn test_sub_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 5);
            let _ = IS::execute(
                Instruction::Sub {
                    acc: Register::R0,
                    rhs: Operand::Value(10),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), -5isize as usize);
        }

        #[test]
        fn test_sub_val_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, -128isize as usize);
            let _ = IS::execute(
                Instruction::Sub {
                    acc: Register::R0,
                    rhs: Operand::Value(1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 127);
        }
    }

    mod mul {
        use super::*;

        #[test]
        fn test_mul_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 5);
            processor.registers.set_reg(Register::R1, 10);
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 50);

            processor.registers.set_reg(Register::R0, -5isize as usize);
            processor.registers.set_reg(Register::R1, 10);
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), -50isize as usize);
        }

        #[test]
        fn test_mul_reg_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 80);
            processor.registers.set_reg(Register::R1, 2);
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), -96isize as usize);
        }

        #[test]
        fn test_mul_reg_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, -80isize as usize);
            processor.registers.set_reg(Register::R1, 2);
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 96);
        }

        #[test]
        fn test_mul_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 5);
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Value(10),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 50);

            processor.registers.set_reg(Register::R0, -5isize as usize);
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Value(10),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), -50isize as usize);
        }

        #[test]
        fn test_mul_val_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 80);
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Value(2),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), -96isize as usize);
        }

        #[test]
        fn test_mul_val_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, -80isize as usize);
            let _ = IS::execute(
                Instruction::Mul {
                    acc: Register::R0,
                    rhs: Operand::Value(2),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 96);
        }
    }

    mod div {
        use super::*;

        #[test]
        fn test_div_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 10);
            processor.registers.set_reg(Register::R1, 5);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 2);

            processor.registers.set_reg(Register::R0, -10isize as usize);
            processor.registers.set_reg(Register::R1, 5);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), -2isize as usize);
        }

        #[test]
        fn test_div_reg_truncate() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 3);
            processor.registers.set_reg(Register::R1, 2);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 1);
        }

        #[test]
        fn test_div_reg_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, usize::MIN);
            processor.registers.set_reg(Register::R1, -1isize as usize);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Register(Register::R1),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), usize::MIN);
        }

        #[test]
        fn test_div_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 10);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(5),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 2);

            processor.registers.set_reg(Register::R0, -10isize as usize);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(5),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), -2isize as usize);
        }

        #[test]
        fn test_div_val_truncate() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 3);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(4),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 0);

            processor.registers.set_reg(Register::R0, 3);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(2),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 1);
        }

        #[test]
        fn test_div_val_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, usize::MIN);
            let _ = IS::execute(
                Instruction::Div {
                    acc: Register::R0,
                    rhs: Operand::Value(-1isize as usize),
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), usize::MIN);
        }
    }

    mod jmp {
        use super::*;

        #[test]
        fn test_jmp() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0);
            let _ = IS::execute(
                Instruction::Jump {
                    to: 2,
                    condition: JumpCondition::Unconditional,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), 2);
        }

        #[test]
        fn test_jmp_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0);
            let _ = IS::execute(
                Instruction::Jump {
                    to: usize::MAX,
                    condition: JumpCondition::Unconditional,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), usize::MAX);
            let _ = IS::execute(
                Instruction::Inc {
                    reg: Register::PC,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), usize::MIN);
        }

        #[test]
        fn test_jmp_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0);
            let _ = IS::execute(
                Instruction::Jump {
                    to: usize::MIN,
                    condition: JumpCondition::Unconditional,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), usize::MIN);
            let _ = IS::execute(
                Instruction::Dec {
                    reg: Register::PC,
                    signed: false,
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::PC), usize::MAX);
        }
    }

    mod cmp {
        use super::*;

        #[test]
        fn test_cmp_eq_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            processor.registers.set_reg(Register::R0, 1);
            processor.registers.set_reg(Register::R1, 1);

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
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            processor.registers.set_reg(Register::R0, 1);

            let _ = IS::execute(
                Instruction::Cmp {
                    lhs: Operand::Register(Register::R0),
                    rhs: Operand::Value(1),
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
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            let _ = IS::execute(
                Instruction::Cmp {
                    lhs: Operand::Value(1),
                    rhs: Operand::Value(1),
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
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            processor.registers.set_reg(Register::R0, 1);
            processor.registers.set_reg(Register::R1, 2);

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
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            processor.registers.set_reg(Register::R0, 2);
            processor.registers.set_reg(Register::R1, 1);

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
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            processor.registers.set_reg(Register::SP, MEM_SIZE - 1);
            processor.registers.set_reg(Register::R0, 1);

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

            assert_eq!(processor.registers.get_reg(Register::R1), 1);
            assert_eq!(processor.registers.get_reg(Register::R2), 1);
            assert_eq!(processor.registers.get_reg(Register::R3), 1);
            assert_eq!(processor.registers.sp(), MEM_SIZE - 1);
        }
    }
}
