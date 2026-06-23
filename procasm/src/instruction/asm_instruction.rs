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
pub enum ASMRegLabelInstruction {
    Adr,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMRegOperandInstruction {
    Add,
    AddS,
    And,
    Div,
    DivS,
    Sdiv,
    SdivS,
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

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum ASMLoadOrStoreInstruction {
    Ldr,
    Ldrb,
    Ldrh,
    Ldrw,
    Ldrd,
    Ldrq,
    Str,
    Strb,
    Strh,
    Strw,
    Strd,
    Strq,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ASMInstruction {
    Jump(ASMJumpInstruction),
    NoArg(ASMNoArgInstruction),
    RegLabel(ASMRegLabelInstruction),
    RegOperand(ASMRegOperandInstruction),
    Rotate(ASMRotateInstruction),
    Shift(ASMShiftInstruction),
    SingleOperand(ASMSingleOperandInstruction),
    SingleReg(ASMSingleRegInstruction),
    TwoOperand(ASMTwoOperandInstruction),
    LoadOrStore(ASMLoadOrStoreInstruction),
}

impl TryFrom<&[u8]> for ASMInstruction {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let inst = match value {
            inst if inst.eq_ignore_ascii_case(b"ADD") => Self::RegOperand(ASMRegOperandInstruction::Add),
            inst if inst.eq_ignore_ascii_case(b"ADDS") => Self::RegOperand(ASMRegOperandInstruction::AddS),
            inst if inst.eq_ignore_ascii_case(b"ADR") => Self::RegLabel(ASMRegLabelInstruction::Adr),
            inst if inst.eq_ignore_ascii_case(b"AND") => Self::RegOperand(ASMRegOperandInstruction::And),
            inst if inst.eq_ignore_ascii_case(b"CALL") => Self::SingleOperand(ASMSingleOperandInstruction::Call),
            inst if inst.eq_ignore_ascii_case(b"CMP") => Self::TwoOperand(ASMTwoOperandInstruction::Cmp),
            inst if inst.eq_ignore_ascii_case(b"DEC") => Self::SingleReg(ASMSingleRegInstruction::Dec),
            inst if inst.eq_ignore_ascii_case(b"DECS") => Self::SingleReg(ASMSingleRegInstruction::DecS),
            inst if inst.eq_ignore_ascii_case(b"DIV") => Self::RegOperand(ASMRegOperandInstruction::Div),
            inst if inst.eq_ignore_ascii_case(b"DIVS") => Self::RegOperand(ASMRegOperandInstruction::DivS),
            inst if inst.eq_ignore_ascii_case(b"SDIV") => Self::RegOperand(ASMRegOperandInstruction::Sdiv),
            inst if inst.eq_ignore_ascii_case(b"SDIVS") => Self::RegOperand(ASMRegOperandInstruction::SdivS),
            inst if inst.eq_ignore_ascii_case(b"INC") => Self::SingleReg(ASMSingleRegInstruction::Inc),
            inst if inst.eq_ignore_ascii_case(b"INCS") => Self::SingleReg(ASMSingleRegInstruction::IncS),
            inst if inst.eq_ignore_ascii_case(b"JC") => Self::Jump(ASMJumpInstruction::Jc),
            inst if inst.eq_ignore_ascii_case(b"JG") => Self::Jump(ASMJumpInstruction::Jg),
            inst if inst.eq_ignore_ascii_case(b"JGE") => Self::Jump(ASMJumpInstruction::Jge),
            inst if inst.eq_ignore_ascii_case(b"JL") => Self::Jump(ASMJumpInstruction::Jl),
            inst if inst.eq_ignore_ascii_case(b"JLE") => Self::Jump(ASMJumpInstruction::Jle),
            inst if inst.eq_ignore_ascii_case(b"JMP") => Self::Jump(ASMJumpInstruction::Jmp),
            inst if inst.eq_ignore_ascii_case(b"JNC") => Self::Jump(ASMJumpInstruction::Jnc),
            inst if inst.eq_ignore_ascii_case(b"JNS") => Self::Jump(ASMJumpInstruction::Jns),
            inst if inst.eq_ignore_ascii_case(b"JNZ") => Self::Jump(ASMJumpInstruction::Jnz),
            inst if inst.eq_ignore_ascii_case(b"JS") => Self::Jump(ASMJumpInstruction::Js),
            inst if inst.eq_ignore_ascii_case(b"JZ") => Self::Jump(ASMJumpInstruction::Jz),
            inst if inst.eq_ignore_ascii_case(b"LDR") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Ldr),
            inst if inst.eq_ignore_ascii_case(b"LDRB") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Ldrb),
            inst if inst.eq_ignore_ascii_case(b"LDRH") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Ldrh),
            inst if inst.eq_ignore_ascii_case(b"LDRW") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Ldrw),
            inst if inst.eq_ignore_ascii_case(b"LDRD") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Ldrd),
            inst if inst.eq_ignore_ascii_case(b"LDRQ") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Ldrq),
            inst if inst.eq_ignore_ascii_case(b"MOV") => Self::RegOperand(ASMRegOperandInstruction::Mov),
            inst if inst.eq_ignore_ascii_case(b"MUL") => Self::RegOperand(ASMRegOperandInstruction::Mul),
            inst if inst.eq_ignore_ascii_case(b"MULS") => Self::RegOperand(ASMRegOperandInstruction::MulS),
            inst if inst.eq_ignore_ascii_case(b"NOP") => Self::NoArg(ASMNoArgInstruction::Nop),
            inst if inst.eq_ignore_ascii_case(b"NOT") => Self::SingleReg(ASMSingleRegInstruction::Not),
            inst if inst.eq_ignore_ascii_case(b"OR") => Self::RegOperand(ASMRegOperandInstruction::Or),
            inst if inst.eq_ignore_ascii_case(b"POP") => Self::SingleReg(ASMSingleRegInstruction::Pop),
            inst if inst.eq_ignore_ascii_case(b"PUSH") => Self::SingleOperand(ASMSingleOperandInstruction::Push),
            inst if inst.eq_ignore_ascii_case(b"RET") => Self::NoArg(ASMNoArgInstruction::Ret),
            inst if inst.eq_ignore_ascii_case(b"ROL") => Self::Rotate(ASMRotateInstruction::Rol),
            inst if inst.eq_ignore_ascii_case(b"ROR") => Self::Rotate(ASMRotateInstruction::Ror),
            inst if inst.eq_ignore_ascii_case(b"SHL") => Self::Shift(ASMShiftInstruction::Shl),
            inst if inst.eq_ignore_ascii_case(b"SHR") => Self::Shift(ASMShiftInstruction::Shr),
            inst if inst.eq_ignore_ascii_case(b"STR") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Str),
            inst if inst.eq_ignore_ascii_case(b"STRB") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Strb),
            inst if inst.eq_ignore_ascii_case(b"STRH") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Strh),
            inst if inst.eq_ignore_ascii_case(b"STRW") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Strw),
            inst if inst.eq_ignore_ascii_case(b"STRD") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Strd),
            inst if inst.eq_ignore_ascii_case(b"STRQ") => Self::LoadOrStore(ASMLoadOrStoreInstruction::Strq),
            inst if inst.eq_ignore_ascii_case(b"SUB") => Self::RegOperand(ASMRegOperandInstruction::Sub),
            inst if inst.eq_ignore_ascii_case(b"SUBS") => Self::RegOperand(ASMRegOperandInstruction::SubS),
            inst if inst.eq_ignore_ascii_case(b"XOR") => Self::RegOperand(ASMRegOperandInstruction::Xor),
            _ => return Err(()),
        };

        Ok(inst)
    }
}
