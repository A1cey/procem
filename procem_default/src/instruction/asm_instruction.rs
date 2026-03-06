#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMJumpInstruction {
    Jmp,
    Jz,
    Jnz,
    Jc,
    Jnc,
    Js,
    Jns,
    Jg,
    Jge,
    Jl,
    Jle,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMNoArgInstruction {
    Nop,
    Ret,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMRegOperandInstruction {
    Add,
    AddS,
    And,
    Div,
    DivS,
    Mov,
    Mul,
    MulS,
    Or,
    Sub,
    SubS,
    Xor,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMRotateInstruction {
    Rol,
    Ror,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMShiftInstruction {
    Shl,
    Shr,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMSingleOperandInstruction {
    Call,
    Push,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMSingleRegInstruction {
    Dec,
    DecS,
    Inc,
    IncS,
    Not,
    Pop,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMTwoOperandInstruction {
    Cmp,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMInstruction {
    Jump(ASMJumpInstruction),
    NoArg(ASMNoArgInstruction),
    RegOperand(ASMRegOperandInstruction),
    Rotate(ASMRotateInstruction),
    Shift(ASMShiftInstruction),
    SingleOperand(ASMSingleOperandInstruction),
    SingleReg(ASMSingleRegInstruction),
    TwoOperand(ASMTwoOperandInstruction),
}

impl TryFrom<&[u8]> for ASMInstruction {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let inst = match value {
            b"ADD" => Self::RegOperand(ASMRegOperandInstruction::Add),
            b"ADDS" => Self::RegOperand(ASMRegOperandInstruction::AddS),
            b"AND" => Self::RegOperand(ASMRegOperandInstruction::And),
            b"CALL" => Self::SingleOperand(ASMSingleOperandInstruction::Call),
            b"CMP" => Self::TwoOperand(ASMTwoOperandInstruction::Cmp),
            b"DEC" => Self::SingleReg(ASMSingleRegInstruction::Dec),
            b"DECS" => Self::SingleReg(ASMSingleRegInstruction::DecS),
            b"DIV" => Self::RegOperand(ASMRegOperandInstruction::Div),
            b"DIVS" => Self::RegOperand(ASMRegOperandInstruction::DivS),
            b"INC" => Self::SingleReg(ASMSingleRegInstruction::Inc),
            b"INCS" => Self::SingleReg(ASMSingleRegInstruction::IncS),
            b"JC" => Self::Jump(ASMJumpInstruction::Jc),
            b"JG" => Self::Jump(ASMJumpInstruction::Jg),
            b"JGE" => Self::Jump(ASMJumpInstruction::Jge),
            b"JL" => Self::Jump(ASMJumpInstruction::Jl),
            b"JLE" => Self::Jump(ASMJumpInstruction::Jle),
            b"JMP" => Self::Jump(ASMJumpInstruction::Jmp),
            b"JNC" => Self::Jump(ASMJumpInstruction::Jnc),
            b"JNS" => Self::Jump(ASMJumpInstruction::Jns),
            b"JNZ" => Self::Jump(ASMJumpInstruction::Jnz),
            b"JS" => Self::Jump(ASMJumpInstruction::Js),
            b"JZ" => Self::Jump(ASMJumpInstruction::Jz),
            b"MOV" => Self::RegOperand(ASMRegOperandInstruction::Mov),
            b"MUL" => Self::RegOperand(ASMRegOperandInstruction::Mul),
            b"MULS" => Self::RegOperand(ASMRegOperandInstruction::MulS),
            b"NOP" => Self::NoArg(ASMNoArgInstruction::Nop),
            b"NOT" => Self::SingleReg(ASMSingleRegInstruction::Not),
            b"OR" => Self::RegOperand(ASMRegOperandInstruction::Or),
            b"POP" => Self::SingleReg(ASMSingleRegInstruction::Pop),
            b"PUSH" => Self::SingleOperand(ASMSingleOperandInstruction::Push),
            b"RET" => Self::NoArg(ASMNoArgInstruction::Ret),
            b"ROL" => Self::Rotate(ASMRotateInstruction::Rol),
            b"ROR" => Self::Rotate(ASMRotateInstruction::Ror),
            b"SHL" => Self::Shift(ASMShiftInstruction::Shl),
            b"SHR" => Self::Shift(ASMShiftInstruction::Shr),
            b"SUB" => Self::RegOperand(ASMRegOperandInstruction::Sub),
            b"SUBS" => Self::RegOperand(ASMRegOperandInstruction::SubS),
            b"XOR" => Self::RegOperand(ASMRegOperandInstruction::Xor),
            _ => return Err(()),
        };

        Ok(inst)
    }
}
