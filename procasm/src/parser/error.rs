use ars::range::Range;
use procem::register::RegisterError;
use std::{
    num::{ParseIntError, TryFromIntError},
    str::Utf8Error,
};

use crate::{
    instruction::Directive,
    parser::{ParserInput, Section},
    tokenizer::{ImmediateLiteralKind, Token, TokenKind},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParserError {
    InvalidToken { got_token_idx: usize, expected: &'static str },
    DuplicateLabel { first_token_idx: usize, second_token_idx: usize },
    UnknownMnemonic { token_idx: usize },
    RegisterParsing { token_idx: usize, err: RegisterError },
    ImmediateLiteralParsingInt { token_idx: usize, lit: ImmediateLiteralKind, err: ParseIntError },
    ImmediateLiteralParsingUtf8 { token_idx: usize, lit: ImmediateLiteralKind, err: Utf8Error },
    CannotConvertImmediateLiteralToU32 { token_idx: usize, lit: u64, err: TryFromIntError },
    InvalidSection { token_idx: usize, section: Directive },
    WrongSection { token_idx: usize, section: Section, expected: Section },
    InvalidDirective { token_idx: usize, directive: String },
    WrongDirective { token_idx: usize, directive: Directive, expected: &'static str },
    TokenNotFound { token_idx: usize },
    LabelBeforeFirstSection { token_idx: usize },
    BssOverflow { token_idx: usize, prev_bss: u64, additional_bss: u64 },
}

impl ParserError {
    #[must_use]
    pub fn render(self, input: ParserInput) -> String {
        match self {
            Self::InvalidToken { got_token_idx, expected } => {
                let token = input.tokens[got_token_idx];
                let line = Self::line(input.tokens, got_token_idx);
                format!(
                    "Invalid token. Expected {expected}, but got {} ({:?}).\n{}",
                    token.resolve(input.raw),
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::DuplicateLabel { first_token_idx, second_token_idx } => {
                let first_token = input.tokens[first_token_idx];
                let second_token = input.tokens[second_token_idx];
                let first_line = Self::line(input.tokens, first_token_idx);
                let second_line = Self::line(input.tokens, second_token_idx);
                format!(
                    "Duplicate label found. First occurrence: {} ({:?})\n{}\nSecond occurence at {} ({:?})\n{}",
                    String::from_utf8_lossy(&input.raw[first_token.span]),
                    first_token.span,
                    String::from_utf8_lossy(&input.raw[first_line]),
                    String::from_utf8_lossy(&input.raw[second_token.span]),
                    second_token.span,
                    String::from_utf8_lossy(&input.raw[second_line])
                )
            }
            Self::UnknownMnemonic { token_idx } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "Unknown Mnemonic: {} ({:?}).\n{}",
                    String::from_utf8_lossy(&input.raw[token.span]),
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::RegisterParsing { token_idx, err } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "Error while parsing register: {} ({:?}).\n{}\n{err}",
                    String::from_utf8_lossy(&input.raw[token.span]),
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::ImmediateLiteralParsingInt { token_idx, lit, err } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "Error while parsing immediate literal into integer: {} ({:?}).\n{}\n{err}",
                    lit.resolve(input.raw, token.span),
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::ImmediateLiteralParsingUtf8 { token_idx, lit, err } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "UTF8 error while parsing immediate literal from input: {} ({:?}).\n{}\n{err}",
                    lit.resolve(input.raw, token.span),
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::CannotConvertImmediateLiteralToU32 { token_idx, lit, err } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "Cannot convert immediate literal {lit} into u32. This is likely due to the literal being too large.\nLiteral defined here: {:?}.\n{}\n{err}",
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::InvalidSection { token_idx, section } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!("Invalid section: {section} ({:?}).\n{}", token.span, String::from_utf8_lossy(&input.raw[line]))
            }
            Self::WrongSection { token_idx, section: directive, expected } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "Wrong section. Got {directive} ({:?}), expected {expected}.\n{}",
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::InvalidDirective { token_idx, directive } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!("Invalid directive: {directive} ({:?}).\n{}", token.span, String::from_utf8_lossy(&input.raw[line]))
            }
            Self::WrongDirective { token_idx, directive, expected } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "Wrong directive. Got {directive} ({:?}), expected {expected}.\n{}",
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::TokenNotFound { token_idx } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!("Expected token at {:?} but got nothing.\n{}", token.span, String::from_utf8_lossy(&input.raw[line]))
            }
            Self::LabelBeforeFirstSection { token_idx } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "Found label before first section: {} ({:?}).\n{}",
                    String::from_utf8_lossy(&input.raw[token.span]),
                    token.span,
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
            Self::BssOverflow { token_idx, prev_bss, additional_bss } => {
                let token = input.tokens[token_idx];
                let line = Self::line(input.tokens, token_idx);
                format!(
                    "Bss overflow. Bss cannot reserve more than u64::MAX space but tries to reserve more at {:?}.\nPrevious bss size: {prev_bss}, tried to add: {additional_bss}, result: {}.\n{}",
                    token.span,
                    u128::from(prev_bss) + u128::from(additional_bss),
                    String::from_utf8_lossy(&input.raw[line])
                )
            }
        }
    }

    fn line(tokens: &[Token], token_idx: usize) -> Range {
        // token_idx exclusive because if curr token is already a newline we still want the previous one
        let prev_newline = tokens[..token_idx].iter().rev().find(|t| t.kind == TokenKind::Newline).map(|t| t.span);
        let next_newline = tokens[token_idx..].iter().find(|t| t.kind == TokenKind::Newline).map(|t| t.span);

        let line_start = match prev_newline {
            Some(Range(_start, end)) => end,
            None => 0,
        };

        let Some(Range(line_end, _)) = next_newline else {
            unreachable!("There is always a following newline in `tokens` or the current token is a newline.")
        };

        Range::from(line_start..line_end)
    }
}
