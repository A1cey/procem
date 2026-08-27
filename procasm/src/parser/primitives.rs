use ars::range::Range;

use crate::{
    instruction::directive::Directive,
    parser::{
        ParserError, ParserInput, ParserState,
        combinators::{Error, Parser},
    },
    tokenizer::{ImmediateLiteralKind, Token, TokenKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct IdentParser;

impl<'input> Parser<'input> for IdentParser {
    type Output = Range;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        match input.tokens.get(state.idx) {
            Some(Token { kind: TokenKind::Identifier, span }) => {
                state.idx += 1;
                Ok(*span)
            }
            Some(_) => Err(ParserError::InvalidToken { expected: "Identifier", got_token_idx: state.idx }),
            None => Err(ParserError::TokenNotFound { token_idx: state.idx }),
        }
        .map_err(Error::NoMatch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct EndParser;

impl<'input> Parser<'input> for EndParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        match input.tokens.get(state.idx) {
            Some(Token { kind: TokenKind::End, .. }) => {
                state.idx += 1;
                state.end = true;
                Ok(())
            }
            Some(_) => Err(ParserError::InvalidToken { expected: "End", got_token_idx: state.idx }),
            None => Err(ParserError::TokenNotFound { token_idx: state.idx }),
        }
        .map_err(Error::NoMatch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct ImmediateLiteralParser;

impl<'input> Parser<'input> for ImmediateLiteralParser {
    type Output = (ImmediateLiteralKind, Range, usize);

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        match input.tokens.get(state.idx) {
            Some(Token { kind: TokenKind::ImmediateLiteral(lit), span }) => {
                state.idx += 1;
                Ok((*lit, *span, state.idx))
            }
            Some(_) => Err(ParserError::InvalidToken { expected: "ImmediateLiteral", got_token_idx: state.idx }),
            None => Err(ParserError::TokenNotFound { token_idx: state.idx }),
        }
        .map_err(Error::NoMatch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct StringLiteralParser;

impl<'input> Parser<'input> for StringLiteralParser {
    type Output = Range;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        match input.tokens.get(state.idx) {
            Some(Token { kind: TokenKind::StringLiteral, span }) => {
                state.idx += 1;
                Ok(*span)
            }
            Some(_) => Err(ParserError::InvalidToken { expected: "StringLiteral", got_token_idx: state.idx }),
            None => Err(ParserError::TokenNotFound { token_idx: state.idx }),
        }
        .map_err(Error::NoMatch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DirectiveParser;

impl<'input> Parser<'input> for DirectiveParser {
    type Output = Directive;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        match input.tokens.get(state.idx) {
            Some(Token { kind: TokenKind::Directive, span }) => {
                Directive::try_from_slice(&input.raw[span], state.idx).inspect(|_| state.idx += 1).map_err(Error::IncompleteMatch)
            }
            Some(_) => Err(Error::NoMatch(ParserError::InvalidToken { expected: "Directive", got_token_idx: state.idx })),
            None => Err(Error::NoMatch(ParserError::TokenNotFound { token_idx: state.idx })),
        }
    }
}

macro_rules! simple_token_parser {
    ($name: ident, $token: ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $name;

        impl<'input> Parser<'input> for $name {
            type Output = ();

            fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
                match input.tokens.get(state.idx) {
                    Some(Token { kind: TokenKind::$token, .. }) => {
                        state.idx += 1;
                        Ok(())
                    }
                    Some(_) => Err(ParserError::InvalidToken { expected: stringify!($token), got_token_idx: state.idx }),
                    None => Err(ParserError::TokenNotFound { token_idx: state.idx }),
                }
                .map_err(Error::NoMatch)
            }
        }
    };
}

simple_token_parser!(ColonParser, Colon);
simple_token_parser!(CommaParser, Comma);
simple_token_parser!(OpenBracketParser, OpenBracket);
simple_token_parser!(ClosedBracketParser, ClosedBracket);
simple_token_parser!(NewlineParser, Newline);
