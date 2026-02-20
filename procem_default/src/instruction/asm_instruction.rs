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

impl TryFrom<&str> for ASMInstruction {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let inst = match value {
            "ADD" => Self::RegOperand(ASMRegOperandInstruction::Add),
            "ADDS" => Self::RegOperand(ASMRegOperandInstruction::AddS),
            "AND" => Self::RegOperand(ASMRegOperandInstruction::And),
            "CALL" => Self::SingleOperand(ASMSingleOperandInstruction::Call),
            "CMP" => Self::TwoOperand(ASMTwoOperandInstruction::Cmp),
            "DEC" => Self::SingleReg(ASMSingleRegInstruction::Dec),
            "DECS" => Self::SingleReg(ASMSingleRegInstruction::DecS),
            "DIV" => Self::RegOperand(ASMRegOperandInstruction::Div),
            "DIVS" => Self::RegOperand(ASMRegOperandInstruction::DivS),
            "INC" => Self::SingleReg(ASMSingleRegInstruction::Inc),
            "INCS" => Self::SingleReg(ASMSingleRegInstruction::IncS),
            "JC" => Self::Jump(ASMJumpInstruction::Jc),
            "JG" => Self::Jump(ASMJumpInstruction::Jg),
            "JGE" => Self::Jump(ASMJumpInstruction::Jge),
            "JL" => Self::Jump(ASMJumpInstruction::Jl),
            "JLE" => Self::Jump(ASMJumpInstruction::Jle),
            "JMP" => Self::Jump(ASMJumpInstruction::Jmp),
            "JNC" => Self::Jump(ASMJumpInstruction::Jnc),
            "JNS" => Self::Jump(ASMJumpInstruction::Jns),
            "JNZ" => Self::Jump(ASMJumpInstruction::Jnz),
            "JS" => Self::Jump(ASMJumpInstruction::Js),
            "JZ" => Self::Jump(ASMJumpInstruction::Jz),
            "MOV" => Self::RegOperand(ASMRegOperandInstruction::Mov),
            "MUL" => Self::RegOperand(ASMRegOperandInstruction::Mul),
            "MULS" => Self::RegOperand(ASMRegOperandInstruction::MulS),
            "NOP" => Self::NoArg(ASMNoArgInstruction::Nop),
            "NOT" => Self::SingleReg(ASMSingleRegInstruction::Not),
            "OR" => Self::RegOperand(ASMRegOperandInstruction::Or),
            "POP" => Self::SingleReg(ASMSingleRegInstruction::Pop),
            "PUSH" => Self::SingleOperand(ASMSingleOperandInstruction::Push),
            "RET" => Self::NoArg(ASMNoArgInstruction::Ret),
            "ROL" => Self::Rotate(ASMRotateInstruction::Rol),
            "ROR" => Self::Rotate(ASMRotateInstruction::Ror),
            "SHL" => Self::Shift(ASMShiftInstruction::Shl),
            "SHR" => Self::Shift(ASMShiftInstruction::Shr),
            "SUB" => Self::RegOperand(ASMRegOperandInstruction::Sub),
            "SUBS" => Self::RegOperand(ASMRegOperandInstruction::SubS),
            "XOR" => Self::RegOperand(ASMRegOperandInstruction::Xor),
            _ => return Err(()),
        };

        Ok(inst)
    }
}
