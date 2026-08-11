use core::num::ParseIntError;
use std::{collections::HashMap, marker::PhantomData, num::TryFromIntError};

use procem::register::{Register, RegisterError};
use thiserror::Error;

use crate::instruction::asm_instruction::{ASMLoadOrStoreInstruction, ASMRegLabelInstruction};
use crate::instruction::memory_location::MemoryLocation;
use crate::instruction::operand::Operand;
use crate::instruction::{Instruction, asm_instruction::ASMNoArgInstruction};
use crate::instruction::{
    asm_instruction::{
        ASMInstruction, ASMJumpInstruction, ASMRegOperandInstruction, ASMRotateInstruction, ASMShiftInstruction,
        ASMSingleOperandInstruction, ASMSingleRegInstruction, ASMTwoOperandInstruction,
    },
    unlinked::UnlinkedInstruction,
};
use crate::tokenizer::{ImmediateLiteral, Token};
use ars::range::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Parsed<'input> {
    instructions: Vec<Instruction>,
    labels: HashMap<&'input [u8], u64>,
    unlinked_instructions: Vec<UnlinkedInstruction>,
    data: Vec<u8>,
    bss: u64,
}

impl Parsed<'_> {
    // Returns vec because linker uses size attribute
    #[inline]
    #[must_use]
    pub(crate) const fn mut_instructions(&mut self) -> &mut Vec<Instruction> {
        &mut self.instructions
    }

    #[inline]
    #[must_use]
    pub(crate) const fn labels(&self) -> &HashMap<&[u8], u64> {
        &self.labels
    }

    // Returns vec because linker uses the size attribute
    #[inline]
    #[must_use]
    pub(crate) const fn mut_unlinked_instructions(&mut self) -> &mut Vec<UnlinkedInstruction> {
        &mut self.unlinked_instructions
    }

    #[inline]
    #[must_use]
    pub(crate) fn data(&self) -> &[u8] {
        &self.data
    }

    #[inline]
    #[must_use]
    pub(crate) const fn mut_data(&mut self) -> &mut Vec<u8> {
        &mut self.data
    }

    #[inline]
    #[must_use]
    pub(crate) const fn bss(&self) -> u64 {
        self.bss
    }
}

impl<'input, Section> From<InnerParser<'input, Section>> for Parsed<'input> {
    fn from(p: InnerParser<'input, Section>) -> Self {
        Self {
            instructions: p.instructions,
            labels: p.labels,
            unlinked_instructions: p.unlinked_instructions,
            data: p.data,
            bss: p.bss,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bss;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undefined;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Section {
    Code,
    Data,
    Bss,
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parser<'input> {
    Undefined(InnerParser<'input, Undefined>),
    Code(InnerParser<'input, Code>),
    Data(InnerParser<'input, Data>),
    Bss(InnerParser<'input, Bss>),
}

impl<'input> Parser<'input> {
    #[inline]
    #[must_use]
    fn new(tokens: &'input [Token], input: &'input [u8]) -> Self {
        Self::Undefined(InnerParser {
            tokens,
            instructions: Vec::with_capacity(tokens.len() / 3), // instructions most often are 4 tokens long, to balance out shorter ones 3 is used,
            labels: HashMap::default(),
            unlinked_instructions: Vec::default(),
            data: Vec::default(),
            bss: 0,
            errors: None,
            idx: 0,
            input,
            end_parsing: false,
            _current_section: PhantomData,
        })
    }

    /// Parse tokens into a list of instructions.
    ///
    /// # Errors
    /// Returns a list of errors that occurred during parsing.
    pub(crate) fn parse(tokens: &'input [Token], input: &'input [u8]) -> Result<Parsed<'input>, Vec<ParserError>> {
        let mut parser = Self::new(tokens, input);

        while !parser.is_done() {
            parser = parser.step();
        }
        parser.finish()
    }

    #[must_use]
    fn step(self) -> Self {
        let current_token = match &self {
            Self::Undefined(p) => p.get_token().copied(),
            Self::Code(p) => p.get_token().copied(),
            Self::Data(p) => p.get_token().copied(),
            Self::Bss(p) => p.get_token().copied(),
        }
        .expect("This function is never called with an invalid idx {}");

        match current_token {
            Token::Directive(range) => self.change_section(range),
            Token::Newline => match self {
                Self::Undefined(mut p) => {
                    p.idx += 1;
                    Self::Undefined(p)
                }
                Self::Code(mut p) => {
                    p.idx += 1;
                    Self::Code(p)
                }
                Self::Data(mut p) => {
                    p.idx += 1;
                    Self::Data(p)
                }
                Self::Bss(mut p) => {
                    p.idx += 1;
                    Self::Bss(p)
                }
            },
            t => match self {
                Self::Undefined(mut p) => {
                    p.add_error(ParserError::InvalidToken {
                        idx: p.idx,
                        expected: "Section Directive",
                        got: t,
                    });
                    p.idx += 1;
                    Self::Undefined(p)
                }
                Self::Code(mut p) => {
                    p.parse_next_token();
                    p.idx += 1;
                    Self::Code(p)
                }
                Self::Data(mut p) => {
                    p.parse_next_token();
                    p.idx += 1;
                    Self::Data(p)
                }
                Self::Bss(mut p) => {
                    p.parse_next_token();
                    p.idx += 1;
                    Self::Bss(p)
                }
            },
        }
    }

    #[inline]
    const fn is_done(&self) -> bool {
        match self {
            Self::Undefined(p) => p.end_parsing || p.idx >= p.tokens.len(),
            Self::Code(p) => p.end_parsing || p.idx >= p.tokens.len(),
            Self::Data(p) => p.end_parsing || p.idx >= p.tokens.len(),
            Self::Bss(p) => p.end_parsing || p.idx >= p.tokens.len(),
        }
    }

    #[inline]
    fn finish(self) -> Result<Parsed<'input>, Vec<ParserError>> {
        match self {
            Self::Undefined(p) => {
                if let Some(errors) = p.errors {
                    Err(errors)
                } else {
                    Ok(Parsed::from(p))
                }
            }
            Self::Code(p) => {
                if let Some(errors) = p.errors {
                    Err(errors)
                } else {
                    Ok(Parsed::from(p))
                }
            }
            Self::Data(p) => {
                if let Some(errors) = p.errors {
                    Err(errors)
                } else {
                    Ok(Parsed::from(p))
                }
            }
            Self::Bss(p) => {
                if let Some(errors) = p.errors {
                    Err(errors)
                } else {
                    Ok(Parsed::from(p))
                }
            }
        }
    }

    #[must_use]
    fn change_section(self, range: Range) -> Self {
        macro_rules! change_and_advance {
            ($parser:expr, $variant:ident) => {{
                let mut next = $parser.transition();
                next.idx += 1;

                Self::$variant(next)
            }};
        }

        macro_rules! error_and_advance {
            ($parser:expr, $variant:ident, $got: expr) => {{
                $parser.add_error(ParserError::InvalidSection {
                    idx: $parser.idx,
                    identifier: $got,
                });
                $parser.idx += 1;
                Self::$variant($parser)
            }};
        }

        macro_rules! change_section {
            ($variant:ident) => {
                match self {
                    Self::Undefined(p) => change_and_advance!(p, $variant),
                    Self::Code(p) => change_and_advance!(p, $variant),
                    Self::Data(p) => change_and_advance!(p, $variant),
                    Self::Bss(p) => change_and_advance!(p, $variant),
                }
            };
        }

        let section = match &self {
            Self::Undefined(p) => Self::parse_section_directive(p, range),
            Self::Code(p) => Self::parse_section_directive(p, range),
            Self::Data(p) => Self::parse_section_directive(p, range),
            Self::Bss(p) => Self::parse_section_directive(p, range),
        };

        match section {
            Section::Code => change_section!(Code),
            Section::Data => change_section!(Data),
            Section::Bss => change_section!(Bss),
            Section::Invalid(directive) => match self {
                Self::Undefined(mut p) => error_and_advance!(p, Undefined, directive), // No directives other then sections allowed when section is still undefined
                Self::Code(mut p) => error_and_advance!(p, Code, directive), // No directives allowed inside code sections
                Self::Data(mut p) => {
                    // directives allowed inside data sections
                    p.parse_next_token();
                    p.idx += 1;
                    Self::Data(p)
                }
                Self::Bss(mut p) => {
                    // directives allowed inside bss sections
                    p.parse_next_token();
                    p.idx += 1;
                    Self::Bss(p)
                }
            },
        }
    }

    #[inline]
    #[must_use]
    fn parse_section_directive<S>(parser: &InnerParser<'_, S>, range: Range) -> Section {
        match &parser.input[range] {
            directive if directive.eq_ignore_ascii_case(b"code") => Section::Code,
            directive if directive.eq_ignore_ascii_case(b"data") => Section::Data,
            directive if directive.eq_ignore_ascii_case(b"bss") => Section::Bss,
            directive => Section::Invalid(string_from_u8_slice(directive)),
        }
    }
}

macro_rules! from_literal {
    ($unsigned:ty, $fn_name: ident) => {
        #[doc = concat!("Parse an `ImmediateLiteral` into an `", stringify!($unsigned), "`.")]
        fn $fn_name(&self, lit: ImmediateLiteral) -> Result<$unsigned, ParserError> {
            match lit {
                ImmediateLiteral::Char(c) => Ok(<$unsigned>::from(c)),
                ImmediateLiteral::Binary(range) => {
                    let lit = String::from_utf8_lossy(&self.input[range]);
                    <$unsigned>::from_str_radix(&lit, 2).map_err(|err| ParserError::LiteralParsing {
                        lit: lit.to_string(),
                        err,
                    })
                }
                ImmediateLiteral::Decimal(range) => {
                    let lit = String::from_utf8_lossy(&self.input[range]);
                    if let Some(lit) = lit.strip_prefix('-') {
                        lit.parse::<$unsigned>().map(<$unsigned>::wrapping_neg)
                    } else {
                        lit.parse()
                    }
                    .map_err(|err| ParserError::LiteralParsing {
                        lit: lit.to_string(),
                        err,
                    })
                }
                ImmediateLiteral::Hexadecimal(range) => {
                    let lit = String::from_utf8_lossy(&self.input[range]);
                    <$unsigned>::from_str_radix(&lit, 16).map_err(|err| ParserError::LiteralParsing {
                        lit: lit.to_string(),
                        err,
                    })
                }
                ImmediateLiteral::Octal(range) => {
                    let lit = String::from_utf8_lossy(&self.input[range]);
                    <$unsigned>::from_str_radix(&lit, 8).map_err(|err| ParserError::LiteralParsing {
                        lit: lit.to_string(),
                        err,
                    })
                }
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerParser<'input, Section = Undefined> {
    tokens: &'input [Token],
    instructions: Vec<Instruction>,
    labels: HashMap<&'input [u8], u64>,
    unlinked_instructions: Vec<UnlinkedInstruction>,
    data: Vec<u8>,
    bss: u64,
    errors: Option<Vec<ParserError>>,
    idx: usize,
    input: &'input [u8],
    end_parsing: bool,
    _current_section: PhantomData<Section>,
}

impl<'input, Section> InnerParser<'input, Section> {
    #[must_use]
    fn transition<S>(self) -> InnerParser<'input, S> {
        InnerParser {
            tokens: self.tokens,
            instructions: self.instructions,
            labels: self.labels,
            unlinked_instructions: self.unlinked_instructions,
            data: self.data,
            bss: self.bss,
            errors: self.errors,
            idx: self.idx,
            input: self.input,
            end_parsing: self.end_parsing,
            _current_section: PhantomData,
        }
    }

    /// Returns the current token.
    #[inline]
    fn get_token(&self) -> Option<&'_ Token> {
        self.tokens.get(self.idx)
    }

    /// Returns the next token if available.
    #[inline]
    fn peek_token(&self) -> Option<&'_ Token> {
        self.tokens.get(self.idx + 1)
    }

    #[inline]
    fn add_error(&mut self, err: ParserError) {
        self.errors.get_or_insert_default().push(err);
        self.skip_to_next_line();
    }

    #[inline]
    fn skip_to_next_line(&mut self) {
        self.idx += 1;
        while let Some(token) = self.get_token() {
            match token {
                Token::Newline => break,
                Token::End => {
                    self.end_parsing = true;
                    break;
                }
                _ => {}
            }
            self.idx += 1;
        }
    }

    from_literal! {u8,  u8_from_literal}
    from_literal! {u16, u16_from_literal}
    from_literal! {u32, u32_from_literal}
    from_literal! {u64, u64_from_literal}
    from_literal! {u128, u128_from_literal}

    fn expect_immediate_literal(&mut self) -> Result<ImmediateLiteral, ParserError> {
        self.idx += 1; // manual, to enable borrow of self inside match
        match self.tokens.get(self.idx) {
            Some(token) => match token {
                Token::ImmediateLiteral(lit) => Ok(*lit),
                token => Err(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "ImmediateLiteral",
                    got: *token,
                }),
            },
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }
}

impl InnerParser<'_, Code> {
    fn parse_next_token(&mut self) {
        match &self.tokens.get(self.idx) {
            Some(t) => match t {
                Token::Identifier(range) => {
                    // At the start of a new line only labels and instructions are valid identifiers
                    let res = if let Some(token) = self.peek_token()
                        && *token == Token::Colon
                    {
                        self.parse_label(*range)
                    } else {
                        self.parse_instruction(&self.input[range])
                    };
                    if let Err(err) = res {
                        self.add_error(err);
                    }
                }
                Token::Newline => {}
                Token::End => self.end_parsing = true,
                token => self.add_error(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "Label, Instruction or End",
                    got: **token,
                }),
            },
            None => unreachable!("self.tokens is never indexed with an invalid idx."),
        }
    }

    fn parse_label(&mut self, range: Range) -> Result<(), ParserError> {
        if let Some(old_instruction_idx) = self.labels.insert(&self.input[range], self.instructions.len() as u64) {
            Err(ParserError::DuplicateLabel {
                idx: self.instructions.len(),
                old_idx: old_instruction_idx,
            })?;
        }
        self.idx += 1; // Skip the colon after label
        Ok(())
    }

    fn parse_instruction(&mut self, instruction: &[u8]) -> Result<(), ParserError> {
        let inst = instruction.try_into().map_err(|_| ParserError::UnknownInstruction {
            idx: self.idx,
            inst: string_from_u8_slice(instruction),
        })?;
        match inst {
            ASMInstruction::NoArg(inst) => {
                self.instructions.push(match inst {
                    ASMNoArgInstruction::Nop => Instruction::Nop,
                    ASMNoArgInstruction::Ret => Instruction::Ret,
                });
                Ok(())
            }
            ASMInstruction::RegLabel(inst) => self.expect_reg_label_instruction(inst),
            ASMInstruction::RegOperand(inst) => self.expect_reg_operand_instruction(inst),
            ASMInstruction::Jump(inst) => self.expect_destination(inst),
            ASMInstruction::TwoOperand(inst) => self.expect_two_operand_instruction(inst),
            ASMInstruction::SingleOperand(inst) => self.expect_single_operand_instruction(inst),
            ASMInstruction::SingleReg(inst) => self.expect_single_reg_instruction(inst),
            ASMInstruction::Rotate(inst) => self.expect_rotate_instruction(inst),
            ASMInstruction::Shift(inst) => self.expect_shift_instruction(inst),
            ASMInstruction::LoadOrStore(inst) => self.expect_load_or_store_instruction(inst),
        }
    }

    fn expect_destination(&mut self, instr: ASMJumpInstruction) -> Result<(), ParserError> {
        self.idx += 1;

        match self.tokens.get(self.idx) {
            Some(Token::Identifier(range)) => {
                self.unlinked_instructions
                    .push(UnlinkedInstruction::new(self.instructions.len(), *range));
                self.instructions
                    .push(Instruction::from_jump_instruction(instr, u64::MAX));
                Ok(())
            }
            Some(token) => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Identifier (Label)",
                got: *token,
            }),
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }

    fn expect_register(&mut self) -> Result<Register, ParserError> {
        self.idx += 1; // manual, to enable borrow of self inside match
        match self.tokens.get(self.idx) {
            Some(Token::Identifier(range)) => {
                Register::try_from(&self.input[range]).map_err(|err| ParserError::RegisterParsing { err })
            }
            Some(token) => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Register",
                got: *token,
            }),
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }

    fn expect_comma(&mut self) -> Result<(), ParserError> {
        self.idx += 1;

        match self.tokens.get(self.idx) {
            Some(Token::Comma) => Ok(()),
            Some(token) => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Comma",
                got: *token,
            }),
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }

    fn expect_operand(&mut self) -> Result<Operand, ParserError> {
        self.idx += 1; // manual, to enable borrow of self inside match
        match self.tokens.get(self.idx) {
            Some(Token::Identifier(range)) => Ok(Operand::Register(
                Register::try_from(&self.input[range]).map_err(|err| ParserError::RegisterParsing { err })?,
            )),
            Some(Token::ImmediateLiteral(lit)) => Ok(Operand::Value(self.u64_from_literal(*lit)?)),
            Some(token) => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Identifier (Register) or Literal",
                got: *token,
            }),
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }

    // _isntr may be used in future if there are other instructions like adr
    fn expect_reg_label_instruction(&mut self, _instr: ASMRegLabelInstruction) -> Result<(), ParserError> {
        let reg = self.expect_register()?;
        self.expect_comma()?;
        self.idx += 1;

        match self.tokens.get(self.idx) {
            Some(Token::Identifier(range)) => {
                self.unlinked_instructions
                    .push(UnlinkedInstruction::new(self.instructions.len(), *range));
                self.instructions.push(Instruction::Adr { reg, addr: u64::MAX });
                Ok(())
            }
            Some(token) => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Identifier (Label)",
                got: *token,
            }),
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }

    fn expect_reg_operand_instruction(&mut self, instr: ASMRegOperandInstruction) -> Result<(), ParserError> {
        let acc = self.expect_register()?;
        self.expect_comma()?;
        let operand = self.expect_operand()?;
        self.instructions
            .push(Instruction::from_reg_operand_instruction(instr, acc, operand));
        Ok(())
    }

    fn expect_single_reg_instruction(&mut self, instr: ASMSingleRegInstruction) -> Result<(), ParserError> {
        let reg = self.expect_register()?;
        self.instructions
            .push(Instruction::from_single_reg_instruction(instr, reg));
        Ok(())
    }

    fn expect_single_operand_instruction(&mut self, instr: ASMSingleOperandInstruction) -> Result<(), ParserError> {
        let operand = self.expect_operand()?;
        self.instructions
            .push(Instruction::from_single_operand_instruction(instr, operand));
        Ok(())
    }

    fn expect_two_operand_instruction(&mut self, instr: ASMTwoOperandInstruction) -> Result<(), ParserError> {
        let lhs = self.expect_operand()?;
        self.expect_comma()?;
        let rhs = self.expect_operand()?;
        self.instructions
            .push(Instruction::from_two_operand_instruction(instr, lhs, rhs));
        Ok(())
    }

    fn expect_shift_instruction(&mut self, instr: ASMShiftInstruction) -> Result<(), ParserError> {
        let register = self.expect_register()?;
        self.expect_comma()?;
        let literal = self.expect_immediate_literal()?;
        let word = self.u64_from_literal(literal)?;
        self.instructions
            .push(Instruction::from_shift_instruction(instr, register, word));
        Ok(())
    }

    fn expect_rotate_instruction(&mut self, instr: ASMRotateInstruction) -> Result<(), ParserError> {
        let register = self.expect_register()?;
        self.expect_comma()?;
        let literal = self.expect_immediate_literal()?;
        let literal = self.u64_from_literal(literal)?;
        let literal: u32 = literal
            .try_into()
            .map_err(|err| ParserError::CannotConvertLiteralToU32 { literal, err })?;

        self.instructions
            .push(Instruction::from_rotate_instruction(instr, register, literal));
        Ok(())
    }

    fn expect_load_or_store_instruction(&mut self, instr: ASMLoadOrStoreInstruction) -> Result<(), ParserError> {
        let reg = self.expect_register()?;
        self.expect_comma()?;
        self.idx += 1;

        let mem_location = match self.tokens.get(self.idx) {
            Some(token) => match *token {
                Token::Identifier(range) => Ok(self.create_labeled_mem_location(range)),
                Token::OpenBracket => self.expect_direct_mem_location(),
                token => Err(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "Label or Memory Location",
                    got: token,
                }),
            }?,
            None => Err(ParserError::TokenNotFound { idx: self.idx })?,
        };

        let instr = Instruction::from_ldr_or_str_instruction(instr, reg, mem_location);

        self.instructions.push(instr);
        Ok(())
    }

    fn create_labeled_mem_location(&mut self, range: Range) -> MemoryLocation {
        self.unlinked_instructions
            .push(UnlinkedInstruction::new(self.instructions.len(), range));
        MemoryLocation::Labeled(u64::MAX)
    }

    fn expect_direct_mem_location(&mut self) -> Result<MemoryLocation, ParserError> {
        let base = self.expect_register()?;

        let offset = if let Some(Token::Comma) = self.peek_token() {
            self.idx += 1; // skip comma
            self.expect_operand()?
        } else {
            Operand::Value(0)
        };

        self.idx += 1;
        match self.tokens.get(self.idx) {
            Some(token) => match token {
                Token::ClosedBracket => Ok(MemoryLocation::Offset { base, offset }),
                token => Err(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "Closed Bracket",
                    got: *token,
                }),
            },
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }
}

impl InnerParser<'_, Data> {
    fn parse_next_token(&mut self) {
        match &self.tokens[self.idx] {
            Token::Identifier(range) => {
                if let Some(old_data_idx) = self.labels.insert(&self.input[range], self.data.len() as u64) {
                    self.add_error(ParserError::DuplicateLabel {
                        idx: self.instructions.len(),
                        old_idx: old_data_idx,
                    });
                }
                self.idx += 1; // Skip colon after label
            }
            Token::Directive(directive) => {
                if let Err(err) = self.parse_directive(&self.input[directive]) {
                    self.add_error(err);
                }
            }
            Token::Newline => {}
            Token::End => self.end_parsing = true,
            token => self.add_error(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Identifier (Label), Directive, or End Token",
                got: *token,
            }),
        }
    }

    fn parse_directive(&mut self, directive: &[u8]) -> Result<(), ParserError> {
        match directive {
            directive if directive.eq_ignore_ascii_case(b"byte") => self.parse_bytes(),
            directive if directive.eq_ignore_ascii_case(b"hword") => self.parse_hwords(),
            directive if directive.eq_ignore_ascii_case(b"word") => self.parse_words(),
            directive if directive.eq_ignore_ascii_case(b"dword") => self.parse_dwords(),
            directive if directive.eq_ignore_ascii_case(b"qword") => self.parse_qwords(),
            directive if directive.eq_ignore_ascii_case(b"ascii") => self.parse_ascii(),
            directive => Err(ParserError::InvalidDirective {
                idx: self.idx,
                directive: string_from_u8_slice(directive),
                expected: "Only .byte, .hword, .word, .dword, .qword and .ascii are allowed in .data sections."
                    .to_string(),
            }),
        }
    }

    #[inline]
    fn parse_bytes(&mut self) -> Result<(), ParserError> {
        self.expect_byte()?;

        while let Some(Token::Comma) = self.peek_token() {
            self.idx += 1;
            self.expect_byte()?;
        }
        Ok(())
    }

    #[inline]
    fn parse_hwords(&mut self) -> Result<(), ParserError> {
        self.expect_hword()?;

        while let Some(Token::Comma) = self.peek_token() {
            self.idx += 1;
            self.expect_hword()?;
        }
        Ok(())
    }

    #[inline]
    fn parse_words(&mut self) -> Result<(), ParserError> {
        self.expect_word()?;

        while let Some(Token::Comma) = self.peek_token() {
            self.idx += 1;
            self.expect_word()?;
        }
        Ok(())
    }

    #[inline]
    fn parse_dwords(&mut self) -> Result<(), ParserError> {
        self.expect_dword()?;

        while let Some(Token::Comma) = self.peek_token() {
            self.idx += 1;
            self.expect_dword()?;
        }
        Ok(())
    }

    #[inline]
    fn parse_qwords(&mut self) -> Result<(), ParserError> {
        self.expect_qword()?;

        while let Some(Token::Comma) = self.peek_token() {
            self.idx += 1;
            self.expect_qword()?;
        }
        Ok(())
    }

    #[inline]
    fn expect_byte(&mut self) -> Result<(), ParserError> {
        let lit = self.expect_immediate_literal()?;
        let byte = self.u8_from_literal(lit)?;
        self.data.extend_from_slice(&byte.to_le_bytes());
        Ok(())
    }

    #[inline]
    fn expect_hword(&mut self) -> Result<(), ParserError> {
        let lit = self.expect_immediate_literal()?;
        let hword = self.u16_from_literal(lit)?;
        self.data.extend_from_slice(&hword.to_le_bytes());
        Ok(())
    }

    #[inline]
    fn expect_word(&mut self) -> Result<(), ParserError> {
        let lit = self.expect_immediate_literal()?;
        let word = self.u32_from_literal(lit)?;
        self.data.extend_from_slice(&word.to_le_bytes());
        Ok(())
    }

    #[inline]
    fn expect_dword(&mut self) -> Result<(), ParserError> {
        let lit = self.expect_immediate_literal()?;
        let dword = self.u64_from_literal(lit)?;
        self.data.extend_from_slice(&dword.to_le_bytes());
        Ok(())
    }

    #[inline]
    fn expect_qword(&mut self) -> Result<(), ParserError> {
        let lit = self.expect_immediate_literal()?;
        let qword = self.u128_from_literal(lit)?;
        self.data.extend_from_slice(&qword.to_le_bytes());
        Ok(())
    }

    #[inline]
    fn parse_ascii(&mut self) -> Result<(), ParserError> {
        self.expect_string_literal()?;

        while let Some(Token::Comma) = self.peek_token() {
            self.idx += 1;
            self.expect_string_literal()?;
        }
        Ok(())
    }

    fn expect_string_literal(&mut self) -> Result<(), ParserError> {
        self.idx += 1;
        let token = self.tokens.get(self.idx);

        match token {
            Some(token) => match token {
                Token::StringLiteral(lit) => {
                    self.data.extend(self.input[lit].iter().copied());
                    Ok(())
                }
                token => Err(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "ImmediateLiteral",
                    got: *token,
                }),
            },
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }
}

impl InnerParser<'_, Bss> {
    fn parse_next_token(&mut self) {
        match &self.tokens[self.idx] {
            Token::Identifier(range) => {
                if let Some(old_bss_idx) = self.labels.insert(&self.input[range], self.bss) {
                    self.add_error(ParserError::DuplicateLabel {
                        idx: self.instructions.len(),
                        old_idx: old_bss_idx,
                    });
                }
                self.idx += 1; // Skip colon after label
            }
            Token::Directive(section) => {
                if let Err(err) = self.parse_directive(&self.input[section]) {
                    self.add_error(err);
                }
            }
            Token::Newline => {}
            Token::End => self.end_parsing = true,
            token => self.add_error(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Identifier (Label), Directive, or End Token",
                got: *token,
            }),
        }
    }

    #[inline]
    fn parse_directive(&mut self, directive: &[u8]) -> Result<(), ParserError> {
        match directive {
            directive if directive.eq_ignore_ascii_case(b"space") => self.parse_space(),
            directive => Err(ParserError::InvalidDirective {
                idx: self.idx,
                directive: string_from_u8_slice(directive),
                expected: "Only .space is allowed in .bss sections.".to_string(),
            }),
        }
    }

    #[inline]
    fn parse_space(&mut self) -> Result<(), ParserError> {
        self.expect_space()?;

        while let Some(Token::Comma) = self.peek_token() {
            self.idx += 1;
            self.expect_space()?;
        }
        Ok(())
    }

    #[inline]
    fn expect_space(&mut self) -> Result<(), ParserError> {
        let lit = self.expect_immediate_literal()?;
        let space = self.u64_from_literal(lit)?;
        self.bss += space;
        Ok(())
    }
}

#[inline]
#[must_use]
fn string_from_u8_slice(slice: &[u8]) -> String {
    String::from_utf8_lossy(slice).to_string()
}

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ParserError {
    #[error("No tokens to parse.")]
    EmptyTokenList,
    #[error("Invalid token at idx {idx}. Expected: {expected} Got: {got}")]
    InvalidToken {
        idx: usize,
        expected: &'static str,
        got: Token,
    },
    #[error("Duplicate label: First occurrence: {old_idx}, second occurrence {idx}")]
    DuplicateLabel { idx: usize, old_idx: u64 },
    #[error("Unknown instruction at idx {idx}: {inst}")]
    UnknownInstruction { idx: usize, inst: String },
    #[error("Error while parsing register: {err}")]
    RegisterParsing {
        #[from]
        err: RegisterError,
    },
    #[error("Error while parsing literal ({lit}): {err}")]
    LiteralParsing { lit: String, err: ParseIntError },
    #[error("Strings cannot be converted to numeric values directly. You could use a hex representation instead.")]
    CannotConvertStrToVal,
    #[error("Cannot convert literal {literal} to u32. This is likely due to the literal being too large.\n{err}")]
    CannotConvertLiteralToU32 { literal: u64, err: TryFromIntError },

    #[error("Invalid section identifier: {identifier} at {idx}.")]
    InvalidSection { idx: usize, identifier: String },
    #[error("Invalid directive {directive} at {idx}: {expected}.")]
    InvalidDirective {
        idx: usize,
        directive: String,
        expected: String,
    },
    #[error("Expected Literal at idx {idx} but got nothing.")]
    TokenNotFound { idx: usize },
}

#[cfg(test)]
mod test {
    use crate::{
        instruction::{Instruction, memory_location::MemoryLocation, operand::Operand},
        parser::{Parser, ParserError},
        tokenizer::Tokenizer,
    };
    use pretty_assertions_sorted::assert_eq;
    use procem::register::Register;

    #[test]
    fn parse_section() {
        let input = b"
            .code
            .bss
            .data
            .Bss
            .CODE
            .Invalid
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let mut p = Parser::new(&tokens, input);

        macro_rules! check {
            ($variant:ident, $p:expr) => {
                match p {
                    Parser::$variant(_) => assert!(true),
                    p => panic!("Expected Parser variant {}, got: {p:?}", stringify!($variant)),
                }
            };
        }

        check!(Undefined, p);
        p = p.step(); // skip Newline
        p = p.step();
        check!(Code, p);
        p = p.step(); // skip Newline
        p = p.step();
        check!(Bss, p);
        p = p.step(); // skip Newline
        p = p.step();
        check!(Data, p);
        p = p.step(); // skip Newline
        p = p.step();
        check!(Bss, p);
        p = p.step(); // skip Newline
        p = p.step();
        check!(Code, p);
        p = p.step(); // skip Newline
        p = p.step();
        check!(Code, p);
        match p {
            Parser::Code(p) => assert_eq!(
                p.errors.unwrap()[0],
                ParserError::InvalidSection {
                    idx: 11,
                    identifier: "Invalid".to_string()
                }
            ),
            _ => unreachable!("check! before ensures, that Code is the Variant"),
        }
    }

    #[test]
    fn parse_bss() {
        let input = b"
            .bss
                a:
                .space 5
                .space 10
                b: .space 5, 0xA
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = Parser::parse(&tokens, input).unwrap();

        assert_eq!(parsed.instructions.len(), 0);
        assert_eq!(parsed.labels().len(), 2);
        assert_eq!(parsed.data().len(), 0);
        assert_eq!(parsed.bss, 5 + 10 + 5 + 10);
        assert_eq!(parsed.labels[b"a".as_slice()], 0);
        assert_eq!(parsed.labels[b"b".as_slice()], 5 + 10);
    }

    #[test]
    fn parse_data() {
        let input = b"
            .data
                a:
                    .byte 5
                    .hword 5
                    .word 5
                    .dword 5
                    .qword 5
                b:
                    .ascii \"Hello World!\", \"\0\"
                c:
                    .word 5, 0xA
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = Parser::parse(&tokens, input).unwrap();

        assert_eq!(parsed.instructions.len(), 0);
        assert_eq!(parsed.unlinked_instructions.len(), 0);
        assert_eq!(parsed.bss(), 0);
        assert_eq!(
            parsed.data.len(),
            1 + 2 + 4 + 8 + 16 + b"Hello World!".len() + b"\0".len() + 4 + 4
        ); // byte, hword, word, dword, qword, 2 ascii, 2 word allocations
        assert_eq!(parsed.labels.len(), 3);
        assert_eq!(parsed.labels[b"a".as_slice()], 0);
        assert_eq!(parsed.labels[b"b".as_slice()], 1 + 2 + 4 + 8 + 16);
        assert_eq!(
            parsed.labels[b"c".as_slice()],
            1 + 2 + 4 + 8 + 16 + b"Hello World!".len() as u64 + b"\0".len() as u64
        );
    }

    #[test]
    fn parse_data_multi_alloc() {
        let input = b"
            .data
                a:
                    .byte 5,5
                    .hword 5, 5
                    .word 5, 5,5
                    .dword 5, 5
                    .qword 5, 5
                b:
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = Parser::parse(&tokens, input).unwrap();

        assert_eq!(parsed.instructions.len(), 0);
        assert_eq!(parsed.unlinked_instructions.len(), 0);
        assert_eq!(parsed.bss(), 0);
        assert_eq!(parsed.data.len(), 1 + 1 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 16 + 16); // 2 byte, 2 hword, 3 word, 2 dword, 2 qword
        assert_eq!(parsed.labels.len(), 2);
        assert_eq!(parsed.labels[b"a".as_slice()], 0);
        assert_eq!(
            parsed.labels[b"b".as_slice()],
            1 + 1 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 16 + 16
        );
    }

    #[test]
    fn parse_all_sections() {
        let input = b"
            .code
            a:
                mov R1, 0
                jmp g

            .bss
            b:
                .space 5, 0xA

            .code
            c:
                mov R0, R1
                jmp a

            .data
            d:
                .word 2

            .bss
                .space 5

            .data
            e:
                .word 8

            .code
            f:
                add R1, 0o1
                jmp c
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = Parser::parse(&tokens, input).unwrap();

        assert_eq!(parsed.instructions.len(), 6);
        assert_eq!(parsed.labels().len(), 6);
        assert_eq!(parsed.data().len(), 4 + 4); // 2 32bit allocations
        assert_eq!(parsed.bss(), 5 + 10 + 5);

        // code
        assert_eq!(parsed.labels()[b"a".as_slice()], 0);
        assert_eq!(parsed.labels()[b"c".as_slice()], 2);
        assert_eq!(parsed.labels()[b"f".as_slice()], 4);
        // bss
        assert_eq!(parsed.labels()[b"b".as_slice()], 0);
        // data
        assert_eq!(parsed.labels()[b"d".as_slice()], 0);
        assert_eq!(parsed.labels()[b"e".as_slice()], 4);
    }

    #[test]
    fn parse_str_instr() {
        let input = b"
            .code
            str r0, [r1]
            str r0, [r1, r2]
            str r0, [r1, 5]
            str r0, [r1, -1]
            str r0, [r1, 0xa]
            str r0, data
            ";

        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = Parser::parse(&tokens, input).unwrap();

        assert_eq!(parsed.instructions.len(), 6);
        assert_eq!(parsed.unlinked_instructions.len(), 1);
        assert_eq!(&input[parsed.unlinked_instructions[0].label()], b"data".as_slice());

        let mut insts = parsed.instructions.iter();

        match insts.next().unwrap() {
            Instruction::Str { from, to } => {
                assert_eq!(*from, Register::R0);
                assert_eq!(
                    *to,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(0)
                    }
                );
            }
            i => unreachable!("Expected Str instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Str { from, to } => {
                assert_eq!(*from, Register::R0);
                assert_eq!(
                    *to,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Register(Register::R2)
                    }
                );
            }
            i => unreachable!("Expected Str instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Str { from, to } => {
                assert_eq!(*from, Register::R0);
                assert_eq!(
                    *to,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(5)
                    }
                );
            }
            i => unreachable!("Expected Str instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Str { from, to } => {
                assert_eq!(*from, Register::R0);
                assert_eq!(
                    *to,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(-1isize as u64)
                    }
                );
            }
            i => unreachable!("Expected Str instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Str { from, to } => {
                assert_eq!(*from, Register::R0);
                assert_eq!(
                    *to,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(10)
                    }
                );
            }
            i => unreachable!("Expected Str instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Str { from, to } => {
                assert_eq!(*from, Register::R0);
                assert_eq!(*to, MemoryLocation::Labeled(u64::MAX));
            }
            i => unreachable!("Expected Str instruction, got {i:?}"),
        }
    }

    #[test]
    fn parse_ldr_instr() {
        let input = b"
            .code
            ldr r0, [r1]
            ldr r0, [r1, r2]
            ldr r0, [r1, 5]
            ldr r0, [r1, -1]
            ldr r0, [r1, 0xa]
            ldr r0, data
            ";

        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = Parser::parse(&tokens, input).unwrap();

        assert_eq!(parsed.instructions.len(), 6);
        assert_eq!(parsed.unlinked_instructions.len(), 1);
        assert_eq!(&input[parsed.unlinked_instructions[0].label()], b"data".as_slice());

        let mut insts = parsed.instructions.iter();

        match insts.next().unwrap() {
            Instruction::Ldr { to, from } => {
                assert_eq!(*to, Register::R0);
                assert_eq!(
                    *from,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(0)
                    }
                );
            }
            i => unreachable!("Expected Ldr instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Ldr { to, from } => {
                assert_eq!(*to, Register::R0);
                assert_eq!(
                    *from,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Register(Register::R2)
                    }
                );
            }
            i => unreachable!("Expected Ldr instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Ldr { to, from } => {
                assert_eq!(*to, Register::R0);
                assert_eq!(
                    *from,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(5)
                    }
                );
            }
            i => unreachable!("Expected Ldr instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Ldr { to, from } => {
                assert_eq!(*to, Register::R0);
                assert_eq!(
                    *from,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(-1isize as u64)
                    }
                );
            }
            i => unreachable!("Expected Ldr instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Ldr { to, from } => {
                assert_eq!(*to, Register::R0);
                assert_eq!(
                    *from,
                    MemoryLocation::Offset {
                        base: Register::R1,
                        offset: Operand::Value(10)
                    }
                );
            }
            i => unreachable!("Expected Ldr instruction, got {i:?}"),
        }
        match insts.next().unwrap() {
            Instruction::Ldr { to, from } => {
                assert_eq!(*to, Register::R0);
                assert_eq!(*from, MemoryLocation::Labeled(u64::MAX));
            }
            i => unreachable!("Expected Ldr instruction, got {i:?}"),
        }
    }

    #[test]
    fn parse_adr_instruction() {
        let input = b"
            .code
            adr r0, data
            ";

        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = Parser::parse(&tokens, input).unwrap();

        assert_eq!(parsed.instructions.len(), 1);
        assert_eq!(parsed.unlinked_instructions.len(), 1);
        assert_eq!(
            parsed.instructions[0],
            Instruction::Adr {
                reg: Register::R0,
                addr: u64::MAX
            }
        );
    }
}
