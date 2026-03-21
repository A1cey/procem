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
pub struct Bss;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Code;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Data;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Undefined;

enum Section {
    Code,
    Data,
    Bss,
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Parser<'input, W> {
    Undefined(InnerParser<'input, W, Undefined>),
    Code(InnerParser<'input, W, Code>),
    Data(InnerParser<'input, W, Data>),
    Bss(InnerParser<'input, W, Bss>),
}

impl<'input, W: Word> Parser<'input, W> {
    #[inline]
    #[must_use]
    fn new(tokens: &'input [Token], input: &'input [u8]) -> Self {
        Self::Undefined(InnerParser {
            tokens,
            errors: None,
            instructions: Vec::with_capacity(tokens.len() / 3), // instructions most often are 4 tokens long, to balance out shorter ones 3 is used
            idx: 0,
            labels: HashMap::default(),
            input,
            end_parsing: false,
            _current_section: PhantomData,
        })
    }

    /// Parse tokens into a list of instructions.
    ///
    /// # Errors
    /// Returns a list of errors that occurred during parsing.
    #[inline]
    pub fn parse(tokens: &'input [Token], input: &'input [u8]) -> Result<Vec<Instruction<W>>, Vec<ParserError>> {
        let mut parser = Self::new(tokens, input);

        while !parser.is_done() {
            parser = parser.step();
        }
        parser.finish()
    }

    #[must_use]
    fn step(self) -> Self {
        let current_token = match &self {
            Self::Undefined(p) => p.peak_token().cloned(),
            Self::Code(p) => p.peak_token().cloned(),
            Self::Data(p) => p.peak_token().cloned(),
            Self::Bss(p) => p.peak_token().cloned(),
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
    const fn is_done(&self) -> bool {
        match self {
            Self::Undefined(p) => p.end_parsing || p.idx >= p.tokens.len(),
            Self::Code(p) => p.end_parsing || p.idx >= p.tokens.len(),
            Self::Data(p) => p.end_parsing || p.idx >= p.tokens.len(),
            Self::Bss(p) => p.end_parsing || p.idx >= p.tokens.len(),
        }
    }

    #[inline]
    fn finish(self) -> Result<Vec<Instruction<W>>, Vec<ParserError>> {
        match self {
            Self::Undefined(p) => p.errors.map_or(Ok(p.instructions), Err),
            Self::Code(p) => p.errors.map_or(Ok(p.instructions), Err),
            Self::Data(p) => p.errors.map_or(Ok(p.instructions), Err),
            Self::Bss(p) => p.errors.map_or(Ok(p.instructions), Err),
        }
    }

    #[must_use]
    fn change_section(self, range: Range) -> Self {
        macro_rules! change_and_advance {
            ($parser:expr, $variant:ident, $method:ident) => {{
                let mut next = $parser.$method();
                next.idx += 1;

                Self::$variant(next)
            }};
        }

        macro_rules! error_and_advance {
            ($parser:expr, $variant:ident, $got: expr) => {{
                $parser.add_error(ParserError::InvalidToken {
                    idx: $parser.idx,
                    expected: "Section Directive (code, data, bss)",
                    got: $got,
                });
                $parser.idx += 1;
                Self::$variant($parser)
            }};
        }

        macro_rules! change_section {
            ($variant:ident, $method:ident) => {
                match self {
                    Self::Undefined(p) => change_and_advance!(p, $variant, $method),
                    Self::Code(p) => change_and_advance!(p, $variant, $method),
                    Self::Data(p) => change_and_advance!(p, $variant, $method),
                    Self::Bss(p) => change_and_advance!(p, $variant, $method),
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
            Section::Code => change_section!(Code, into_code),
            Section::Data => change_section!(Data, into_data),
            Section::Bss => change_section!(Bss, into_bss),
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
    fn parse_section_directive<S>(parser: &InnerParser<'_, W, S>, range: Range) -> Section {
        match &parser.input[range] {
            directive if directive.eq_ignore_ascii_case(b"code") => Section::Code,
            directive if directive.eq_ignore_ascii_case(b"data") => Section::Data,
            directive if directive.eq_ignore_ascii_case(b"bss") => Section::Bss,
            directive => Section::Invalid(string_from_u8_slice(directive)),
        }
    }
}

// Add labels for bss and bss length to store at this label
// Add labels for data and data to store at this label
// Change labels in instructions to be marked for linking
// Add linker to link labels in instructions to be linked to labels in code, data and bss -> jmp may only be mapped to code labels, reading data instructions can only read from data and bss labels 
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InnerParser<'input, W, Section = Undefined> {
    tokens: &'input [Token],
    instructions: Vec<Instruction<W>>,
    errors: Option<Vec<ParserError>>,
    idx: usize,
    labels: HashMap<&'input [u8], usize>,
    input: &'input [u8],
    end_parsing: bool,
    _current_section: PhantomData<Section>,
}

impl<'input, W: Word, Section> InnerParser<'input, W, Section> {
    #[inline]
    #[must_use]
    fn into_code(self) -> InnerParser<'input, W, Code> {
        InnerParser {
            tokens: self.tokens,
            errors: self.errors,
            instructions: self.instructions,
            idx: self.idx,
            labels: self.labels,
            input: self.input,
            end_parsing: self.end_parsing,
            _current_section: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    fn into_data(self) -> InnerParser<'input, W, Data> {
        InnerParser {
            tokens: self.tokens,
            errors: self.errors,
            instructions: self.instructions,
            idx: self.idx,
            labels: self.labels,
            input: self.input,
            end_parsing: self.end_parsing,
            _current_section: PhantomData,
        }
    }

    #[inline]
    #[must_use]
    fn into_bss(self) -> InnerParser<'input, W, Bss> {
        InnerParser {
            tokens: self.tokens,
            errors: self.errors,
            instructions: self.instructions,
            idx: self.idx,
            labels: self.labels,
            input: self.input,
            end_parsing: self.end_parsing,
            _current_section: PhantomData,
        }
    }

    /// Returns the current token.
    #[inline]
    fn peak_token(&self) -> Option<&'_ Token> {
        self.tokens.get(self.idx)
    }

    #[inline]
    fn add_error(&mut self, err: ParserError) {
        self.errors.get_or_insert_default().push(err);
    }

    #[inline]
    #[must_use]
    fn string_from_asm(&self, range: &Range) -> String {
        String::from_utf8_lossy(&self.input[range]).to_string()
    }
}

impl<W: Word> InnerParser<'_, W, Undefined> {}

impl<W: Word> InnerParser<'_, W, Code> {
    fn parse_next_token(&mut self) {
        match &self.tokens.get(self.idx) {
            Some(t) => match t {
                Token::Label(label) => {
                    if let Some(old_instruction_idx) = self.labels.insert(&self.input[label], self.instructions.len()) {
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

    // TODO: this only works for labels defined in previous code. To fix this mark this location as needing to be linked + add a linker step to link all labels to locations
    fn expect_destination(&mut self, instr: ASMJumpInstruction) {
        self.idx += 1;

        if let Some(Token::LabelOrInstruction(label)) = self.tokens.get(self.idx) {
            match self.labels.get(&self.input[label]) {
                Some(&idx) => match idx.try_into() {
                    Ok(idx) => {
                        self.instructions.push(Instruction::from_jump_instruction(instr, idx));
                    }
                    Err(_) => {
                        self.add_error(ParserError::LabelIndexToWordConversionFailed {
                            idx: self.idx,
                            label: self.string_from_asm(label),
                        });
                    }
                },
                None => self.add_error(ParserError::LabelNotFound { 
                    idx: self.idx,
                    label: self.string_from_asm(label),
                }),
            }
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
            Some(Token::ImmediateLiteral(lit)) => Ok(Operand::Value(self.convert_lit_to_val(lit)?)),
            _ => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Register or Literal",
                got: self.current_token_string(),
            }),
        }
    }

    fn expect_word(&mut self) -> Result<W, ParserError> {
        self.idx += 1; // manual, to enable borrow of self inside match
        match self.tokens.get(self.idx) {
            Some(Token::ImmediateLiteral(lit)) => Ok(self.convert_lit_to_val(lit)?),
            _ => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Literal",
                got: self.current_token_string(),
            }),
        }
    }

    #[inline]
    fn get_next(&mut self) -> Option<&'_ Token> {
        self.idx += 1;
        self.tokens.get(self.idx)
    }

    #[inline]
    fn current_token_string(&self) -> String {
        self.tokens
            .get(self.idx)
            .map_or_else(|| "End".to_string(), |token| format!("{token:?}"))
    }

    fn convert_lit_to_val(&self, lit: &ImmediateLiteral) -> Result<W, ParserError> {
        match lit {
            ImmediateLiteral::Char(c) => Ok((*c as i32).into()),
            ImmediateLiteral::Binary(range) => {
                let lit = String::from_utf8_lossy(&self.input[range]);
                W::from_str_radix(&lit, 2).map_err(|err| ParserError::LiteralParsing {
                    lit: lit.to_string(),
                    err,
                })
            }
            ImmediateLiteral::Boolean(b) => Ok(i32::from(*b).into()),
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

        let literal = match self.expect_word() {
            Ok(lit) => lit,
            Err(err) => return self.add_error(err),
        };

        self.instructions
            .push(Instruction::from_shift_instruction(instr, register, literal));
    }

    fn expect_rotate_instruction(&mut self, instr: ASMRotateInstruction) {
        let register = match self.expect_register() {
            Ok(reg) => reg,
            Err(err) => return self.add_error(err),
        };

        if let Err(err) = self.expect_comma() {
            return self.add_error(err);
        }

        let literal = match self.expect_word() {
            Ok(lit) => lit,
            Err(err) => return self.add_error(err),
        };

        let literal: usize = literal.into();
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
                if let Some(old_instruction_idx) = self.labels.insert(&self.input[label], self.instructions.len()) {
                    self.add_error(ParserError::DuplicateLabel {
                        idx: self.instructions.len(),
                        old_idx: old_instruction_idx,
                    });
                }
                todo!("Data labels and instruction len is not correct: separate data labels?");
            }
            Token::Directive(directive) => self.parse_directive(&self.input[directive]), // Expect directives
            token => self.add_error(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Label or Directive",
                got: format!("{token:?}"),
            }),
        }
    }

    fn parse_directive(&mut self, directive: &[u8]) {
        match directive {
            directive if directive.eq_ignore_ascii_case(b"word") => {
                todo!("Expect ImmediateLiterals / Comma")
            }
            directive if directive.eq_ignore_ascii_case(b"ascii") => {
                todo!("Expect StringLiterals / Comma")
            }
            directive => self.add_error(ParserError::InvalidDirective {
                idx: self.idx,
                directive: string_from_u8_slice(directive),
                expected: "Only .word and .ascii are allowed in .data sections.".to_string(),
            }),
        }
    }
}

impl<W: Word> InnerParser<'_, W, Bss> {
    fn parse_next_token(&mut self) {
        match &self.tokens[self.idx] {
            Token::Directive(section) => self.parse_directive(&self.input[section]),
            Token::Label(label) => {
                if let Some(old_instruction_idx) = self.labels.insert(&self.input[label], self.instructions.len()) {
                    self.add_error(ParserError::DuplicateLabel {
                        idx: self.instructions.len(),
                        old_idx: old_instruction_idx,
                    });
                }
                todo!("bss labels and instruction len is not correct: separate bss labels?");
            }
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
            directive if directive.eq_ignore_ascii_case(b"space") => {}
            directive => self.add_error(ParserError::InvalidDirective {
                idx: self.idx,
                directive: string_from_u8_slice(directive),
                expected: "Only .space is allowed in .bss sections.".to_string(),
            }),
        }
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
}
