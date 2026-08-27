use ars::range::Range;

use crate::{parser::ParserError, tokenizer::ImmediateLiteralKind};

pub trait FromImmediateLiteral {
    /// Parse from an `ImmediateLiteral`.
    fn from_immediate_literal(
        lit: ImmediateLiteralKind,
        span: Range,
        token_idx: usize,
        input: &[u8],
    ) -> Result<Self, ParserError>
    where
        Self: Sized;
}

macro_rules! from_literal {
    ($($unsigned:ty),*) => {
        $(impl FromImmediateLiteral for $unsigned {
            fn from_immediate_literal(lit: ImmediateLiteralKind, span: Range, token_idx: usize, input: &[u8]) -> Result<Self, ParserError> {
                match lit {
                    ImmediateLiteralKind::Char => {
                        let raw_lit = &input[span];
                        debug_assert_eq!(raw_lit.len(), 1);
                        Ok(Self::from(raw_lit[0]))
                    }
                    ImmediateLiteralKind::Binary => {
                        let raw_lit = ::core::str::from_utf8(&input[span])
                            .map_err(|err| ParserError::ImmediateLiteralParsingUtf8 { token_idx, lit, err })?;
                        Self::from_str_radix(raw_lit, 2).map_err(|err| ParserError::ImmediateLiteralParsingInt {
                            token_idx,
                            lit,
                            err,
                        })
                    }
                    ImmediateLiteralKind::Decimal => {
                        let raw_lit = ::core::str::from_utf8(&input[span])
                            .map_err(|err| ParserError::ImmediateLiteralParsingUtf8 { token_idx, lit, err })?;
                        if let Some(raw_lit) = raw_lit.strip_prefix('-') {
                            raw_lit.parse::<$unsigned>().map(<$unsigned>::wrapping_neg)
                        } else {
                            raw_lit.parse()
                        }
                        .map_err(|err| ParserError::ImmediateLiteralParsingInt { token_idx, lit, err })
                    }
                    ImmediateLiteralKind::Hexadecimal => {
                        let raw_lit = ::core::str::from_utf8(&input[span])
                            .map_err(|err| ParserError::ImmediateLiteralParsingUtf8 { token_idx, lit, err })?;
                        Self::from_str_radix(&raw_lit, 16).map_err(|err| ParserError::ImmediateLiteralParsingInt {
                            token_idx,
                            lit,
                            err,
                        })
                    }
                    ImmediateLiteralKind::Octal => {
                        let raw_lit = ::core::str::from_utf8(&input[span])
                            .map_err(|err| ParserError::ImmediateLiteralParsingUtf8 { token_idx, lit, err })?;
                        Self::from_str_radix(&raw_lit, 8).map_err(|err| ParserError::ImmediateLiteralParsingInt {
                            token_idx,
                            lit,
                            err,
                        })
                    }
                }
            }
        })*
    };
}

from_literal! {u8, u16, u32, u64, u128}
