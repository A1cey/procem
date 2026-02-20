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

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Parser<'a, W> {
    tokens: &'a [Token<'a>],
    instructions: Vec<Instruction<W>>,
    errors: Option<Vec<ParserError>>,
    idx: usize,
    labels: HashMap<&'a str, usize>,
}

impl<'a, W: Word> Parser<'a, W> {
    fn new(tokens: &'a [Token<'a>]) -> Self {
        Self {
            tokens,
            errors: None,
            instructions: Vec::default(),
            idx: 0,
            labels: HashMap::default(),
        }
    }

    pub(crate) fn parse(tokens: &'a [Token<'a>]) -> Result<Vec<Instruction<W>>, Vec<ParserError>> {
        let mut parser = Parser::new(tokens);
        parser.run();

        match parser.errors {
            None => Ok(parser.instructions),
            Some(err) => Err(err),
        }
    }

    fn run(&mut self) {
        let mut instruction_count = 0;

        while self.idx < self.tokens.len() {
            match &self.tokens[self.idx] {
                Token::Label(label) => {
                    if let Some(old_instruction_idx) = self.labels.insert(label, instruction_count) {
                        self.add_error(ParserError::DuplicateLabel {
                            idx: instruction_count,
                            old_idx: old_instruction_idx,
                        });
                    }
                }
                Token::Instruction(inst) => {
                    self.parse_instruction(inst);
                    instruction_count += 1;
                }
                Token::End => break,
                token => self.add_error(ParserError::InvalidToken {
                    idx: self.idx,
                    expected: "Label or Instruction",
                    got: format!("{token:?}"),
                }),
            }

            self.idx += 1;
        }
    }

    #[inline]
    fn add_error(&mut self, err: ParserError) {
        self.errors.get_or_insert_default().push(err);
    }

    fn parse_instruction(&mut self, instruction: &str) {
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
                inst: instruction.to_string(),
            }),
        }
    }

    fn expect_destination(&mut self, instr: ASMJumpInstruction) {
        self.idx += 1;

        if let Some(Token::Label(label)) = self.tokens.get(self.idx) {
            match self.labels.get(label.as_str()) {
                Some(&idx) => match idx.try_into() {
                    Ok(idx) => {
                        self.instructions.push(Instruction::from_jump_instruction(instr, idx));
                    }
                    Err(_) => {
                        self.add_error(ParserError::LabelIndexToWordConversionFailed {
                            idx: self.idx,
                            label: label.clone(),
                        });
                    }
                },
                None => self.add_error(ParserError::LabelNotFound {
                    idx: self.idx,
                    label: label.clone(),
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
        match self.get_next() {
            Some(Token::Register(reg)) => reg.parse::<Register>().map_err(ParserError::RegisterParsing),
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
        match self.get_next() {
            Some(Token::Register(reg)) => Ok(Operand::Register(reg.parse().map_err(ParserError::RegisterParsing)?)),
            Some(Token::Literal(lit)) => Ok(Operand::Value(Self::convert_lit_to_val(lit)?)),
            _ => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Register or Literal",
                got: self.current_token_string(),
            }),
        }
    }

    fn expect_word(&mut self) -> Result<W, ParserError> {
        match self.get_next() {
            Some(Token::Literal(lit)) => Ok(Self::convert_lit_to_val(lit)?),
            _ => Err(ParserError::InvalidToken {
                idx: self.idx,
                expected: "Literal",
                got: self.current_token_string(),
            }),
        }
    }

    #[inline]
    fn get_next(&mut self) -> Option<&Token<'_>> {
        self.idx += 1;
        self.tokens.get(self.idx)
    }

    #[inline]
    fn current_token_string(&self) -> String {
        self.tokens
            .get(self.idx)
            .map_or_else(|| "End".to_string(), |token| format!("{token:?}"))
    }

    fn convert_lit_to_val(lit: &Literal<'_>) -> Result<W, ParserError> {
        match lit {
            Literal::Char(s) => Ok((*s as i32).into()),
            Literal::Binary(s) => W::from_str_radix(s, 2).map_err(ParserError::LiteralParsing),
            Literal::Boolean(s) => Ok(i32::from(*s).into()),
            Literal::Decimal(s) => W::from_str_radix(s, 10).map_err(ParserError::LiteralParsing),
            Literal::Hexadecimal(s) => W::from_str_radix(s, 16).map_err(ParserError::LiteralParsing),
            Literal::Octal(s) => W::from_str_radix(s, 8).map_err(ParserError::LiteralParsing),
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
}
