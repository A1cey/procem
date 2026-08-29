#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum JumpMnemonic {
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
pub enum NoArgMnemonic {
    Nop,
    Ret,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum RegLabelMnemonic {
    Adr,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum RegOperandMnemonic {
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
pub enum RotateMnemonic {
    Rol,
    Ror,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum ShiftMnemonic {
    Shl,
    Shr,
    Asr,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum SingleOperandMnemonic {
    Call,
    Push,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum SingleRegMnemonic {
    Dec,
    DecS,
    Inc,
    IncS,
    Not,
    Pop,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, PartialOrd, Ord, Hash)]
pub enum TwoOperandMnemonic {
    Cmp,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub enum LoadOrStoreMnemonic {
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
pub enum Mnemonic {
    Jump(JumpMnemonic),
    NoArg(NoArgMnemonic),
    RegLabel(RegLabelMnemonic),
    RegOperand(RegOperandMnemonic),
    Rotate(RotateMnemonic),
    Shift(ShiftMnemonic),
    SingleOperand(SingleOperandMnemonic),
    SingleReg(SingleRegMnemonic),
    TwoOperand(TwoOperandMnemonic),
    LoadOrStore(LoadOrStoreMnemonic),
}

impl TryFrom<&[u8]> for Mnemonic {
    type Error = ();

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let inst = match value {
            inst if inst.eq_ignore_ascii_case(b"ADD") => Self::RegOperand(RegOperandMnemonic::Add),
            inst if inst.eq_ignore_ascii_case(b"ADDS") => Self::RegOperand(RegOperandMnemonic::AddS),
            inst if inst.eq_ignore_ascii_case(b"ADR") => Self::RegLabel(RegLabelMnemonic::Adr),
            inst if inst.eq_ignore_ascii_case(b"AND") => Self::RegOperand(RegOperandMnemonic::And),
            inst if inst.eq_ignore_ascii_case(b"CALL") => Self::SingleOperand(SingleOperandMnemonic::Call),
            inst if inst.eq_ignore_ascii_case(b"CMP") => Self::TwoOperand(TwoOperandMnemonic::Cmp),
            inst if inst.eq_ignore_ascii_case(b"DEC") => Self::SingleReg(SingleRegMnemonic::Dec),
            inst if inst.eq_ignore_ascii_case(b"DECS") => Self::SingleReg(SingleRegMnemonic::DecS),
            inst if inst.eq_ignore_ascii_case(b"DIV") => Self::RegOperand(RegOperandMnemonic::Div),
            inst if inst.eq_ignore_ascii_case(b"DIVS") => Self::RegOperand(RegOperandMnemonic::DivS),
            inst if inst.eq_ignore_ascii_case(b"SDIV") => Self::RegOperand(RegOperandMnemonic::Sdiv),
            inst if inst.eq_ignore_ascii_case(b"SDIVS") => Self::RegOperand(RegOperandMnemonic::SdivS),
            inst if inst.eq_ignore_ascii_case(b"INC") => Self::SingleReg(SingleRegMnemonic::Inc),
            inst if inst.eq_ignore_ascii_case(b"INCS") => Self::SingleReg(SingleRegMnemonic::IncS),
            inst if inst.eq_ignore_ascii_case(b"JC") => Self::Jump(JumpMnemonic::Jc),
            inst if inst.eq_ignore_ascii_case(b"JG") => Self::Jump(JumpMnemonic::Jg),
            inst if inst.eq_ignore_ascii_case(b"JGE") => Self::Jump(JumpMnemonic::Jge),
            inst if inst.eq_ignore_ascii_case(b"JL") => Self::Jump(JumpMnemonic::Jl),
            inst if inst.eq_ignore_ascii_case(b"JLE") => Self::Jump(JumpMnemonic::Jle),
            inst if inst.eq_ignore_ascii_case(b"JMP") => Self::Jump(JumpMnemonic::Jmp),
            inst if inst.eq_ignore_ascii_case(b"JNC") => Self::Jump(JumpMnemonic::Jnc),
            inst if inst.eq_ignore_ascii_case(b"JNS") => Self::Jump(JumpMnemonic::Jns),
            inst if inst.eq_ignore_ascii_case(b"JNZ") => Self::Jump(JumpMnemonic::Jnz),
            inst if inst.eq_ignore_ascii_case(b"JS") => Self::Jump(JumpMnemonic::Js),
            inst if inst.eq_ignore_ascii_case(b"JZ") => Self::Jump(JumpMnemonic::Jz),
            inst if inst.eq_ignore_ascii_case(b"LDR") => Self::LoadOrStore(LoadOrStoreMnemonic::Ldr),
            inst if inst.eq_ignore_ascii_case(b"LDRB") => Self::LoadOrStore(LoadOrStoreMnemonic::Ldrb),
            inst if inst.eq_ignore_ascii_case(b"LDRH") => Self::LoadOrStore(LoadOrStoreMnemonic::Ldrh),
            inst if inst.eq_ignore_ascii_case(b"LDRW") => Self::LoadOrStore(LoadOrStoreMnemonic::Ldrw),
            inst if inst.eq_ignore_ascii_case(b"LDRD") => Self::LoadOrStore(LoadOrStoreMnemonic::Ldrd),
            inst if inst.eq_ignore_ascii_case(b"LDRQ") => Self::LoadOrStore(LoadOrStoreMnemonic::Ldrq),
            inst if inst.eq_ignore_ascii_case(b"MOV") => Self::RegOperand(RegOperandMnemonic::Mov),
            inst if inst.eq_ignore_ascii_case(b"MUL") => Self::RegOperand(RegOperandMnemonic::Mul),
            inst if inst.eq_ignore_ascii_case(b"MULS") => Self::RegOperand(RegOperandMnemonic::MulS),
            inst if inst.eq_ignore_ascii_case(b"NOP") => Self::NoArg(NoArgMnemonic::Nop),
            inst if inst.eq_ignore_ascii_case(b"NOT") => Self::SingleReg(SingleRegMnemonic::Not),
            inst if inst.eq_ignore_ascii_case(b"OR") => Self::RegOperand(RegOperandMnemonic::Or),
            inst if inst.eq_ignore_ascii_case(b"POP") => Self::SingleReg(SingleRegMnemonic::Pop),
            inst if inst.eq_ignore_ascii_case(b"PUSH") => Self::SingleOperand(SingleOperandMnemonic::Push),
            inst if inst.eq_ignore_ascii_case(b"RET") => Self::NoArg(NoArgMnemonic::Ret),
            inst if inst.eq_ignore_ascii_case(b"ROL") => Self::Rotate(RotateMnemonic::Rol),
            inst if inst.eq_ignore_ascii_case(b"ROR") => Self::Rotate(RotateMnemonic::Ror),
            inst if inst.eq_ignore_ascii_case(b"SHL") => Self::Shift(ShiftMnemonic::Shl),
            inst if inst.eq_ignore_ascii_case(b"SHR") => Self::Shift(ShiftMnemonic::Shr),
            inst if inst.eq_ignore_ascii_case(b"ASR") => Self::Shift(ShiftMnemonic::Asr),
            inst if inst.eq_ignore_ascii_case(b"STR") => Self::LoadOrStore(LoadOrStoreMnemonic::Str),
            inst if inst.eq_ignore_ascii_case(b"STRB") => Self::LoadOrStore(LoadOrStoreMnemonic::Strb),
            inst if inst.eq_ignore_ascii_case(b"STRH") => Self::LoadOrStore(LoadOrStoreMnemonic::Strh),
            inst if inst.eq_ignore_ascii_case(b"STRW") => Self::LoadOrStore(LoadOrStoreMnemonic::Strw),
            inst if inst.eq_ignore_ascii_case(b"STRD") => Self::LoadOrStore(LoadOrStoreMnemonic::Strd),
            inst if inst.eq_ignore_ascii_case(b"STRQ") => Self::LoadOrStore(LoadOrStoreMnemonic::Strq),
            inst if inst.eq_ignore_ascii_case(b"SUB") => Self::RegOperand(RegOperandMnemonic::Sub),
            inst if inst.eq_ignore_ascii_case(b"SUBS") => Self::RegOperand(RegOperandMnemonic::SubS),
            inst if inst.eq_ignore_ascii_case(b"XOR") => Self::RegOperand(RegOperandMnemonic::Xor),
            _ => return Err(()),
        };

        Ok(inst)
    }
}
