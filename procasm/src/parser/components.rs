use crate::instruction::memory_location::MemoryLocation;
use crate::instruction::mnemonics::Mnemonic;
use crate::instruction::operand::Operand;
use crate::instruction::unlinked::UnlinkedInstruction;
use crate::parser::combinators::{Error, Parser, Value};
use crate::parser::literal::FromImmediateLiteral;
use crate::parser::primitives::{
    ClosedBracketParser, CommaParser, IdentParser, ImmediateLiteralParser, OpenBracketParser, StringLiteralParser,
};
use crate::parser::{ParserError, ParserInput, ParserState};

use procem::register::Register;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct RegisterParser;

impl<'input> Parser<'input> for RegisterParser {
    type Output = Register;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let span = IdentParser.parse(input, state)?;
        Register::try_from(&input.raw[span])
            .map_err(|err| ParserError::RegisterParsing { token_idx: state.idx, err })
            .map_err(Error::NoMatch)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MnemonicParser;

impl<'input> Parser<'input> for MnemonicParser {
    type Output = Mnemonic;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let span = IdentParser.parse(input, state)?;
        let ident = &input.raw[span];
        ident.try_into().map_err(|()| ParserError::UnknownMnemonic { token_idx: state.idx }).map_err(Error::NoMatch)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct OperandParser;

impl<'input> Parser<'input> for OperandParser {
    type Output = Operand;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        RegisterParser
            .map(|reg| Ok(Operand::Register(reg)))
            .or(ImmediateLiteralParser.map(|(lit, span, token_idx)| {
                u64::from_immediate_literal(lit, span, token_idx, input.raw) // TODO: This always u64?
                    .map(Operand::Value)
                    .map_err(Error::IncompleteMatch)
            }))
            .parse(input, state)?
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LabeledMemoryLocationParser;

impl<'input> Parser<'input> for LabeledMemoryLocationParser {
    type Output = MemoryLocation;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let span = IdentParser.parse(input, state)?;

        state.unlinked_instructions.push(UnlinkedInstruction::new(state.instructions.len(), span));

        Ok(MemoryLocation::Labeled(u64::MAX))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DirectMemoryLocationParser;

impl<'input> Parser<'input> for DirectMemoryLocationParser {
    type Output = MemoryLocation;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        OpenBracketParser
            .and(RegisterParser)
            .right()
            .and(CommaParser.and(OperandParser).right().or(Value(Operand::Value(0))))
            .and(ClosedBracketParser)
            .left()
            .map(|(base, offset)| MemoryLocation::Offset { base, offset })
            .parse(input, state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct MemoryLocationParser;

impl<'input> Parser<'input> for MemoryLocationParser {
    type Output = MemoryLocation;

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        LabeledMemoryLocationParser.or(DirectMemoryLocationParser).parse(input, state)
    }
}

macro_rules! immediate_literal_list_parser {
    ($list_parser_name: ident, $parser_name: ident, $ret_val: ty) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $parser_name;

        impl<'input> Parser<'input> for $parser_name {
            type Output = $ret_val;

            fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
                let (lit, span, token_idx) = ImmediateLiteralParser.parse(input, state)?;
                <$ret_val as crate::parser::literal::FromImmediateLiteral>::from_immediate_literal(
                    lit, span, token_idx, input.raw,
                )
                .map_err(Error::IncompleteMatch)
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
        pub struct $list_parser_name;

        impl<'input> Parser<'input> for $list_parser_name {
            type Output = ();

            fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
                let val = $parser_name.parse(input, state)?;
                state.data.extend_from_slice(&val.to_le_bytes());

                while CommaParser.parse(input, state) == Ok(()) {
                    let val = $parser_name.parse(input, state).map_err(Error::into_incomplete_match)?;
                    state.data.extend_from_slice(&val.to_le_bytes());
                }
                Ok(())
            }
        }
    };
}

// TODO: Maybe move the from_literal macro into the data_value_list_parser if the functions arent needed somewhere else
immediate_literal_list_parser!(ByteListParser, ByteParser, u8);
immediate_literal_list_parser!(HwordListParser, HwordParser, u16);
immediate_literal_list_parser!(WordListParser, WordParser, u32);
immediate_literal_list_parser!(DwordListParser, DwordParser, u64);
immediate_literal_list_parser!(QwordListParser, QwordParser, u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct AsciiListParser;

impl<'input> Parser<'input> for AsciiListParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let span = StringLiteralParser.parse(input, state)?;
        state.data.extend_from_slice(&input.raw[span]);

        while CommaParser.parse(input, state) == Ok(()) {
            let span = StringLiteralParser.parse(input, state).map_err(Error::into_incomplete_match)?;
            state.data.extend_from_slice(&input.raw[span]);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SpaceParser;

impl<'input> Parser<'input> for SpaceParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let (lit, span, token_idx) = ImmediateLiteralParser.parse(input, state)?;
        let space = u64::from_immediate_literal(lit, span, token_idx, input.raw).map_err(Error::IncompleteMatch)?;
        state.bss = state.bss.checked_add(space).ok_or(Error::IncompleteMatch(ParserError::BssOverflow {
            token_idx,
            prev_bss: state.bss,
            additional_bss: space,
        }))?;
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SpaceListParser;

impl<'input> Parser<'input> for SpaceListParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        SpaceParser.parse(input, state)?;

        while CommaParser.parse(input, state) == Ok(()) {
            SpaceParser.parse(input, state).map_err(Error::into_incomplete_match)?;
        }
        Ok(())
    }
}
