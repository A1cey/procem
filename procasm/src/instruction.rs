mod directive;
mod jump_condition;
mod memory_location;
pub(crate) mod mnemonics;
mod operand;
pub(crate) mod unlinked;

pub use directive::Directive;
pub use jump_condition::JumpCondition;
pub use memory_location::MemoryLocation;
pub use operand::Operand;

use core::cmp::Ordering;
use procem::{
    instruction::{Instruction as InstructionTrait, InstructionResult},
    processor::{Processor, ProcessorError},
    register::{Flag, Register},
};

use crate::instruction::mnemonics::{
    JumpMnemonic, LoadOrStoreMnemonic, RegOperandMnemonic, RotateMnemonic, ShiftMnemonic, SingleOperandMnemonic,
    SingleRegMnemonic, TwoOperandMnemonic,
};

/// A default instruction set implementation, that can be used for the [procem](../../procem/index.html) crate.
#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum Instruction {
    /// No operation. (NOP)
    Nop,
    /// Store the memory location of a label in a register. (ADR)
    Adr { reg: Register, addr: u64 },
    /// Copy a value from the operand to the register. (MOV)
    Mov { to: Register, from: Operand },
    /// Store an 8-bit value from a register to a memory location. (STRB)
    Strb { from: Register, to: MemoryLocation },
    /// Store a 16-bit value from a register to a memory location. (STRH)
    Strh { from: Register, to: MemoryLocation },
    /// Store a 32-bit value from a register to a memory location. (STRW)
    Strw { from: Register, to: MemoryLocation },
    /// Store a 64-bit value from a register to a memory location. (STRD)
    Strd { from: Register, to: MemoryLocation },
    /// Store a 128-bit value from a register to a memory location. (STRQ)
    Strq { from: Register, to: MemoryLocation },
    /// Store a 64-bit value from a register to a memory location. (STR)
    Str { from: Register, to: MemoryLocation },
    /// Load an 8-bit value from a memory location into a register. (LDR)
    Ldrb { to: Register, from: MemoryLocation },
    /// Load a 16-bit value from a memory location into a register. (LDR)
    Ldrh { to: Register, from: MemoryLocation },
    /// Load a 32-bit value from a memory location into a register. (LDR)
    Ldrw { to: Register, from: MemoryLocation },
    /// Load a 64-bit value from a memory location into a register. (LDR)
    Ldrd { to: Register, from: MemoryLocation },
    /// Load a 128-bit value from a memory location into a register. (LDR)
    Ldrq { to: Register, from: MemoryLocation },
    /// Load a 64-bit value from a memory location into a register. (LDR)
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
    Add { acc: Register, rhs: Operand, set_flags: bool },
    /// Subtract the value of the operand (rhs) from the register (acc).
    /// The result is stored in acc. (SUB\[S\])
    Sub { acc: Register, rhs: Operand, set_flags: bool },
    /// Multiply the value of the operand (rhs) with the value of the register (acc).
    /// The result is stored in acc. (MUL\[S\])
    Mul { acc: Register, rhs: Operand, set_flags: bool },
    /// Unsigned divide the value of the register (acc) by the value of the operand (rhs).
    /// The result is stored in acc. (DIV\[S\])
    Div { acc: Register, rhs: Operand, set_flags: bool },
    /// Signed divide the value of the register (acc) by the value of the operand (rhs).
    /// The result is stored in acc. (SDIV\[S\])
    Sdiv { acc: Register, rhs: Operand, set_flags: bool },
    /// Increment the value in a register by one. (INC\[S\])
    Inc { reg: Register, set_flags: bool },
    /// Decrement the value in a register by one. (DEC\[S\])
    Dec { reg: Register, set_flags: bool },
    /// Set program counter to a value, effectively jumping to the instruction at this point in the program.
    /// The condition is checked before jumping and the jump is performed if the condition is met.
    /// See the assembly instruction at `JumpCondition`.
    Jump { to: u64, condition: JumpCondition },
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
    Shl { reg: Register, val: u32 },
    /// Shift the value in the register right by the specified number of bits.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Shr { reg: Register, val: u32 },
    /// Arithmetic shift the value in the register right by the specified number of bits, preserving the sign bit.
    /// The assembler only accepts values between 1 and the number of bits of the Word size minus 1.
    Asr { reg: Register, val: u32 },
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
    ) -> InstructionResult {
        match instruction {
            Self::Nop => (),
            Self::Adr { reg, addr } => Self::adr(reg, addr, processor),
            Self::Mov { to, from } => Self::mov(to, from, processor),
            Self::Strb { from, to } => Self::strb(from, to, processor)?,
            Self::Strh { from, to } => Self::strh(from, to, processor)?,
            Self::Strw { from, to } => Self::strw(from, to, processor)?,
            Self::Strd { from, to } => Self::strd(from, to, processor)?,
            Self::Strq { from, to } => Self::strq(from, to, processor)?,
            Self::Str { from, to } => Self::str(from, to, processor)?,
            Self::Ldrb { to, from } => Self::ldrb(to, from, processor)?,
            Self::Ldrw { to, from } => Self::ldrw(to, from, processor)?,
            Self::Ldrh { to, from } => Self::ldrh(to, from, processor)?,
            Self::Ldrd { to, from } => Self::ldrd(to, from, processor)?,
            Self::Ldrq { to, from } => Self::ldrq(to, from, processor),
            Self::Ldr { to, from } => Self::ldr(to, from, processor)?,
            Self::Push { from } => Self::push(from, processor)?,
            Self::Pop { to } => Self::pop(to, processor)?,
            Self::Call { addr } => Self::call(addr, processor)?,
            Self::Ret => Self::ret(processor)?,
            Self::Add { acc, rhs, set_flags } => Self::add(acc, rhs, set_flags, processor),
            Self::Sub { acc, rhs, set_flags } => Self::sub(acc, rhs, set_flags, processor),
            Self::Mul { acc, rhs, set_flags } => Self::mul(acc, rhs, set_flags, processor),
            Self::Div { acc, rhs, set_flags } => Self::div(acc, rhs, set_flags, processor)?,
            Self::Sdiv { acc, rhs, set_flags } => Self::sdiv(acc, rhs, set_flags, processor)?,
            Self::Inc { reg, set_flags } => Self::inc(reg, set_flags, processor),
            Self::Dec { reg, set_flags } => Self::dec(reg, set_flags, processor),
            Self::Jump { to, condition } => Self::jmp(to, condition, processor),
            Self::Cmp { lhs, rhs } => Self::cmp(lhs, rhs, processor),
            Self::Xor { reg, rhs } => Self::xor(reg, rhs, processor),
            Self::Or { reg, rhs } => Self::or(reg, rhs, processor),
            Self::And { reg, rhs } => Self::and(reg, rhs, processor),
            Self::Not { reg } => Self::not(reg, processor),
            Self::Shl { reg, val } => Self::shl(reg, val, processor),
            Self::Shr { reg, val } => Self::shr(reg, val, processor),
            Self::Asr { reg, val } => Self::asr(reg, val, processor),
            Self::Rol { reg, val } => Self::rol(reg, val, processor),
            Self::Ror { reg, val } => Self::ror(reg, val, processor),
        }

        Ok(())
    }
}

impl Instruction {
    // skips forrmatting the match
    pub(crate) const fn from_reg_operand_mnemonic(mnemonic: RegOperandMnemonic, lhs: Register, rhs: Operand) -> Self {
        match mnemonic {
            RegOperandMnemonic::Mov => Self::Mov { to: lhs, from: rhs },
            RegOperandMnemonic::Add => Self::Add { acc: lhs, rhs, set_flags: false },
            RegOperandMnemonic::AddS => Self::Add { acc: lhs, rhs, set_flags: true },
            RegOperandMnemonic::Sub => Self::Sub { acc: lhs, rhs, set_flags: false },
            RegOperandMnemonic::SubS => Self::Sub { acc: lhs, rhs, set_flags: true },
            RegOperandMnemonic::Mul => Self::Mul { acc: lhs, rhs, set_flags: false },
            RegOperandMnemonic::MulS => Self::Mul { acc: lhs, rhs, set_flags: true },
            RegOperandMnemonic::Div => Self::Div { acc: lhs, rhs, set_flags: false },
            RegOperandMnemonic::DivS => Self::Div { acc: lhs, rhs, set_flags: true },
            RegOperandMnemonic::Sdiv => Self::Sdiv { acc: lhs, rhs, set_flags: false },
            RegOperandMnemonic::SdivS => Self::Sdiv { acc: lhs, rhs, set_flags: true },
            RegOperandMnemonic::Or => Self::Or { reg: lhs, rhs },
            RegOperandMnemonic::And => Self::And { reg: lhs, rhs },
            RegOperandMnemonic::Xor => Self::Xor { reg: lhs, rhs },
        }
    }

    pub(crate) const fn from_single_reg_mnemonic(instr: SingleRegMnemonic, reg: Register) -> Self {
        match instr {
            SingleRegMnemonic::Inc => Self::Inc { reg, set_flags: false },
            SingleRegMnemonic::IncS => Self::Inc { reg, set_flags: true },
            SingleRegMnemonic::Dec => Self::Dec { reg, set_flags: false },
            SingleRegMnemonic::DecS => Self::Dec { reg, set_flags: true },
            SingleRegMnemonic::Not => Self::Not { reg },
            SingleRegMnemonic::Pop => Self::Pop { to: reg },
        }
    }

    pub(crate) const fn from_single_operand_mnemonic(instr: SingleOperandMnemonic, operand: Operand) -> Self {
        match instr {
            SingleOperandMnemonic::Call => Self::Call { addr: operand },
            SingleOperandMnemonic::Push => Self::Push { from: operand },
        }
    }

    pub(crate) const fn from_two_operand_mnemonic(instr: TwoOperandMnemonic, lhs: Operand, rhs: Operand) -> Self {
        match instr {
            TwoOperandMnemonic::Cmp => Self::Cmp { lhs, rhs },
        }
    }

    pub(crate) const fn from_shift_mnemonic(instr: ShiftMnemonic, reg: Register, val: u32) -> Self {
        match instr {
            ShiftMnemonic::Shl => Self::Shl { reg, val },
            ShiftMnemonic::Shr => Self::Shr { reg, val },
            ShiftMnemonic::Asr => Self::Asr { reg, val },
        }
    }

    pub(crate) const fn from_rotate_mnemonic(instr: RotateMnemonic, reg: Register, val: u32) -> Self {
        match instr {
            RotateMnemonic::Ror => Self::Ror { reg, val },
            RotateMnemonic::Rol => Self::Rol { reg, val },
        }
    }

    pub(crate) const fn from_jump_mnemonic(instr: JumpMnemonic, dest: u64) -> Self {
        let condition = match instr {
            JumpMnemonic::Jmp => JumpCondition::Unconditional,
            JumpMnemonic::Jz => JumpCondition::Zero,
            JumpMnemonic::Jnz => JumpCondition::NotZero,
            JumpMnemonic::Jc => JumpCondition::Carry,
            JumpMnemonic::Jnc => JumpCondition::NotCarry,
            JumpMnemonic::Js => JumpCondition::Signed,
            JumpMnemonic::Jns => JumpCondition::NotSigned,
            JumpMnemonic::Jg => JumpCondition::Greater,
            JumpMnemonic::Jl => JumpCondition::Less,
            JumpMnemonic::Jge => JumpCondition::GreaterOrEq,
            JumpMnemonic::Jle => JumpCondition::LessOrEq,
        };

        Self::Jump { to: dest, condition }
    }

    // skips forrmatting the match
    pub(crate) const fn from_ldr_or_str_mnemonic(
        instr: LoadOrStoreMnemonic,
        reg: Register,
        mem_location: MemoryLocation,
    ) -> Self {
        match instr {
            LoadOrStoreMnemonic::Ldr => Self::Ldr { to: reg, from: mem_location },
            LoadOrStoreMnemonic::Ldrb => Self::Ldrb { to: reg, from: mem_location },
            LoadOrStoreMnemonic::Ldrh => Self::Ldrh { to: reg, from: mem_location },
            LoadOrStoreMnemonic::Ldrw => Self::Ldrw { to: reg, from: mem_location },
            LoadOrStoreMnemonic::Ldrd => Self::Ldrd { to: reg, from: mem_location },
            LoadOrStoreMnemonic::Ldrq => Self::Ldrq { to: reg, from: mem_location },
            LoadOrStoreMnemonic::Str => Self::Str { to: mem_location, from: reg },
            LoadOrStoreMnemonic::Strb => Self::Strb { to: mem_location, from: reg },
            LoadOrStoreMnemonic::Strh => Self::Strh { to: mem_location, from: reg },
            LoadOrStoreMnemonic::Strw => Self::Strw { to: mem_location, from: reg },
            LoadOrStoreMnemonic::Strd => Self::Strd { to: mem_location, from: reg },
            LoadOrStoreMnemonic::Strq => Self::Strq { to: mem_location, from: reg },
        }
    }
}

#[expect(clippy::cast_possible_truncation)]
impl Instruction {
    /// Move a memory address into a register.
    #[inline]
    fn adr<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        addr: u64,
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

    /// Store an 8-bit value from a register into a memory location.
    #[inline]
    fn strb<const MEM_SIZE: usize, Insts, Bytes>(
        from: Register,
        to: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let to_addr = to.resolve(processor);
        let val = processor.registers.get_reg(from);
        processor.mem.try_write(to_addr, val as u8) // Discard the upper bytes of the u64 value
    }

    /// Store a 16-bit value from a register into a memory location.
    #[inline]
    fn strh<const MEM_SIZE: usize, Insts, Bytes>(
        from: Register,
        to: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let to_addr = to.resolve(processor);
        let val = processor.registers.get_reg(from);
        let bytes = (val as u16).to_le_bytes(); // Discard the upper bytes of the u64 value

        processor.mem.try_write_slice(to_addr, &bytes)
    }

    /// Store a 32-bit value from a register into a memory location.
    #[inline]
    fn strw<const MEM_SIZE: usize, Insts, Bytes>(
        from: Register,
        to: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let to_addr = to.resolve(processor);
        let val = processor.registers.get_reg(from);
        let bytes = (val as u32).to_le_bytes(); // Discard the upper bytes of the u64 value

        processor.mem.try_write_slice(to_addr, &bytes)
    }

    /// Store a 64-bit value from a register into a memory location.
    #[inline]
    fn strd<const MEM_SIZE: usize, Insts, Bytes>(
        from: Register,
        to: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let to_addr = to.resolve(processor);
        let val = processor.registers.get_reg(from);
        let bytes = val.to_le_bytes();

        processor.mem.try_write_slice(to_addr, &bytes)
    }

    // TODO: Handle 128bit immediate values when storing the value in memory -> Add u128 registers
    /// Store a 128-bit value from a register into a memory location.
    #[inline]
    fn strq<const MEM_SIZE: usize, Insts, Bytes>(
        from: Register,
        to: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let to_addr = to.resolve(processor);
        let val = processor.registers.get_reg(from);
        let bytes = u128::from(val).to_le_bytes(); // Fills the upper bytes with 0

        processor.mem.try_write_slice(to_addr, &bytes)
    }

    /// Store a 64-bit value from a register to a memory location.
    #[inline]
    fn str<const MEM_SIZE: usize, Insts, Bytes>(
        from: Register,
        to: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let to_addr = to.resolve(processor);
        let val = processor.registers.get_reg(from);
        let bytes = val.to_le_bytes();

        processor.mem.try_write_slice(to_addr, &bytes)
    }

    /// Load an 8-bit value from a memory location into a register.
    #[inline]
    fn ldrb<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        from: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let from_addr = from.resolve(processor);
        let byte = processor.mem.try_read(from_addr)?;

        processor.registers.set_reg(to, u64::from(byte)); // Fills the upper bytes with 0
        Ok(())
    }

    /// Load a 16-bit value from a memory location into a register.
    #[inline]
    fn ldrh<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        from: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let from_addr = from.resolve(processor);
        let bytes = processor.mem.try_read_slice(from_addr, size_of::<u16>())?;
        let val = u16::from_le_bytes(*bytes.as_array().expect("Just read two bytes."));

        processor.registers.set_reg(to, u64::from(val)); // Fills the upper bytes with 0
        Ok(())
    }

    /// Load a 32-bit value from a memory location into a register.
    #[inline]
    fn ldrw<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        from: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let from_addr = from.resolve(processor);
        let bytes = processor.mem.try_read_slice(from_addr, size_of::<u32>())?;
        let val = u32::from_le_bytes(*bytes.as_array().expect("Just read four bytes."));

        processor.registers.set_reg(to, u64::from(val)); // Fills the upper bytes with 0
        Ok(())
    }

    /// Load a 64-bit value from a memory location into a register.
    #[inline]
    fn ldrd<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        from: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let from_addr = from.resolve(processor);
        let bytes = processor.mem.try_read_slice(from_addr, size_of::<u64>())?;
        let val = u64::from_le_bytes(*bytes.as_array().expect("Just read eight bytes."));

        processor.registers.set_reg(to, val);
        Ok(())
    }

    // TODO: This always discards the upper bytes
    /// Load a 128-bit value from a memory location into a register.
    #[inline]
    fn ldrq<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        from: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let from_addr = from.resolve(processor);
        let bytes = processor.mem.read_slice(from_addr, size_of::<u128>());
        let val = u128::from_le_bytes(*bytes.as_array().expect("Just read sixteen bytes."));

        processor.registers.set_reg(to, val as u64); // Discards the upper bytes
    }

    /// Load a 64-bit value from a memory location into a register.
    #[inline]
    fn ldr<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        from: MemoryLocation,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let from_addr = from.resolve(processor);
        let bytes = processor.mem.try_read_slice(from_addr, size_of::<u64>())?;
        let val = u64::from_le_bytes(*bytes.as_array().expect("Just read eight bytes."));

        processor.registers.set_reg(to, val);
        Ok(())
    }

    /// Push a value from the operand to the stack.
    #[inline]
    fn push<const MEM_SIZE: usize, Insts, Bytes>(
        from: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        // Move SP
        let new_sp = processor.registers.sp() - size_of::<u64>() as u64; // Substract size of register
        processor.registers.set_reg(Register::SP, new_sp);

        // Write value
        let bytes = from.resolve(processor).to_le_bytes();
        processor.mem.try_write_slice(new_sp, &bytes)
    }

    /// Pop a value from the stack to the register.
    #[inline]
    fn pop<const MEM_SIZE: usize, Insts, Bytes>(
        to: Register,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        // Read value
        let sp = processor.registers.sp();
        let bytes = processor.mem.try_read_slice(sp, size_of::<u64>())?;
        let val = u64::from_le_bytes(*bytes.as_array().expect("Just read eight bytes"));
        processor.registers.set_reg(to, val);

        // Move SP
        let new_sp = sp + size_of::<u64>() as u64; // Add size of register
        processor.registers.set_reg(Register::SP, new_sp);

        Ok(())
    }

    /// Call a subroutine at the program address specified by the operand.
    /// Pushes the current program counter onto the stack and sets the program counter to the address of the subroutine.
    #[inline]
    fn call<const MEM_SIZE: usize, Insts, Bytes>(
        addr: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        Self::push(Operand::Value(processor.registers.pc()), processor)?;
        processor.registers.set_reg(Register::PC, addr.resolve(processor));
        Ok(())
    }

    /// Return from a subroutine.
    /// Pops the return address from the stack and sets the program counter to the popped value.
    #[inline]
    fn ret<const MEM_SIZE: usize, Insts, Bytes>(processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>) -> InstructionResult {
        Self::pop(Register::PC, processor)
    }

    /// Set program pointer to value, effectively jumping to the instruction at this point in the program.
    /// The condition is checked before jumping and the jump is performed if the condition is met.
    #[inline]
    fn jmp<const MEM_SIZE: usize, Insts, Bytes>(
        to: u64,
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
        set_flags: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        // u64::overflowing_add returns res and unsigned wrap (carry)
        let (res, carry) = a.overflowing_add(b);

        if set_flags {
            // i64::overflowing_add returns res and signed wrap (overflow)
            let (_, overflow) = a.cast_signed().overflowing_add(b.cast_signed());

            processor.registers.set_reg(acc, res);

            processor.registers.set_flag(Flag::V, overflow);
            processor.registers.set_flag(Flag::C, carry);
            Self::set_signed_and_zero_flags(res.cast_signed(), processor);
        } else {
            processor.registers.set_reg(acc, res);
        }
    }

    /// Subtract the value of an operand (rhs) from a register (acc).
    #[inline]
    fn sub<const MEM_SIZE: usize, Insts, Bytes>(
        acc: Register,
        rhs: Operand,
        set_flags: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        // u64::overflowing_sub returns res and unsigned wrap (carry)
        let (res, carry) = a.overflowing_sub(b);

        if set_flags {
            // i64::overflowing_sub returns res and signed wrap (overflow)
            let (_, overflow) = a.cast_signed().overflowing_sub(b.cast_signed());

            processor.registers.set_reg(acc, res);

            processor.registers.set_flag(Flag::V, overflow);
            processor.registers.set_flag(Flag::C, carry);
            Self::set_signed_and_zero_flags(res.cast_signed(), processor);
        } else {
            processor.registers.set_reg(acc, res);
        }
    }

    /// Multiply the value of an operand (acc) with the value of a register (rhs).
    /// The result is stored in acc.
    #[inline]
    fn mul<const MEM_SIZE: usize, Insts, Bytes>(
        acc: Register,
        rhs: Operand,
        set_flags: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        // u64::overflowing_mul returns res and unsigned wrap (carry)
        let (res, carry) = a.overflowing_mul(b);

        if set_flags {
            // i64::overflowing_sub returns res and signed wrap (overflow)
            let (_, overflow) = a.cast_signed().overflowing_mul(b.cast_signed());

            processor.registers.set_reg(acc, res);

            processor.registers.set_flag(Flag::V, overflow);
            processor.registers.set_flag(Flag::C, carry);
            Self::set_signed_and_zero_flags(res.cast_signed(), processor);
        } else {
            processor.registers.set_reg(acc, a * b);
        }
    }

    /// Unsigned divide the value of an operand (acc) by the value of a register (rhs).
    /// The result is stored in acc.
    #[inline]
    fn div<const MEM_SIZE: usize, Insts, Bytes>(
        acc: Register,
        rhs: Operand,
        set_flags: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let a = processor.registers.get_reg(acc);
        let b = rhs.resolve(processor);

        if b == 0 {
            return Err(ProcessorError::DivisionByZero);
        }

        let res = a / b; // unsigned div cannot overflow

        processor.registers.set_reg(acc, res);
        if set_flags {
            processor.registers.set_flag(Flag::V, false); // unsigned div cannot overflow
            processor.registers.set_flag(Flag::C, false); // division never carries
            Self::set_signed_and_zero_flags(res.cast_signed(), processor);
        }

        Ok(())
    }

    /// Signed divide the value of an operand (acc) by the value of a register (rhs).
    /// The result is stored in acc.
    #[inline]
    fn sdiv<const MEM_SIZE: usize, Insts, Bytes>(
        acc: Register,
        rhs: Operand,
        set_flags: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) -> InstructionResult {
        let a = processor.registers.get_reg(acc).cast_signed();
        let b = rhs.resolve(processor).cast_signed();

        if b == 0 {
            return Err(ProcessorError::DivisionByZero);
        }

        let (res, overflow) = a.overflowing_div(b);

        processor.registers.set_reg(acc, res.cast_unsigned());
        if set_flags {
            processor.registers.set_flag(Flag::V, overflow);
            processor.registers.set_flag(Flag::C, false); // division never carries

            Self::set_signed_and_zero_flags(res, processor);
        }

        Ok(())
    }

    /// Increment the value in a register by one.
    #[inline]
    fn inc<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        set_flags: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        if set_flags {
            Self::add(reg, Operand::Value(1), true, processor);
        } else {
            processor.registers.inc(reg);
        }
    }

    /// Decrement the value in a register by one.
    #[inline]
    fn dec<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        set_flags: bool,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        if set_flags {
            Self::sub(reg, Operand::Value(1), true, processor);
        } else {
            processor.registers.dec(reg);
        }
    }

    /// Sets the signed and zero flags by comparing `val` to 0.
    #[inline]
    fn set_signed_and_zero_flags<const MEM_SIZE: usize, Insts, Bytes>(
        val: i64,
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

    /// Compares two operands and sets the flags accordingly.
    #[inline]
    fn cmp<const MEM_SIZE: usize, Insts, Bytes>(
        lhs: Operand,
        rhs: Operand,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = lhs.resolve(processor);
        let b = rhs.resolve(processor);

        // u64::overflowing_sub returns res and unsigned wrap (carry)
        let (res, carry) = a.overflowing_sub(b);

        // i64::overflowing_sub returns res and signed wrap (overflow)
        let (_, overflow) = a.cast_signed().overflowing_sub(b.cast_signed());

        processor.registers.set_flag(Flag::V, overflow);
        processor.registers.set_flag(Flag::C, carry);
        Self::set_signed_and_zero_flags(res.cast_signed(), processor);
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
    fn not<const MEM_SIZE: usize, Insts, Bytes>(reg: Register, processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, !a);
    }

    /// Shift the value in the register left by the specified number of bits.
    #[inline]
    fn shl<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        val: u32,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a.checked_shl(val).unwrap_or(0));
    }

    /// Shift the value in the register right by the specified number of bits.
    #[inline]
    fn shr<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        val: u32,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);
        processor.registers.set_reg(reg, a.checked_shr(val).unwrap_or(0));
    }

    /// Arithmetic shift the value in the register right by the specified number of bits, preserving the sign bit.
    #[inline]
    fn asr<const MEM_SIZE: usize, Insts, Bytes>(
        reg: Register,
        val: u32,
        processor: &mut Processor<MEM_SIZE, Self, Insts, Bytes>,
    ) {
        let a = processor.registers.get_reg(reg);

        // Stop at 63 so the sign bit fills the rest of the register
        let result = (a.cast_signed() >> val.min(63)).cast_unsigned();
        processor.registers.set_reg(reg, result);
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
    use pretty_assertions_sorted::assert_eq;

    const MEM_SIZE: usize = 32;
    type IS = Instruction;
    type P = Vec<IS>;
    type Bytes = Vec<u8>;

    mod mov {
        use super::{super::*, Bytes, IS, MEM_SIZE, P, assert_eq};

        #[test]
        fn test_move_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 10);
            let _ = IS::execute(Instruction::Mov { from: Operand::Register(Register::R0), to: Register::R1 }, &mut processor);
            assert_eq!(processor.registers.get_reg(Register::R1), processor.registers.get_reg(Register::R0));
        }

        #[test]
        fn test_move_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            let _ = IS::execute(Instruction::Mov { to: Register::R0, from: Operand::Value(10) }, &mut processor);
            assert_eq!(processor.registers.get_reg(Register::R0), 10);
        }
    }

    // TODO: test all ldr and str variants
    mod ldr_str {
        use super::{super::*, Bytes, IS, MEM_SIZE, P, assert_eq};

        #[test]
        fn str_direct_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 42);

            let _ = IS::execute(Instruction::Str { from: Register::R0, to: MemoryLocation::Labeled(0) }, &mut processor);

            assert_eq!(processor.mem.read(0), 42);
        }

        #[test]
        fn str_indirect_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R0, 42);
            processor.registers.set_reg(Register::R1, 1);

            // Positive value offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(0) },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(1), 42);

            // Negative value offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(-1isize as u64) },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(0), 42);

            // Register offset
            let _ = IS::execute(
                Instruction::Str {
                    from: Register::R0,
                    to: MemoryLocation::Offset { base: Register::R1, offset: Operand::Register(Register::R1) },
                },
                &mut processor,
            );
            assert_eq!(processor.mem.read(2), 42);
        }

        #[test]
        #[should_panic]
        fn str_invalid_memory_location_panics() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            let _ = IS::execute(
                Instruction::Str { from: Register::R0, to: MemoryLocation::Labeled(MEM_SIZE as u64) },
                &mut processor,
            );

            unreachable!("Panic should happen before")
        }

        #[test]
        fn ldr_direct_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.mem.write(0, 42);

            let _ = IS::execute(Instruction::Ldr { to: Register::R0, from: MemoryLocation::Labeled(0) }, &mut processor);

            assert_eq!(processor.registers.get_reg(Register::R0), 42);
        }

        #[test]
        fn ldr_indirect_mem_location() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            processor.registers.set_reg(Register::R1, 1);
            processor.mem.write(0, 42);
            processor.mem.write(1, 43);
            processor.mem.write(2, 44);

            println!("{:?}", &processor.mem[0..3]);

            // Positive value offset
            let _ = IS::execute(
                Instruction::Ldrb {
                    to: Register::R0,
                    from: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(0) },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 43);

            // Negative value offset
            let _ = IS::execute(
                Instruction::Ldrb {
                    to: Register::R0,
                    from: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(-1isize as u64) },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 42);

            // Register offset
            let _ = IS::execute(
                Instruction::Ldrb {
                    to: Register::R0,
                    from: MemoryLocation::Offset { base: Register::R1, offset: Operand::Register(Register::R1) },
                },
                &mut processor,
            );
            assert_eq!(processor.registers.get_reg(Register::R0), 44);
        }

        #[test]
        #[should_panic]
        fn ldr_invalid_memory_location_panics() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            let _ = IS::execute(
                Instruction::Ldr { from: MemoryLocation::Labeled(MEM_SIZE as u64), to: Register::R0 },
                &mut processor,
            );

            unreachable!("Panic should happen before")
        }
    }

    mod arithmetic {
        use super::{Bytes, IS, MEM_SIZE, P, assert_eq};

        struct Flags {
            carry: bool,
            signed: bool,
            overflow: bool,
            zero: bool,
        }

        macro_rules! exec {
            ($instr: ident, $lhs: expr, $rhs: expr, $res: expr, $flags: ident, $err: ident) => {
                let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
                processor.registers.set_reg(Register::R0, $lhs);
                processor.registers.set_reg(Register::R1, $rhs);
                let res = IS::execute(
                    Instruction::$instr { acc: Register::R0, rhs: Operand::Register(Register::R1), set_flags: true },
                    &mut processor,
                );

                if $err.is_ok() {
                    res.expect(concat!(stringify!($instr), "{} cannot return result"));
                    assert_eq!(processor.registers.get_reg(Register::R0), $res);
                    assert_eq!(processor.registers.get_flag(Flag::C), $flags.carry, "carry");
                    assert_eq!(processor.registers.get_flag(Flag::S), $flags.signed, "signed");
                    assert_eq!(processor.registers.get_flag(Flag::V), $flags.overflow, "overflow");
                    assert_eq!(processor.registers.get_flag(Flag::Z), $flags.zero, "zero");
                } else {
                    assert_eq!(res, $err);
                }
            };
        }

        mod add {
            use super::{super::*, Bytes, Flags, IS, MEM_SIZE, P, assert_eq};

            fn add(lhs: u64, rhs: u64, res: u64, flags: Flags) {
                let err = InstructionResult::Ok(());
                exec!(Add, lhs, rhs, res, flags, err);
            }

            #[test]
            fn unsigned() {
                add(
                    5,  // 0x0000_0000_0000_0005
                    10, // 0x0000_0000_0000_000A
                    15, // 0x0000_0000_0000_000F
                    Flags {
                        carry: false, // No unsigned overflow
                        signed: false,
                        overflow: false, // No signed overflow: pos + pos = pos
                        zero: false,
                    },
                );
            }

            #[test]
            fn unsigned_overflow() {
                add(
                    u64::MAX, // 0xFFFF_FFFF_FFFF_FFFF
                    1,        // 0x0000_0000_0000_0001
                    u64::MIN, // 0x0000000000000000000
                    Flags {
                        carry: true, // Unsigned overflow: wrap past u64::MAX
                        signed: false,
                        overflow: false, // No signed overflow: opposing signs cannot cause signed overflow
                        zero: true,      // u64::MIN == 0
                    },
                );
            }

            #[test]
            fn signed_res_negative() {
                add(
                    -5i64 as u64,  // 0xFFFF_FFFF_FFFF_FFFB
                    -10i64 as u64, // 0xFFFF_FFFF_FFFF_FFF6
                    -15i64 as u64, // 0xFFFF_FFFF_FFFF_FFF1
                    Flags {
                        carry: true, // Unsigned overflow: wrap past u64::MAX
                        signed: true,
                        overflow: false, // No signed overflow: neg + neg = neg
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_res_positive() {
                add(
                    -5i64 as u64, // 0xFFFF_FFFF_FFFF_FFFB
                    10,           // 0x0000_0000_0000_000A
                    5,            // 0x0000_0000_0000_0005
                    Flags {
                        carry: true, // Unsigned overflow: wrap past u64::MAX
                        signed: false,
                        overflow: false, // No signed overflow: opposing signs cannot cause signed overflow
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_overflow() {
                add(
                    i64::MAX as u64, // 0x7FFF_FFFF_FFFF_FFFF
                    1,               // 0x0000_0000_0000_0001
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    Flags {
                        carry: false, // No unsigned overflow
                        signed: true,
                        overflow: true, // Signed overflow: pos + pos = neg
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_underflow() {
                add(
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    -1i64 as u64,    // 0xFFFF_FFFF_FFFF_FFFF
                    i64::MAX as u64, // 0x7FFF_FFFF_FFFF_FFFF
                    Flags {
                        carry: true, // Unsigned overflow: neg + neg = pos
                        signed: false,
                        overflow: true, // Signed overflow: wrap past u64::MAX
                        zero: false,
                    },
                );
            }
        }

        mod sub {
            use super::{super::*, Bytes, Flags, IS, MEM_SIZE, P, assert_eq};

            fn sub(lhs: u64, rhs: u64, res: u64, flags: Flags) {
                let err = InstructionResult::Ok(());
                exec!(Sub, lhs, rhs, res, flags, err);
            }

            #[test]
            fn unsigned() {
                sub(
                    15, // 0x0000_0000_0000_000F
                    10, // 0x0000_0000_0000_000A
                    5,  // 0x0000_0000_0000_0005
                    Flags {
                        carry: false, // No unsigned overflow
                        signed: false,
                        overflow: false, // No signed overflow: pos - pos = pos
                        zero: false,
                    },
                );
            }

            #[test]
            fn unsigned_underflow() {
                sub(
                    u64::MIN, // 0x0000000000000000000
                    1,        // 0x0000_0000_0000_0001
                    u64::MAX, // 0xFFFF_FFFF_FFFF_FFFF
                    Flags {
                        carry: true,     // Unsigned underflow: 0 < 1
                        signed: true,    // Highest bit == 1
                        overflow: false, // No signed underflow: crosses 0, not u64::MIN/MAX border
                        zero: false,
                    },
                );
            }

            #[test]
            fn res_zero() {
                sub(
                    15, // 0x0000_0000_0000_000F
                    15, // 0x0000_0000_0000_000F
                    0,  // 0x0000_0000_0000_0000
                    Flags {
                        carry: false, // No unsigned overflow
                        signed: false,
                        overflow: false, // No signed overflow: pos - pos = pos
                        zero: true,
                    },
                );
            }

            #[test]
            fn signed_res_negative() {
                sub(
                    -5i64 as u64,  // 0xFFFF_FFFF_FFFF_FFFB
                    10,            // 0x0000_0000_0000_000A
                    -15i64 as u64, // 0xFFFF_FFFF_FFFF_FFF1
                    Flags {
                        carry: false, // No unsigned overflow: -5 > 10 (bits)
                        signed: true,
                        overflow: false, // No signed overflow: result sign matches first operand
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_res_positive() {
                sub(
                    -5i64 as u64,  // 0xFFFF_FFFF_FFFF_FFFB
                    -10i64 as u64, // 0xFFFF_FFFF_FFFF_FFF6
                    5,             // 0x0000_0000_0000_0005
                    Flags {
                        carry: false, // No unsigned overflow: -5 > -10 (bits)
                        signed: false,
                        overflow: false, // No signed overflow: Not crossing i64::MAX/MIN
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_overflow() {
                sub(
                    i64::MAX as u64, // 0x7FFF_FFFF_FFFF_FFFF
                    -1i64 as u64,    // 0xFFFF_FFFF_FFFF_FFFF
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    Flags {
                        carry: true, // Unsigned overflow: i64::MAX < -1 (bits)
                        signed: true,
                        overflow: true, // Signed overflow: pos - neg = neg
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_underflow() {
                sub(
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    1,               // 0x0000_0000_0000_0001
                    i64::MAX as u64, // 0x7FFF_FFFF_FFFF_FFFF
                    Flags {
                        carry: false, // No signed underflow: i64::MIN > 1 (bits)
                        signed: false,
                        overflow: true, // Unsigned underflow: wraps i64::MIN
                        zero: false,
                    },
                );
            }
        }

        mod mul {
            use super::{super::*, Bytes, Flags, IS, MEM_SIZE, P, assert_eq};

            fn mul(lhs: u64, rhs: u64, res: u64, flags: Flags) {
                let err = InstructionResult::Ok(());
                exec!(Mul, lhs, rhs, res, flags, err);
            }

            #[test]
            fn unsigned() {
                mul(5, 10, 50, Flags { carry: false, signed: false, overflow: false, zero: false });
            }

            #[test]
            fn signed() {
                mul(
                    -5i64 as u64, // 0xFFFF_FFFF_FFFF_FFFB
                    10,
                    -50i64 as u64, // 0xFFFF_FFFF_FFFF_FFCE
                    Flags {
                        carry: true, // Unsigned overflow: 0xFF...FB * 10 > u64::MAX
                        signed: true,
                        overflow: false, // No signed overflow: -5 * 10 = -50 (Fits in i64)
                        zero: false,
                    },
                );
            }

            #[test]
            fn res_zero() {
                mul(0, 5, 0, Flags { carry: false, signed: false, overflow: false, zero: true });
            }

            #[test]
            fn unsigned_overflow() {
                mul(
                    u64::MAX, // 0xFFFF_FFFF_FFFF_FFFF (-1 as i64)
                    2,
                    -2i64 as u64, // 0xFFFF_FFFF_FFFF_FFFE OR -1 * 2
                    Flags {
                        carry: true, // Unsigned overflow: result requires 65 bits
                        signed: true,
                        overflow: false, // No signed overflow: -1 * 2 = -2 (Fits in i64)
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_overflow() {
                mul(
                    i64::MAX as u64, // 0x7FFF_FFFF_FFFF_FFFF
                    2,
                    -2i64 as u64, // 0xFFFF_FFFF_FFFF_FFFE
                    Flags {
                        carry: false,   // No unsigned overflow: 0x7F...FF * 2 fits in u64
                        signed: true,   // MSB flipped to 1
                        overflow: true, // Signed overflow: pos * pos = neg
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_underflow() {
                mul(
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    2,
                    0, // 0x0000_0000_0000_0000
                    Flags {
                        carry: true, // Unsigned overflow: 0x80..00 * 2 = 0x1_00..00
                        signed: false,
                        overflow: true, // Signed overflow: neg * pos = 0
                        zero: true,
                    },
                );
            }

            #[test]
            fn signed_min_by_minus_one() {
                mul(
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    -1i64 as u64,    // 0xFFFF_FFFF_FFFF_FFFF
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    Flags {
                        carry: true, // Unsigned overflow
                        signed: true,
                        overflow: true, // Signed overflow: neg * neg = neg
                        zero: false,
                    },
                );
            }

            #[test]
            fn unsigned_max_by_max() {
                mul(
                    u64::MAX, // 0xFFFF_FFFF_FFFF_FFFF
                    u64::MAX, // 0xFFFF_FFFF_FFFF_FFFF
                    1,        // 0x0000_0000_0000_0001
                    Flags {
                        carry: true, // Unsigned overflow
                        signed: false,
                        overflow: false, // No signed overflow: -1 * -1 = 1
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_max_by_max() {
                mul(
                    i64::MAX as u64, // 0x7FFF_FFFF_FFFF_FFFF
                    i64::MAX as u64, // 0x7FFF_FFFF_FFFF_FFFF
                    1,
                    Flags {
                        carry: true, // Unsigned overflow
                        signed: false,
                        overflow: true, // Signed overflow
                        zero: false,
                    },
                );
            }

            #[test]
            fn signed_min_by_min() {
                mul(
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    0,
                    Flags {
                        carry: true, // Unsigned overflow
                        signed: false,
                        overflow: true, // Signed overflow
                        zero: true,
                    },
                );
            }
        }

        mod div {
            use procem::processor::ProcessorError;

            use super::{super::*, Bytes, Flags, IS, MEM_SIZE, P, assert_eq};

            fn div(lhs: u64, rhs: u64, res: u64, flags: Flags, err: InstructionResult) {
                exec!(Div, lhs, rhs, res, flags, err);
            }

            fn sdiv(lhs: u64, rhs: u64, res: u64, flags: Flags, err: InstructionResult) {
                exec!(Sdiv, lhs, rhs, res, flags, err);
            }

            #[test]
            fn unsigned_fractional_truncation() {
                div(5, 10, 0, Flags { carry: false, signed: false, overflow: false, zero: true }, Ok(()));
            }

            #[test]
            fn signed_fractional_truncation() {
                sdiv(-5i64 as u64, 10, 0, Flags { carry: false, signed: false, overflow: false, zero: true }, Ok(()));
            }

            #[test]
            fn signed_min_by_minus_one() {
                sdiv(
                    i64::MIN as u64, // 0x8000_0000_0000_0000
                    -1i64 as u64,    // 0xFFFF_FFFF_FFFF_FFFF
                    i64::MIN as u64, // Wraps around to MIN
                    Flags {
                        carry: false,
                        signed: true,
                        overflow: true, // Signed overflow: exceeds i64::MAX
                        zero: false,
                    },
                    Ok(()),
                );
            }

            #[test]
            fn unsigned_by_zero() {
                div(
                    50,
                    0,
                    474238974, // Not important, will not get evaluated
                    Flags {
                        // No flags are set
                        carry: false,
                        signed: false,
                        overflow: false,
                        zero: false,
                    },
                    Err(ProcessorError::DivisionByZero),
                );
            }

            #[test]
            fn signed_by_zero() {
                sdiv(
                    -50i64 as u64,
                    0,
                    474238974, // Not important, will not get evaluated
                    Flags {
                        // No flags are set
                        carry: false,
                        signed: false,
                        overflow: false,
                        zero: false,
                    },
                    Err(ProcessorError::DivisionByZero),
                );
            }
        }

        mod inc {
            use super::{super::*, Bytes, IS, MEM_SIZE, P, assert_eq};

            #[test]
            fn normal() {
                let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
                processor.registers.set_reg(Register::R0, 10);
                let _ = IS::execute(Instruction::Inc { reg: Register::R0, set_flags: true }, &mut processor);
                assert_eq!(processor.registers.get_reg(Register::R0), 11);
                assert_eq!(processor.registers.get_flag(Flag::C), false, "carry");
                assert_eq!(processor.registers.get_flag(Flag::S), false, "signed");
                assert_eq!(processor.registers.get_flag(Flag::V), false, "overflow");
                assert_eq!(processor.registers.get_flag(Flag::Z), false, "zero");
            }

            #[test]
            fn overflow() {
                let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
                processor.registers.set_reg(Register::R0, u64::MAX);
                let _ = IS::execute(Instruction::Inc { reg: Register::R0, set_flags: true }, &mut processor);
                assert_eq!(processor.registers.get_reg(Register::R0), u64::MIN);
                assert_eq!(processor.registers.get_flag(Flag::C), true, "carry");
                assert_eq!(processor.registers.get_flag(Flag::S), false, "signed");
                assert_eq!(processor.registers.get_flag(Flag::V), false, "overflow");
                assert_eq!(processor.registers.get_flag(Flag::Z), true, "zero");
            }
        }

        mod dec {
            use super::{super::*, Bytes, IS, MEM_SIZE, P, assert_eq};

            #[test]
            fn normal() {
                let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
                processor.registers.set_reg(Register::R0, 10);
                let _ = IS::execute(Instruction::Dec { reg: Register::R0, set_flags: true }, &mut processor);
                assert_eq!(processor.registers.get_reg(Register::R0), 9);
                assert_eq!(processor.registers.get_flag(Flag::C), false, "carry");
                assert_eq!(processor.registers.get_flag(Flag::S), false, "signed");
                assert_eq!(processor.registers.get_flag(Flag::V), false, "overflow");
                assert_eq!(processor.registers.get_flag(Flag::Z), false, "zero");
            }

            #[test]
            fn unsigned_underflow() {
                let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
                processor.registers.set_reg(Register::R0, u64::MIN);
                let _ = IS::execute(Instruction::Dec { reg: Register::R0, set_flags: true }, &mut processor);
                assert_eq!(processor.registers.get_reg(Register::R0), u64::MAX);
                assert_eq!(processor.registers.get_flag(Flag::C), true, "carry"); // Unsigned Overflow: 0 < 1 (bits)
                assert_eq!(processor.registers.get_flag(Flag::S), true, "signed");
                assert_eq!(processor.registers.get_flag(Flag::V), false, "overflow"); // No signed overflow
                assert_eq!(processor.registers.get_flag(Flag::Z), false, "zero");
            }

            #[test]
            fn signed_underflow() {
                let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
                processor.registers.set_reg(Register::R0, i64::MIN as u64);
                let _ = IS::execute(Instruction::Dec { reg: Register::R0, set_flags: true }, &mut processor);
                assert_eq!(processor.registers.get_reg(Register::R0), i64::MAX as u64);
                assert_eq!(processor.registers.get_flag(Flag::C), false, "carry"); // No unsigned overflow: i64::MIN > 1 (bits)
                assert_eq!(processor.registers.get_flag(Flag::S), false, "signed");
                assert_eq!(processor.registers.get_flag(Flag::V), true, "overflow"); // Signed overflow: neg - pos = pos
                assert_eq!(processor.registers.get_flag(Flag::Z), false, "zero");
            }
        }
    }

    mod jmp {
        use super::{super::*, Bytes, IS, MEM_SIZE, P, assert_eq};

        #[test]
        fn test_jmp() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0);
            let _ = IS::execute(Instruction::Jump { to: 2, condition: JumpCondition::Unconditional }, &mut processor);
            assert_eq!(processor.registers.get_reg(Register::PC), 2);
        }

        #[test]
        fn test_jmp_overflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0);
            let _ = IS::execute(Instruction::Jump { to: u64::MAX, condition: JumpCondition::Unconditional }, &mut processor);
            assert_eq!(processor.registers.get_reg(Register::PC), u64::MAX);
            let _ = IS::execute(Instruction::Inc { reg: Register::PC, set_flags: false }, &mut processor);
            assert_eq!(processor.registers.get_reg(Register::PC), u64::MIN);
        }

        #[test]
        fn test_jmp_underflow() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();
            assert_eq!(processor.registers.get_reg(Register::PC), 0);
            let _ = IS::execute(Instruction::Jump { to: u64::MIN, condition: JumpCondition::Unconditional }, &mut processor);
            assert_eq!(processor.registers.get_reg(Register::PC), u64::MIN);
            let _ = IS::execute(Instruction::Dec { reg: Register::PC, set_flags: false }, &mut processor);
            assert_eq!(processor.registers.get_reg(Register::PC), u64::MAX);
        }
    }

    mod cmp {
        use super::{super::*, Bytes, IS, MEM_SIZE, P, assert_eq};

        #[test]
        fn test_cmp_eq_reg() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            processor.registers.set_reg(Register::R0, 1);
            processor.registers.set_reg(Register::R1, 1);

            let _ = IS::execute(
                Instruction::Cmp { lhs: Operand::Register(Register::R0), rhs: Operand::Register(Register::R1) },
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

            let _ =
                IS::execute(Instruction::Cmp { lhs: Operand::Register(Register::R0), rhs: Operand::Value(1) }, &mut processor);
            assert_eq!(processor.registers.get_flag(Flag::C), false);
            assert_eq!(processor.registers.get_flag(Flag::S), false);
            assert_eq!(processor.registers.get_flag(Flag::V), false);
            assert_eq!(processor.registers.get_flag(Flag::Z), true);
        }

        #[test]
        fn test_cmp_eq_val() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            let _ = IS::execute(Instruction::Cmp { lhs: Operand::Value(1), rhs: Operand::Value(1) }, &mut processor);
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
                Instruction::Cmp { lhs: Operand::Register(Register::R0), rhs: Operand::Register(Register::R1) },
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
                Instruction::Cmp { lhs: Operand::Register(Register::R0), rhs: Operand::Register(Register::R1) },
                &mut processor,
            );
            assert_eq!(processor.registers.get_flag(Flag::C), false);
            assert_eq!(processor.registers.get_flag(Flag::S), false);
            assert_eq!(processor.registers.get_flag(Flag::V), false);
            assert_eq!(processor.registers.get_flag(Flag::Z), false);
        }
    }

    mod stack {
        use super::{super::*, Bytes, IS, MEM_SIZE, P, assert_eq};

        #[test]
        fn test_push_pop() {
            let mut processor = Processor::<MEM_SIZE, IS, P, Bytes>::new();

            processor.registers.set_reg(Register::SP, MEM_SIZE as u64 - 1);
            processor.registers.set_reg(Register::R0, 1);

            [
                Instruction::Push { from: Operand::Register(Register::R0) },
                Instruction::Push { from: Operand::Register(Register::R0) },
                Instruction::Pop { to: Register::R1 },
                Instruction::Push { from: Operand::Register(Register::R0) },
                Instruction::Pop { to: Register::R2 },
                Instruction::Pop { to: Register::R3 },
            ]
            .iter()
            .for_each(|&inst| IS::execute(inst, &mut processor).unwrap());

            assert_eq!(processor.registers.get_reg(Register::R1), 1);
            assert_eq!(processor.registers.get_reg(Register::R2), 1);
            assert_eq!(processor.registers.get_reg(Register::R3), 1);
            assert_eq!(processor.registers.sp(), MEM_SIZE as u64 - 1);
        }
    }
}
