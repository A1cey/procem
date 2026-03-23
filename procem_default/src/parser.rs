use core::num::ParseIntError;
use std::{collections::HashMap, marker::PhantomData, num::TryFromIntError};

use procem::{
    register::{Register, RegisterError},
    word::Word,
};
use thiserror::Error;

use crate::instruction::asm_instruction::{
    ASMInstruction, ASMJumpInstruction, ASMRegOperandInstruction, ASMRotateInstruction, ASMShiftInstruction,
    ASMSingleOperandInstruction, ASMSingleRegInstruction, ASMTwoOperandInstruction,
};
use crate::instruction::operand::Operand;
use crate::instruction::{Instruction, asm_instruction::ASMNoArgInstruction};
use crate::tokenizer::{ImmediateLiteral, Token};
use ars::range::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed<'input, W> {
    instructions: Vec<Instruction<W>>,
    instruction_labels: HashMap<&'input [u8], usize>,
    data: Vec<W>,
    data_labels: HashMap<&'input [u8], usize>,
    bss: usize,
    bss_labels: HashMap<&'input [u8], usize>,
}

impl<W> Parsed<'_, W> {
    #[inline]
    #[must_use]
    pub fn instructions(&self) -> &[Instruction<W>] {
        &self.instructions
    }
    #[inline]
    #[must_use]
    pub const fn instruction_labels(&self) -> &HashMap<&[u8], usize> {
        &self.instruction_labels
    }
    #[inline]
    #[must_use]
    pub fn data(&self) -> &[W] {
        &self.data
    }
    #[inline]
    #[must_use]
    pub const fn data_labels(&self) -> &HashMap<&[u8], usize> {
        &self.data_labels
    }
    #[inline]
    #[must_use]
    pub const fn bss(&self) -> usize {
        self.bss
    }
    #[inline]
    #[must_use]
    pub const fn bss_labels(&self) -> &HashMap<&[u8], usize> {
        &self.bss_labels
    }
}

impl<'input, W, Section> From<InnerParser<'input, W, Section>> for Parsed<'input, W> {
    fn from(p: InnerParser<'input, W, Section>) -> Self {
        Self {
            instructions: p.instructions,
            instruction_labels: p.instruction_labels,
            data: p.data,
            data_labels: p.data_labels,
            bss: p.bss,
            bss_labels: p.bss_labels,
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parser<'input, W> {
    Undefined(InnerParser<'input, W, Undefined>),
    Code(InnerParser<'input, W, Code>),
    Data(InnerParser<'input, W, Data>),
    Bss(InnerParser<'input, W, Bss>),
}

impl<'input, W: Word> Parser<'input, W> {
    fn new(tokens: &'input [Token], input: &'input [u8]) -> Self {
        Self::Undefined(InnerParser {
            tokens,
            instructions: Vec::default(),
            instruction_labels: HashMap::default(),
            data: Vec::default(),
            data_labels: HashMap::default(),
            bss: 0,
            bss_labels: HashMap::default(),
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
    pub fn parse(tokens: &'input [Token], input: &'input [u8]) -> Result<Parsed<'input, W>, Vec<ParserError>> {
        let mut parser = Self::new(tokens, input);

        while !parser.is_done() {
            parser = parser.step();
        }
        parser.finish()
    }

    fn step(self) -> Self {
        let current_token = match &self {
            Self::Undefined(p) => p.get_token().copied(),
            Self::Code(p) => p.get_token().copied(),
            Self::Data(p) => p.get_token().copied(),
            Self::Bss(p) => p.get_token().copied(),
        };

        match current_token {
            Some(Token::Directive(range)) => self.change_section(range),
            Some(t) => match self {
                Self::Undefined(mut p) => {
                    p.add_error(ParserError::InvalidToken {
                        idx: p.idx,
                        expected: "Section Directive",
                        got: t.to_string(),
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
            None => unreachable!("This function is never called with an invalid idx"),
        }
    }

    #[inline]
    const fn input(&self) -> &[u8] {
        match self {
            Self::Undefined(p) => p.input,
            Self::Code(p) => p.input,
            Self::Data(p) => p.input,
            Self::Bss(p) => p.input,
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
    fn finish(self) -> Result<Parsed<'input, W>, Vec<ParserError>> {
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

    fn change_section(self, range: Range) -> Self {
        macro_rules! change_and_advance {
            ($parser:expr, $method:ident, $variant:ident) => {{
                let mut next = $parser.$method();
                next.idx += 1;

                Self::$variant(next)
            }};
        }

        macro_rules! error_and_advance {
            ($parser:expr, $variant:ident, $directive: expr) => {{
                let got = string_from_u8_slice($directive);

                $parser.add_error(ParserError::InvalidToken {
                    idx: $parser.idx,
                    expected: "Section Directive (code, data, bss)",
                    got,
                });
                $parser.idx += 1;
                Self::$variant($parser)
            }};
        }

        let directive = self.input()[range].to_vec(); // NOTE: This allocates a vec every step!

        match &directive {
            directive if directive.eq_ignore_ascii_case(b"code") => match self {
                Self::Undefined(p) => change_and_advance!(p, into_code, Code),
                Self::Code(p) => change_and_advance!(p, into_code, Code),
                Self::Data(p) => change_and_advance!(p, into_code, Code),
                Self::Bss(p) => change_and_advance!(p, into_code, Code),
            },
            directive if directive.eq_ignore_ascii_case(b"data") => match self {
                Self::Undefined(p) => change_and_advance!(p, into_data, Data),
                Self::Code(p) => change_and_advance!(p, into_data, Data),
                Self::Data(p) => change_and_advance!(p, into_data, Data),
                Self::Bss(p) => change_and_advance!(p, into_data, Data),
            },
            directive if directive.eq_ignore_ascii_case(b"bss") => match self {
                Self::Undefined(p) => change_and_advance!(p, into_bss, Bss),
                Self::Code(p) => change_and_advance!(p, into_bss, Bss),
                Self::Data(p) => change_and_advance!(p, into_bss, Bss),
                Self::Bss(p) => change_and_advance!(p, into_bss, Bss),
            },
            directive => match self {
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerParser<'input, W, Section = Undefined> {
    tokens: &'input [Token],
    instructions: Vec<Instruction<W>>,
    instruction_labels: HashMap<&'input [u8], usize>,
    data: Vec<W>,
    data_labels: HashMap<&'input [u8], usize>,
    bss: usize,
    bss_labels: HashMap<&'input [u8], usize>,
    errors: Option<Vec<ParserError>>,
    idx: usize,
    input: &'input [u8],
    end_parsing: bool,
    _current_section: PhantomData<Section>,
}

// TODO: implement logic, that instructions can only be parsed when in .code section and similar for other sections
impl<'input, W: Word, Section> InnerParser<'input, W, Section> {
    fn into_code(self) -> InnerParser<'input, W, Code> {
        InnerParser {
            tokens: self.tokens,
            instructions: self.instructions,
            instruction_labels: self.instruction_labels,
            data: self.data,
            data_labels: self.data_labels,
            bss: self.bss,
            bss_labels: self.bss_labels,
            errors: self.errors,
            idx: self.idx,
            input: self.input,
            end_parsing: self.end_parsing,
            _current_section: PhantomData,
        }
    }

    fn into_data(self) -> InnerParser<'input, W, Data> {
        InnerParser {
            tokens: self.tokens,
            instructions: self.instructions,
            instruction_labels: self.instruction_labels,
            data: self.data,
            data_labels: self.data_labels,
            bss: self.bss,
            bss_labels: self.bss_labels,
            errors: self.errors,
            idx: self.idx,
            input: self.input,
            end_parsing: self.end_parsing,
            _current_section: PhantomData,
        }
    }

    fn into_bss(self) -> InnerParser<'input, W, Bss> {
        InnerParser {
            tokens: self.tokens,
            instructions: self.instructions,
            instruction_labels: self.instruction_labels,
            data: self.data,
            data_labels: self.data_labels,
            bss: self.bss,
            bss_labels: self.bss_labels,
            errors: self.errors,
            idx: self.idx,
            input: self.input,
            end_parsing: self.end_parsing,
            _current_section: PhantomData,
        }
    }

    /// Returns the current token.
    fn get_token(&self) -> Option<&'_ Token> {
        self.tokens.get(self.idx)
    }

    /// Returns the next token if available.
    fn peak_token(&self) -> Option<&'_ Token> {
        self.tokens.get(self.idx + 1)
    }

    #[inline]
    fn get_next(&mut self) -> Option<&'_ Token> {
        self.idx += 1;
        self.tokens.get(self.idx)
    }

    #[inline]
    fn add_error(&mut self, err: ParserError) {
        self.errors.get_or_insert_default().push(err);
    }

    fn convert_lit_to_word(&self, lit: ImmediateLiteral) -> Result<W, ParserError> {
        match lit {
            ImmediateLiteral::Char(c) => Ok(W::from(c as i32)),
            ImmediateLiteral::Binary(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                W::from_str_radix(&lit, 2).map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
            ImmediateLiteral::Boolean(b) => Ok(W::from(i32::from(b))),
            ImmediateLiteral::Decimal(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                W::from_str_radix(&lit, 10).map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
            ImmediateLiteral::Hexadecimal(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                W::from_str_radix(&lit, 16).map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
            ImmediateLiteral::Octal(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                W::from_str_radix(&lit, 8).map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
        }
    }

    fn expect_immediate_literal(&mut self) -> Result<ImmediateLiteral, ParserError> {
        self.idx += 1; // manual, to enable borrow of self inside match
        match self.tokens.get(self.idx) {
            Some(token) => match token {
                Token::ImmediateLiteral(lit) => Ok(*lit),
                token => Err(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "ImmediateLiteral",
                    got: token.to_string(),
                }),
            },
            None => Err(ParserError::TokenNotFound { idx: self.idx }),
        }
    }
}

impl<W: Word> InnerParser<'_, W, Undefined> {}

impl<W: Word> InnerParser<'_, W, Code> {
    fn parse_next_token(&mut self) {
        match &self.tokens.get(self.idx) {
            Some(t) => match t {
                Token::Label(label) => {
                    if let Some(old_instruction_idx) = self
                        .instruction_labels
                        .insert(&self.input[label], self.instructions.len())
                    {
                        self.add_error(ParserError::DuplicateLabel {
                            idx: self.instructions.len(),
                            old_idx: old_instruction_idx,
                        });
                    }
                }
                Token::LabelOrInstruction(inst) => {
                    // Here only instructions are possible
                    // labels need to be written "<name>:" if they are at the start of an asm line
                    // -> Labels like this are tokenized as Label not as LabelOrInstruction.
                    self.parse_instruction(&self.input[inst]);
                }
                Token::End => self.end_parsing = true,
                token => self.add_error(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "Label, Instruction or End",
                    got: format!("{token:?}"),
                }),
            },
            None => unreachable!("self.tokens is never indexed with an invalid idx."),
        }
    }

    fn parse_instruction(&mut self, instruction: &[u8]) {
        match instruction.try_into() {
            Ok(inst) => match inst {
                ASMInstruction::NoArg(inst) => self.instructions.push(match inst {
                    ASMNoArgInstruction::Nop => Instruction::Nop,
                    ASMNoArgInstruction::Ret => Instruction::Ret,
                }),
                ASMInstruction::RegOperand(inst) => self.expect_reg_operand_instruction(inst),
                ASMInstruction::Jump(inst) => self.expect_destination(inst),
                ASMInstruction::TwoOperand(inst) => self.expect_two_operand_instruction(inst),
                ASMInstruction::SingleOperand(inst) => self.expect_single_operand_instruction(inst),
                ASMInstruction::SingleReg(inst) => self.expect_single_reg_instruction(inst),
                ASMInstruction::Rotate(inst) => self.expect_rotate_instruction(inst),
                ASMInstruction::Shift(inst) => self.expect_shift_instruction(inst),
            },
            Err(()) => self.add_error(ParserError::UnknownInstruction {
                idx: self.idx,
                inst: string_from_u8_slice(instruction),
            }),
        }
    }

    fn expect_destination(&mut self, instr: ASMJumpInstruction) {
        self.idx += 1;

        if let Some(Token::LabelOrInstruction(label)) = self.tokens.get(self.idx) {
            self.instructions
                .push(Instruction::UnlinkedJump { instr, label: *label });

            // TODO: Code in linker ?
            // match self.instruction_labels.get(&self.input[label]) {
            //     Some(&idx) => match idx.try_into() {
            //         Ok(idx) => {
            //             self.instructions.push(Instruction::from_jump_instruction(instr, idx));
            //         }
            //         Err(_) => {
            //             self.add_error(ParserError::LabelIndexToWordConversionFailed {
            //                 idx: self.idx,
            //                 label: self.string_from_asm(label),
            //             });
            //         }
            //     },
            //     None => self.add_error(ParserError::LabelNotFound {
            //         idx: self.idx,
            //         label: self.string_from_asm(label),
            //     }),
            // }
        } else {
            self.add_error(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Label",
                got: self.current_token_string(),
            });
        }
    }

    fn expect_register(&mut self) -> Result<Register, ParserError> {
        self.idx += 1; // manual, to enable borrow of self inside match
        match self.tokens.get(self.idx) {
            Some(Token::Register(reg)) => {
                Register::try_from(&self.input[reg]).map_err(|err| ParserError::RegisterParsing { err })
            }
            _ => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Register",
                got: self.current_token_string(),
            }),
        }
    }

    fn expect_comma(&mut self) -> Result<(), ParserError> {
        match self.get_next() {
            Some(Token::Comma) => Ok(()),
            _ => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Comma",
                got: self.current_token_string(),
            }),
        }
    }

    fn expect_operand(&mut self) -> Result<Operand<W>, ParserError> {
        self.idx += 1; // manual, to enable borrow of self inside match
        match self.tokens.get(self.idx) {
            Some(Token::Register(reg)) => Ok(Operand::Register(
                Register::try_from(&self.input[reg]).map_err(|err| ParserError::RegisterParsing { err })?,
            )),
            Some(Token::ImmediateLiteral(lit)) => Ok(Operand::Value(self.convert_lit_to_word(*lit)?)),
            _ => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Register or Literal",
                got: self.current_token_string(),
            }),
        }
    }

    #[inline]
    fn current_token_string(&self) -> String {
        self.tokens
            .get(self.idx)
            .map_or_else(|| "End".to_string(), |token| format!("{token:?}"))
    }

    fn expect_reg_operand_instruction(&mut self, instr: ASMRegOperandInstruction) {
        let acc = match self.expect_register() {
            Ok(reg) => reg,
            Err(err) => return self.add_error(err),
        };

        if let Err(err) = self.expect_comma() {
            return self.add_error(err);
        }

        let operand = match self.expect_operand() {
            Ok(op) => op,
            Err(err) => return self.add_error(err),
        };

        self.instructions
            .push(Instruction::from_reg_operand_instruction(instr, acc, operand));
    }

    fn expect_single_reg_instruction(&mut self, instr: ASMSingleRegInstruction) {
        let reg = match self.expect_register() {
            Ok(reg) => reg,
            Err(err) => return self.add_error(err),
        };

        self.instructions
            .push(Instruction::from_single_reg_instruction(instr, reg));
    }

    fn expect_single_operand_instruction(&mut self, instr: ASMSingleOperandInstruction) {
        let operand = match self.expect_operand() {
            Ok(op) => op,
            Err(err) => return self.add_error(err),
        };

        self.instructions
            .push(Instruction::from_single_operand_instruction(instr, operand));
    }

    fn expect_two_operand_instruction(&mut self, instr: ASMTwoOperandInstruction) {
        let lhs = match self.expect_operand() {
            Ok(op) => op,
            Err(err) => return self.add_error(err),
        };

        if let Err(err) = self.expect_comma() {
            return self.add_error(err);
        }

        let rhs = match self.expect_operand() {
            Ok(op) => op,
            Err(err) => return self.add_error(err),
        };

        self.instructions
            .push(Instruction::from_two_operand_instruction(instr, lhs, rhs));
    }

    fn expect_shift_instruction(&mut self, instr: ASMShiftInstruction) {
        let register = match self.expect_register() {
            Ok(reg) => reg,
            Err(err) => return self.add_error(err),
        };

        if let Err(err) = self.expect_comma() {
            return self.add_error(err);
        }

        let literal = match self.expect_immediate_literal() {
            Ok(lit) => lit,
            Err(err) => return self.add_error(err),
        };

        let word = match self.convert_lit_to_word(literal) {
            Ok(word) => word,
            Err(err) => return self.add_error(err),
        };

        self.instructions
            .push(Instruction::from_shift_instruction(instr, register, word));
    }

    fn expect_rotate_instruction(&mut self, instr: ASMRotateInstruction) {
        let register = match self.expect_register() {
            Ok(reg) => reg,
            Err(err) => return self.add_error(err),
        };

        if let Err(err) = self.expect_comma() {
            return self.add_error(err);
        }

        let literal = match self.expect_immediate_literal() {
            Ok(lit) => lit,
            Err(err) => return self.add_error(err),
        };

        let word = match self.convert_lit_to_word(literal) {
            Ok(word) => word,
            Err(err) => return self.add_error(err),
        };

        let literal: usize = word.into();
        let literal: u32 = match literal.try_into() {
            Ok(lit) => lit,
            Err(err) => return self.add_error(ParserError::CannotConvertLiteralToU32 { literal, err }),
        };

        self.instructions
            .push(Instruction::from_rotate_instruction(instr, register, literal));
    }
}

impl<W: Word> InnerParser<'_, W, Data> {
    fn parse_next_token(&mut self) {
        match &self.tokens[self.idx] {
            Token::Label(label) => {
                if let Some(old_data_idx) = self.data_labels.insert(&self.input[label], self.data.len()) {
                    self.add_error(ParserError::DuplicateLabel {
                        idx: self.instructions.len(),
                        old_idx: old_data_idx,
                    });
                }
            }
            Token::Directive(directive) => self.parse_directive(&self.input[directive]),
            Token::End => self.end_parsing = true,
            token => self.add_error(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Label or Directive",
                got: format!("{token:?}"),
            }),
        }
    }

    fn parse_directive(&mut self, directive: &[u8]) {
        match directive {
            directive if directive.eq_ignore_ascii_case(b"word") => self.parse_words(),
            directive if directive.eq_ignore_ascii_case(b"ascii") => self.parse_ascii(),
            directive => self.add_error(ParserError::InvalidDirective {
                idx: self.idx,
                directive: string_from_u8_slice(directive),
                expected: "Only .word and .ascii are allowed in .data sections.".to_string(),
            }),
        }
    }

    #[inline]
    fn parse_words(&mut self) {
        self.expect_word();

        while let Some(Token::Comma) = self.peak_token() {
            self.idx += 1;
            self.expect_word();
        }
    }

    #[inline]
    fn expect_word(&mut self) {
        match self.expect_immediate_literal() {
            Ok(lit) => match self.convert_lit_to_word(lit) {
                Ok(word) => self.data.push(word),
                Err(err) => self.add_error(err),
            },
            Err(err) => self.add_error(err),
        }
    }

    #[inline]
    fn parse_ascii(&mut self) {
        self.expect_string_literal();

        while let Some(Token::Comma) = self.peak_token() {
            self.idx += 1;
            self.expect_string_literal();
        }
    }

    fn expect_string_literal(&mut self) {
        self.idx += 1;
        let token = self.tokens.get(self.idx);

        match token {
            Some(token) => match token {
                Token::StringLiteral(lit) => {
                    self.data
                        .extend(self.input[lit].iter().map(|&byte| W::from(i32::from(byte))));
                }
                token => self.add_error(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "ImmediateLiteral",
                    got: token.to_string(),
                }),
            },
            None => self.add_error(ParserError::TokenNotFound { idx: self.idx }),
        }
    }
}

impl<W: Word> InnerParser<'_, W, Bss> {
    fn parse_next_token(&mut self) {
        match &self.tokens[self.idx] {
            Token::Label(label) => {
                if let Some(old_bss_idx) = self.bss_labels.insert(&self.input[label], self.bss) {
                    self.add_error(ParserError::DuplicateLabel {
                        idx: self.instructions.len(),
                        old_idx: old_bss_idx,
                    });
                }
            }
            Token::Directive(section) => self.parse_directive(&self.input[section]),
            Token::End => self.end_parsing = true,
            token => self.add_error(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Label or Directive",
                got: format!("{token:?}"),
            }),
        }
    }

    #[inline]
    fn parse_directive(&mut self, directive: &[u8]) {
        match directive {
            directive if directive.eq_ignore_ascii_case(b"space") => self.parse_space(),
            directive => self.add_error(ParserError::InvalidDirective {
                idx: self.idx,
                directive: string_from_u8_slice(directive),
                expected: "Only .space is allowed in .bss sections.".to_string(),
            }),
        }
    }

    #[inline]
    fn parse_space(&mut self) {
        self.expect_space();

        while let Some(Token::Comma) = self.peak_token() {
            self.idx += 1;
            self.expect_space();
        }
    }

    #[inline]
    fn expect_space(&mut self) {
        match self.expect_immediate_literal() {
            Ok(lit) => match self.convert_lit_to_usize(lit) {
                Ok(space) => self.bss += space,
                Err(err) => self.add_error(err),
            },
            Err(err) => self.add_error(err),
        }
    }

    fn convert_lit_to_usize(&self, lit: ImmediateLiteral) -> Result<usize, ParserError> {
        match lit {
            ImmediateLiteral::Decimal(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                lit.parse().map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
            ImmediateLiteral::Binary(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                usize::from_str_radix(&lit, 2).map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
            ImmediateLiteral::Hexadecimal(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                usize::from_str_radix(&lit, 16).map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
            ImmediateLiteral::Octal(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                usize::from_str_radix(&lit, 8).map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
            ImmediateLiteral::Boolean(b) => Ok(usize::from(b)),
            ImmediateLiteral::Char(c) => Ok(c as usize),
        }
    }
}

#[inline]
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
        got: String,
    },
    #[error("Duplicate lable: First occurrence: {old_idx}, second occurrence {idx}")]
    DuplicateLabel { idx: usize, old_idx: usize },
    #[error("Unkown instruction at idx {idx}: {inst}")]
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
    CannotConvertLiteralToU32 { literal: usize, err: TryFromIntError },
    #[error("Label \".{label}\" not found. Needed at {idx}.")]
    LabelNotFound { idx: usize, label: String },
    #[error("Index {idx} of label \".{label}\" cannot be converted to word.")]
    LabelIndexToWordConversionFailed { idx: usize, label: String },
    #[error("Invalid section name: {section} at {idx}.")]
    InvalidSectionName { idx: usize, section: String },
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
    use procem::word::I32;

    use crate::{
        parser::{Parser, ParserError},
        tokenizer::Tokenizer,
    };

    #[test]
    fn parse_section() {
        let input = "
            .code
            .bss
            .data
            .Bss
            .CODE
            .Invalid
            ";
        let tokens = Tokenizer::tokenize(input.as_bytes()).unwrap();
        let mut p = Parser::<I32>::new(&tokens, input.as_bytes());

        macro_rules! check {
            ($variant:ident, $p:expr) => {
                match p {
                    Parser::$variant(_) => assert!(true),
                    p => panic!("Expected Parser variant {}, got: {p:?}", stringify!($variant)),
                }
            };
        }

        check!(Undefined, p);
        p = p.step();
        check!(Code, p);
        p = p.step();
        check!(Bss, p);
        p = p.step();
        check!(Data, p);
        p = p.step();
        check!(Bss, p);
        p = p.step();
        check!(Code, p);
        p = p.step();
        check!(Code, p);

        match p {
            Parser::Code(p) => assert_eq!(
                p.errors.unwrap()[0],
                ParserError::InvalidToken {
                    idx: 5,
                    expected: "Section Directive (code, data, bss)",
                    got: "Invalid".to_string()
                }
            ),
            _ => unreachable!("check! before ensures, that Code is the Variant"),
        }
    }

    #[test]
    fn parse_bss() {
        let input = "
            .bss
                a:
                .space 5
                .space 10
                b: .space 5, 0xA
            ";
        let tokens = Tokenizer::tokenize(input.as_bytes()).unwrap();
        let parsed = Parser::<I32>::parse(&tokens, input.as_bytes()).unwrap();

        assert_eq!(parsed.instructions().len(), 0);
        assert_eq!(parsed.instruction_labels().len(), 0);
        assert_eq!(parsed.data().len(), 0);
        assert_eq!(parsed.data_labels().len(), 0);

        assert_eq!(parsed.bss, 5 + 10 + 5 + 10);
        let labels = parsed.bss_labels();
        assert_eq!(labels.len(), 2);
        assert_eq!(labels[b"a".as_slice()], 0);
        assert_eq!(labels[b"b".as_slice()], 5 + 10);
    }

    #[test]
    fn parse_data() {
        let input = "
            .data
                a:
                    .word 5
                    .word 10
                b:
                    .ascii \"Hello World!\", \"\0\"
                c:
                    .word 5, 0xA
            ";
        let tokens = Tokenizer::tokenize(input.as_bytes()).unwrap();
        let parsed = Parser::<I32>::parse(&tokens, input.as_bytes()).unwrap();

        assert_eq!(parsed.instructions().len(), 0);
        assert_eq!(parsed.instruction_labels().len(), 0);
        assert_eq!(parsed.bss(), 0);
        assert_eq!(parsed.bss_labels().len(), 0);

        assert_eq!(parsed.data.len(), 1 + 1 + b"Hello World!".len() + b"\0".len() + 1 + 1);
        let labels = parsed.data_labels();
        assert_eq!(labels.len(), 3);
        assert_eq!(labels[b"a".as_slice()], 0);
        assert_eq!(labels[b"b".as_slice()], 1 + 1);
        assert_eq!(labels[b"c".as_slice()], 1 + 1 + b"Hello World!".len() + b"\0".len());
    }

    #[test]
    fn parse_all_sections() {
        let input = "
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
            g:
                add R1, 0o1
                jmp c
            ";
        let tokens = Tokenizer::tokenize(input.as_bytes()).unwrap();
        let parsed = Parser::<I32>::parse(&tokens, input.as_bytes()).unwrap();

        assert_eq!(parsed.instructions().len(), 6);
        assert_eq!(parsed.instruction_labels().len(), 3);
        assert_eq!(parsed.data().len(), 1 + 1);
        assert_eq!(parsed.data_labels().len(), 1 + 1);
        assert_eq!(parsed.bss, 5 + 10 + 5);
        assert_eq!(parsed.bss_labels().len(), 1);

        assert_eq!(parsed.instruction_labels()[b"a".as_slice()], 0);
        assert_eq!(parsed.instruction_labels()[b"c".as_slice()], 2);
        assert_eq!(parsed.instruction_labels()[b"g".as_slice()], 4);
        assert_eq!(parsed.data_labels()[b"d".as_slice()], 0);
        assert_eq!(parsed.data_labels()[b"e".as_slice()], 1);
        assert_eq!(parsed.bss_labels()[b"b".as_slice()], 0);
    }
}
