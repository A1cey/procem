use ars::ascii::eq_ignore_ascii_case;

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
            inst if eq_ignore_ascii_case(inst, b"ADD") => Self::RegOperand(ASMRegOperandInstruction::Add),
            inst if eq_ignore_ascii_case(inst, b"ADDS") => Self::RegOperand(ASMRegOperandInstruction::AddS),
            inst if eq_ignore_ascii_case(inst, b"AND") => Self::RegOperand(ASMRegOperandInstruction::And),
            inst if eq_ignore_ascii_case(inst, b"CALL") => Self::SingleOperand(ASMSingleOperandInstruction::Call),
            inst if eq_ignore_ascii_case(inst, b"CMP") => Self::TwoOperand(ASMTwoOperandInstruction::Cmp),
            inst if eq_ignore_ascii_case(inst, b"DEC") => Self::SingleReg(ASMSingleRegInstruction::Dec),
            inst if eq_ignore_ascii_case(inst, b"DECS") => Self::SingleReg(ASMSingleRegInstruction::DecS),
            inst if eq_ignore_ascii_case(inst, b"DIV") => Self::RegOperand(ASMRegOperandInstruction::Div),
            inst if eq_ignore_ascii_case(inst, b"DIVS") => Self::RegOperand(ASMRegOperandInstruction::DivS),
            inst if eq_ignore_ascii_case(inst, b"INC") => Self::SingleReg(ASMSingleRegInstruction::Inc),
            inst if eq_ignore_ascii_case(inst, b"INCS") => Self::SingleReg(ASMSingleRegInstruction::IncS),
            inst if eq_ignore_ascii_case(inst, b"JC") => Self::Jump(ASMJumpInstruction::Jc),
            inst if eq_ignore_ascii_case(inst, b"JG") => Self::Jump(ASMJumpInstruction::Jg),
            inst if eq_ignore_ascii_case(inst, b"JGE") => Self::Jump(ASMJumpInstruction::Jge),
            inst if eq_ignore_ascii_case(inst, b"JL") => Self::Jump(ASMJumpInstruction::Jl),
            inst if eq_ignore_ascii_case(inst, b"JLE") => Self::Jump(ASMJumpInstruction::Jle),
            inst if eq_ignore_ascii_case(inst, b"JMP") => Self::Jump(ASMJumpInstruction::Jmp),
            inst if eq_ignore_ascii_case(inst, b"JNC") => Self::Jump(ASMJumpInstruction::Jnc),
            inst if eq_ignore_ascii_case(inst, b"JNS") => Self::Jump(ASMJumpInstruction::Jns),
            inst if eq_ignore_ascii_case(inst, b"JNZ") => Self::Jump(ASMJumpInstruction::Jnz),
            inst if eq_ignore_ascii_case(inst, b"JS") => Self::Jump(ASMJumpInstruction::Js),
            inst if eq_ignore_ascii_case(inst, b"JZ") => Self::Jump(ASMJumpInstruction::Jz),
            inst if eq_ignore_ascii_case(inst, b"MOV") => Self::RegOperand(ASMRegOperandInstruction::Mov),
            inst if eq_ignore_ascii_case(inst, b"MUL") => Self::RegOperand(ASMRegOperandInstruction::Mul),
            inst if eq_ignore_ascii_case(inst, b"MULS") => Self::RegOperand(ASMRegOperandInstruction::MulS),
            inst if eq_ignore_ascii_case(inst, b"NOP") => Self::NoArg(ASMNoArgInstruction::Nop),
            inst if eq_ignore_ascii_case(inst, b"NOT") => Self::SingleReg(ASMSingleRegInstruction::Not),
            inst if eq_ignore_ascii_case(inst, b"OR") => Self::RegOperand(ASMRegOperandInstruction::Or),
            inst if eq_ignore_ascii_case(inst, b"POP") => Self::SingleReg(ASMSingleRegInstruction::Pop),
            inst if eq_ignore_ascii_case(inst, b"PUSH") => Self::SingleOperand(ASMSingleOperandInstruction::Push),
            inst if eq_ignore_ascii_case(inst, b"RET") => Self::NoArg(ASMNoArgInstruction::Ret),
            inst if eq_ignore_ascii_case(inst, b"ROL") => Self::Rotate(ASMRotateInstruction::Rol),
            inst if eq_ignore_ascii_case(inst, b"ROR") => Self::Rotate(ASMRotateInstruction::Ror),
            inst if eq_ignore_ascii_case(inst, b"SHL") => Self::Shift(ASMShiftInstruction::Shl),
            inst if eq_ignore_ascii_case(inst, b"SHR") => Self::Shift(ASMShiftInstruction::Shr),
            inst if eq_ignore_ascii_case(inst, b"SUB") => Self::RegOperand(ASMRegOperandInstruction::Sub),
            inst if eq_ignore_ascii_case(inst, b"SUBS") => Self::RegOperand(ASMRegOperandInstruction::SubS),
            inst if eq_ignore_ascii_case(inst, b"XOR") => Self::RegOperand(ASMRegOperandInstruction::Xor),
            _ => return Err(()),
        };

        Ok(inst)
    }
}
