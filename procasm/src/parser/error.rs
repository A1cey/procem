use procem::register::RegisterError;
use std::num::{ParseIntError, TryFromIntError};
use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq, Clone)]
pub enum ParserError {
    #[error("No tokens to parse.")]
    EmptyTokenList,
    #[error("Invalid token at idx {idx}. Expected: {expected} Got: {got}.")]
    InvalidToken {
        idx: usize,
        expected: &'static str,
        got: String,
    },
    #[error("Duplicate label: First occurrence: {old_idx}, second occurrence {idx}.")]
    DuplicateLabel { idx: usize, old_idx: u64 },
    #[error("Unknown instruction at idx {idx}: {inst}.")]
    UnknownMnemonic { idx: usize, inst: String },
    #[error("Error while parsing register: {err}.")]
    RegisterParsing {
        #[from]
        err: RegisterError,
    },
    #[error("Error while parsing literal ({lit}): {err}.")]
    LiteralParsing { lit: String, err: ParseIntError },
    #[error("Strings cannot be converted to numeric values directly. You could use a hex representation instead.")]
    CannotConvertStrToVal,
    #[error("Cannot convert literal {literal} to u32. This is likely due to the literal being too large.\n{err}")]
    CannotConvertLiteralToU32 { literal: u64, err: TryFromIntError },
    #[error("Invalid section identifier: {identifier} at {idx}.")]
    InvalidSection { idx: usize, identifier: String },
    #[error("Invalid directive: got: {got}, allowed: {allowed}.")]
    InvalidDirective { got: String, allowed: String },
    #[error("Wrong directive: got: {got}, expected: {expected}.")]
    WrongDirective {
        idx: usize,
        got: String,
        expected: &'static str,
    },
    #[error("Expected Literal at idx {idx} but got nothing.")]
    TokenNotFound { idx: usize },
    #[error("Wrong section. Current section: {current}, expected: {expected}.")]
    WrongSection { current: String, expected: String },
    #[error("Found Label before first Section at idx: {idx}.")]
    LabelBeforeFirstSection { idx: usize },
    #[error(
        "Bss cannot reserve more than u64::MAX space but tries to reserve more at {idx}. Previous bss size: {prev_bss}, tried to add: {additional_bss}, result: {}.",
        u128::from(*prev_bss) + u128::from(*additional_bss)
    )]
    BssOverflow {
        idx: usize,
        prev_bss: u64,
        additional_bss: u64,
    },
}
