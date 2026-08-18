use ars::range::Range;
use procem::register::RegisterError;
use std::{
    num::{ParseIntError, TryFromIntError},
    str::Utf8Error,
};

use crate::{
    instruction::directive::Directive,
    parser::Section,
    tokenizer::{ImmediateLiteralKind, Token},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ParserError {
    InvalidToken { got: Token, expected: &'static str },
    DuplicateLabel { first_occurence: Range, second_occurence: Range },
    UnknownMnemonic { span: Range },
    RegisterParsing { span: Range, err: RegisterError },
    ImmediateLiteralParsingInt { span: Range, lit: ImmediateLiteralKind, err: ParseIntError },
    ImmediateLiteralParsingUtf8 { span: Range, lit: ImmediateLiteralKind, err: Utf8Error },
    CannotConvertImmediateLiteralToU32 { span: Range, lit: u64, err: TryFromIntError },
    InvalidSection { span: Range, section: Directive },
    WrongSection { span: Range, section: Section, expected: Section },
    InvalidDirective { span: Range, directive: String },
    WrongDirective { span: Range, directive: Directive, expected: &'static str },
    TokenNotFound { span: Range },
    LabelBeforeFirstSection { span: Range },
    BssOverflow { span: Range, prev_bss: u64, additional_bss: u64 },
}

impl ParserError {
    #[must_use]
    pub fn render(self, input: &[u8]) -> String {
        match self {
            Self::InvalidToken { got, expected } => {
                format!("Invalid token. Expected {expected}, but got {} ({:?}).", got.resolve(input), got.span)
            }
            Self::DuplicateLabel { first_occurence, second_occurence } => {
                format!(
                    "Duplicate label found. First occurrence: {} ({first_occurence:?}), second occurence at {} ({second_occurence:?}).",
                    String::from_utf8_lossy(&input[first_occurence]),
                    String::from_utf8_lossy(&input[second_occurence])
                )
            }
            Self::UnknownMnemonic { span } => format!("Unknown Mnemonic: {} ({span:?}).", String::from_utf8_lossy(&input[span])),
            Self::RegisterParsing { span, err } => {
                format!("Error while parsing register: {} ({span:?}).\n{err}", String::from_utf8_lossy(&input[span]))
            }
            Self::ImmediateLiteralParsingInt { span, lit, err } => {
                format!("Error while parsing immediate literal into integer: {} ({span:?}).\n{err}", lit.resolve(input, span),)
            }
            Self::ImmediateLiteralParsingUtf8 { span, lit, err } => {
                format!("UTF8 error while parsing immediate literal from input: {} ({span:?}).\n{err}", lit.resolve(input, span))
            }
            Self::CannotConvertImmediateLiteralToU32 { span, lit, err } => {
                format!(
                    "Cannot convert immediate literal {lit} into u32. This is likely due to the literal being too large.\nLiteral defined here: {span:?}.\n{err}",
                )
            }
            Self::InvalidSection { span, section } => {
                format!("Invalid section: {section} ({span:?}).")
            }
            Self::WrongSection { span, section: directive, expected } => {
                format!("Wrong section. Got {directive} ({span:?}), expected {expected}.")
            }
            Self::InvalidDirective { span, directive } => {
                format!("Invalid directive: {directive} ({span:?}).")
            }
            Self::WrongDirective { span, directive, expected } => {
                format!("Wrong directive. Got {directive} ({span:?}), expected {expected}.")
            }
            Self::TokenNotFound { span } => format!("Expected token at {span:?} but got nothing."),
            Self::LabelBeforeFirstSection { span } => {
                format!("Found label before first section: {} ({span:?})", String::from_utf8_lossy(&input[span]))
            }
            Self::BssOverflow { span, prev_bss, additional_bss } => format!(
                "Bss overflow. Bss cannot reserve more than u64::MAX space but tries to reserve more at {span:?}.\nPrevious bss size: {prev_bss}, tried to add: {additional_bss}, result: {}.",
                u128::from(prev_bss) + u128::from(additional_bss)
            ),
        }
    }
}
