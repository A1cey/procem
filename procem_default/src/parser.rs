use core::num::ParseIntError;
use std::{collections::HashMap, num::TryFromIntError};

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
use crate::tokenizer::{Literal, Token};
use ars::{ascii::eq_ignore_ascii_case, range::Range};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy)]
pub(crate) enum Section {
    Bss,
    Code,
    Data,
    NotDefined,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Parser<'input, W> {
    tokens: &'input [Token],
    instructions: Vec<Instruction<W>>,
    errors: Option<Vec<ParserError>>,
    idx: usize,
    labels: HashMap<&'input [u8], usize>,
    input: &'input [u8],
    current_section: Section,
    end_parsing: bool,
}

// TODO: implement logic, that instructions can only be parsed when in .code section and similar for other sections

impl<'input, W: Word> Parser<'input, W> {
    fn new(tokens: &'input [Token], input: &'input [u8]) -> Self {
        Self {
            tokens,
            errors: None,
            instructions: Vec::default(),
            idx: 0,
            labels: HashMap::default(),
            input,
            current_section: Section::NotDefined,
            end_parsing: false,
        }
    }

    pub(crate) fn parse(tokens: &'input [Token], input: &'input [u8]) -> Result<Vec<Instruction<W>>, Vec<ParserError>> {
        let mut parser = Parser::new(tokens, input);
        parser.run();

        match parser.errors {
            None => Ok(parser.instructions),
            Some(err) => Err(err),
        }
    }

    fn run(&mut self) {
        let mut instruction_count = 0;

        while self.idx < self.tokens.len() && !self.end_parsing {
            self.parse_next_token(&mut instruction_count);
            self.idx += 1;
        }
    }

    // TODO: can instruction_count be replaced by self.instructions.len()?
    fn parse_next_token(&mut self, instruction_count: &mut usize) {
        match &self.tokens[self.idx] {
            Token::Section(section) => self.parse_section(&self.input[section]),
            Token::Label(label) => {
                if let Some(old_instruction_idx) = self.labels.insert(&self.input[label], *instruction_count) {
                    self.add_error(ParserError::DuplicateLabel {
                        idx: *instruction_count,
                        old_idx: old_instruction_idx,
                    });
                }
            }
            Token::LabelOrInstruction(inst) => {
                self.parse_instruction(&self.input[inst]); // Here only instructions are possible
                *instruction_count += 1;
            }
            Token::End => self.end_parsing = true,
            token => self.add_error(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Label or Instruction",
                got: format!("{token:?}"),
            }),
        }
    }

    #[inline]
    fn string_from_asm(&self, range: &Range) -> String {
        String::from_utf8_lossy(&self.input[range]).to_string()
    }

    #[inline]
    fn string_from_u8_slice(slice: &[u8]) -> String {
        String::from_utf8_lossy(slice).to_string()
    }

    #[inline]
    fn add_error(&mut self, err: ParserError) {
        self.errors.get_or_insert_default().push(err);
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
                inst: Self::string_from_u8_slice(instruction),
            }),
        }
    }

    fn parse_section(&mut self, section: &[u8]) {
        match section {
            section if eq_ignore_ascii_case(section, b"code") => self.current_section = Section::Code,
            section if eq_ignore_ascii_case(section, b"data") => self.current_section = Section::Data,
            section if eq_ignore_ascii_case(section, b"bss") => self.current_section = Section::Bss,
            _ => {
                // TODO: Should section be reset to NotDefined here?
                self.add_error(ParserError::InvalidSectionName {
                    idx: self.idx,
                    section: Self::string_from_u8_slice(section),
                });
            }
        }
    }

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
            Some(Token::Register(reg)) => Register::try_from(&self.input[reg]).map_err(ParserError::RegisterParsing),
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
                Register::try_from(&self.input[reg]).map_err(ParserError::RegisterParsing)?,
            )),
            Some(Token::Literal(lit)) => Ok(Operand::Value(self.convert_lit_to_val(lit)?)),
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
            Some(Token::Literal(lit)) => Ok(self.convert_lit_to_val(lit)?),
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

    fn convert_lit_to_val(&self, lit: &Literal) -> Result<W, ParserError> {
        match lit {
            Literal::Char(s) => Ok((*s as i32).into()),
            Literal::Binary(s) => {
                let s = String::from_utf8_lossy(&self.input[s]);
                W::from_str_radix(&s, 2).map_err(ParserError::LiteralParsing)
            }
            Literal::Boolean(s) => Ok(i32::from(*s).into()),
            Literal::Decimal(s) => {
                let s = String::from_utf8_lossy(&self.input[s]);
                W::from_str_radix(&s, 10).map_err(ParserError::LiteralParsing)
            }
            Literal::Hexadecimal(s) => {
                let s = String::from_utf8_lossy(&self.input[s]);
                W::from_str_radix(&s, 16).map_err(ParserError::LiteralParsing)
            }
            Literal::Octal(s) => {
                let s = String::from_utf8_lossy(&self.input[s]);
                W::from_str_radix(&s, 8).map_err(ParserError::LiteralParsing)
            }
            Literal::String(_) => Err(ParserError::CannotConvertStrToVal),
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
    #[error("Error while parsing register.")]
    RegisterParsing(#[from] RegisterError),
    #[error("Error while parsing literal.")]
    LiteralParsing(#[from] ParseIntError),
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
}

#[cfg(test)]
mod test {
    use procem::word::I32;

    use crate::{
        parser::{Parser, ParserError, Section},
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
        let mut instruction_count = 0;

        assert_eq!(p.current_section, Section::NotDefined);

        p.parse_next_token(&mut instruction_count);
        assert_eq!(p.current_section, Section::Code);
        p.idx += 1;

        p.parse_next_token(&mut instruction_count);
        assert_eq!(p.current_section, Section::Bss);
        p.idx += 1;

        p.parse_next_token(&mut instruction_count);
        assert_eq!(p.current_section, Section::Data);
        p.idx += 1;

        p.parse_next_token(&mut instruction_count);
        assert_eq!(p.current_section, Section::Bss);
        p.idx += 1;

        p.parse_next_token(&mut instruction_count);
        assert_eq!(p.current_section, Section::Code);
        p.idx += 1;

        p.parse_next_token(&mut instruction_count);
        assert_eq!(p.current_section, Section::Code);
        assert_eq!(
            p.errors.unwrap()[0],
            ParserError::InvalidSectionName {
                idx: 5,
                section: "Invalid".to_string()
            }
        );
    }
}
