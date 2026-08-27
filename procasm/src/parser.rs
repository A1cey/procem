mod combinators;
mod components;
mod error;
mod literal;
mod primitives;
mod statements;

pub use error::ParserError;
use primitives::{EndParser, NewlineParser};
use statements::{CodeParser, LabelParser, SectionParser};
use std::{collections::HashMap, fmt::Display};

use crate::{
    instruction::{Instruction, unlinked::UnlinkedInstruction},
    parser::{
        combinators::{Error, Parser},
        statements::{BssParser, DataParser},
    },
    tokenizer::{Token, TokenKind},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Parsed<'input> {
    pub instructions: Vec<Instruction>,
    labels: HashMap<&'input [u8], (u64, usize)>,
    pub unlinked_instructions: Vec<UnlinkedInstruction>,
    pub data: Vec<u8>,
    bss: u64,
}

impl<'input> From<ParserState<'input>> for Parsed<'input> {
    fn from(state: ParserState<'input>) -> Self {
        Parsed {
            instructions: state.instructions,
            labels: state.labels,
            unlinked_instructions: state.unlinked_instructions,
            data: state.data,
            bss: state.bss,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ParserInput<'input> {
    pub raw: &'input [u8],
    pub tokens: &'input [Token],
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct ParserState<'input> {
    idx: usize,
    end: bool,
    section: Section,
    labels: HashMap<&'input [u8], (u64, usize)>,
    instructions: Vec<Instruction>,
    unlinked_instructions: Vec<UnlinkedInstruction>,
    data: Vec<u8>,
    bss: u64,
}

impl Parsed<'_> {
    #[inline]
    #[must_use]
    pub const fn labels(&self) -> &HashMap<&[u8], (u64, usize)> {
        &self.labels
    }

    #[inline]
    #[must_use]
    pub const fn bss(&self) -> u64 {
        self.bss
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum Section {
    Bss,
    Data,
    Code,
    #[default]
    Undefined,
}

impl Display for Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bss => write!(f, "Bss"),
            Self::Data => write!(f, "Data"),
            Self::Code => write!(f, "Code"),
            Self::Undefined => write!(f, "Undefined"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
struct ProcasmParser;

impl<'input> Parser<'input> for ProcasmParser {
    type Output = ();

    // TODO: This results in discarding every error and replacing it with expected Newline, because NewlineParser is last
    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        CodeParser
            .or(BssParser)
            .or(DataParser)
            .or(LabelParser)
            .or(SectionParser)
            .or(EndParser)
            .or(NewlineParser)
            .parse(input, state)
    }
}

/// Parse tokens into a list of instructions.
///
/// # Errors
/// Returns a list of errors that occurred during parsing.
pub fn parse(input: ParserInput) -> Result<Parsed, Vec<ParserError>> {
    let mut state = ParserState::default();
    let mut errors = Vec::new();

    let parser = ProcasmParser;

    while !state.end && state.idx < input.tokens.len() {
        if let Err(err) = parser.parse(input, &mut state) {
            errors.push(err.inner());
            skip_to_next_line(input.tokens, &mut state);
        }
    }

    if errors.is_empty() { Ok(Parsed::from(state)) } else { Err(errors) }
}

//TODO: Maybe make this a parser?
fn skip_to_next_line(tokens: &[Token], state: &mut ParserState) {
    while let Some(token) = tokens.get(state.idx) {
        state.idx += 1;
        match *token {
            Token { kind: TokenKind::Newline, .. } => break,
            Token { kind: TokenKind::End, .. } => {
                state.end = true;
                break;
            }
            Token { kind: TokenKind::Identifier, .. }
            | Token { kind: TokenKind::ImmediateLiteral(_), .. }
            | Token { kind: TokenKind::StringLiteral, .. }
            | Token { kind: TokenKind::Comma, .. }
            | Token { kind: TokenKind::Colon, .. }
            | Token { kind: TokenKind::OpenBracket, .. }
            | Token { kind: TokenKind::ClosedBracket, .. }
            | Token { kind: TokenKind::Directive, .. } => {}
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        instruction::{Instruction, memory_location::MemoryLocation, operand::Operand},
        parser::{Parser, ParserError, ParserInput, ParserState, ProcasmParser, Section, combinators::Error, parse},
        tokenizer::Tokenizer,
    };
    use ars::range::Range;
    use pretty_assertions_sorted::assert_eq;
    use procem::register::Register;

    #[test]
    fn parse_section() {
        let input = b"
            .code
            .bss
            .data
            .Bss
            .CODE
            .Invalid
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let input = ParserInput { raw: input, tokens: &tokens };
        let mut state = ParserState::default();
        let p = ProcasmParser;

        macro_rules! check {
            ($variant:ident, $state:expr) => {
                match state.section {
                    Section::$variant => {}
                    s => panic!("Expected section {}, got: {s}", stringify!($variant)),
                }
            };
        }

        check!(Undefined, state);
        p.parse(input, &mut state).unwrap(); // Skip initial Newline
        p.parse(input, &mut state).unwrap();
        check!(Code, state);
        p.parse(input, &mut state).unwrap();
        check!(Bss, state);
        p.parse(input, &mut state).unwrap();
        check!(Data, state);
        p.parse(input, &mut state).unwrap();
        check!(Bss, state);
        p.parse(input, &mut state).unwrap();
        check!(Code, state);
        match p.parse(input, &mut state) {
            Ok(()) => panic!("Expected error, but succeeded"),
            Err(Error::NoMatch(err)) => panic!("Expected IncompleteMatch error, but got {err:?}"),
            Err(Error::IncompleteMatch(err)) => {
                assert_eq!(err, ParserError::InvalidDirective { directive: "Invalid".to_string(), token_idx: 11 })
            }
        }
        check!(Code, state);
    }

    #[test]
    fn parse_bss() {
        let input = b"
            .bss
            a:
                .space 5
                .space 10
            b:
                .space 5, 0xA
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = parse(ParserInput { raw: input, tokens: &tokens }).unwrap();

        assert_eq!(parsed.instructions.len(), 0);
        assert_eq!(parsed.labels().len(), 2);
        assert_eq!(parsed.data.len(), 0);
        assert_eq!(parsed.bss, 5 + 10 + 5 + 10);
        assert_eq!(parsed.labels[b"a".as_slice()], (0, 5));
        assert_eq!(parsed.labels[b"b".as_slice()], (5 + 10, 14));
    }

    #[test]
    fn parse_data() {
        let input = b"
            .data
            a:
                .byte 5
                .hword 5
                .word 5
                .dword 5
                .qword 5
            b:
                .ascii \"Hello World!\", \"\0\"
            c:
                .word 5, 0xA
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = parse(ParserInput { raw: input, tokens: &tokens }).unwrap();

        assert_eq!(parsed.instructions.len(), 0);
        assert_eq!(parsed.unlinked_instructions.len(), 0);
        assert_eq!(parsed.bss(), 0);
        assert_eq!(parsed.data.len(), 1 + 2 + 4 + 8 + 16 + b"Hello World!".len() + b"\0".len() + 4 + 4); // byte, hword, word, dword, qword, 2 ascii, 2 word allocations
        assert_eq!(parsed.labels.len(), 3);
        assert_eq!(parsed.labels[b"a".as_slice()], (0, 5));
        assert_eq!(parsed.labels[b"b".as_slice()], (1 + 2 + 4 + 8 + 16, 23));
        assert_eq!(parsed.labels[b"c".as_slice()], (1 + 2 + 4 + 8 + 16 + b"Hello World!".len() as u64 + b"\0".len() as u64, 31));
    }

    #[test]
    fn parse_data_multi_alloc() {
        let input = b"
            .data
            a:
                .byte 5,5
                .hword 5, 5
                .word 5, 5,5
                .dword 5, 5
                .qword 5, 5
            b:
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = parse(ParserInput { raw: input, tokens: &tokens }).unwrap();

        assert_eq!(parsed.instructions.len(), 0);
        assert_eq!(parsed.unlinked_instructions.len(), 0);
        assert_eq!(parsed.bss(), 0);
        assert_eq!(parsed.data.len(), 1 + 1 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 16 + 16); // 2 byte, 2 hword, 3 word, 2 dword, 2 qword
        assert_eq!(parsed.labels.len(), 2);
        assert_eq!(parsed.labels[b"a".as_slice()], (0, 5));
        assert_eq!(parsed.labels[b"b".as_slice()], (1 + 1 + 2 + 2 + 4 + 4 + 4 + 8 + 8 + 16 + 16, 35));
    }

    #[test]
    fn parse_all_sections() {
        let input = b"
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
            f:
                add R1, 0o1
                jmp c
            ";
        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = parse(ParserInput { raw: input, tokens: &tokens }).unwrap();

        assert_eq!(parsed.instructions.len(), 6);
        assert_eq!(parsed.labels().len(), 6);
        assert_eq!(parsed.data.len(), 4 + 4); // 2 32bit allocations
        assert_eq!(parsed.bss(), 5 + 10 + 5);

        // code
        assert_eq!(parsed.labels()[b"a".as_slice()], (0, 5));
        assert_eq!(parsed.labels()[b"c".as_slice()], (2, 30));
        assert_eq!(parsed.labels()[b"f".as_slice()], (4, 68));
        // bss
        assert_eq!(parsed.labels()[b"b".as_slice()], (0, 19));
        // data
        assert_eq!(parsed.labels()[b"d".as_slice()], (0, 44));
        assert_eq!(parsed.labels()[b"e".as_slice()], (4, 59));
    }

    #[test]
    fn parse_str_instr() {
        let input = b"
            .code
            str r0, [r1]
            str r0, [r1, r2]
            str r0, [r1, 5]
            str r0, [r1, -1]
            str r0, [r1, 0xa]
            str r0, data
            ";

        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = parse(ParserInput { raw: input, tokens: &tokens }).unwrap();

        assert_eq!(parsed.instructions.len(), 6);
        assert_eq!(parsed.unlinked_instructions.len(), 1);
        assert_eq!(&input[parsed.unlinked_instructions[0].label()], b"data".as_slice());

        let mut insts = parsed.instructions.iter();

        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Str { from: Register::R0, to: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(0) } }
        );
        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Str {
                from: Register::R0,
                to: MemoryLocation::Offset { base: Register::R1, offset: Operand::Register(Register::R2) }
            }
        );
        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Str { from: Register::R0, to: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(5) } }
        );
        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Str {
                from: Register::R0,
                to: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(-1isize as u64) }
            }
        );
        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Str {
                from: Register::R0,
                to: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(10) }
            }
        );
        assert_eq!(*insts.next().unwrap(), Instruction::Str { from: Register::R0, to: MemoryLocation::Labeled(u64::MAX) });
    }

    #[test]
    fn parse_ldr_instr() {
        let input = b"
            .code
            ldr r0, [r1]
            ldr r0, [r1, r2]
            ldr r0, [r1, 5]
            ldr r0, [r1, -1]
            ldr r0, [r1, 0xa]
            ldr r0, data
            ";

        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = parse(ParserInput { raw: input, tokens: &tokens }).unwrap();

        assert_eq!(parsed.instructions.len(), 6);
        assert_eq!(parsed.unlinked_instructions.len(), 1);
        assert_eq!(&input[parsed.unlinked_instructions[0].label()], b"data".as_slice());

        let mut insts = parsed.instructions.iter();

        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Ldr { to: Register::R0, from: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(0) } }
        );
        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Ldr {
                to: Register::R0,
                from: MemoryLocation::Offset { base: Register::R1, offset: Operand::Register(Register::R2) }
            }
        );
        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Ldr { to: Register::R0, from: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(5) } }
        );

        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Ldr {
                to: Register::R0,
                from: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(-1isize as u64) }
            }
        );
        assert_eq!(
            *insts.next().unwrap(),
            Instruction::Ldr {
                to: Register::R0,
                from: MemoryLocation::Offset { base: Register::R1, offset: Operand::Value(10) }
            }
        );
        assert_eq!(*insts.next().unwrap(), Instruction::Ldr { to: Register::R0, from: MemoryLocation::Labeled(u64::MAX) });
    }

    #[test]
    fn parse_adr_instruction() {
        let input = b"
            .code
            adr r0, data
            ";

        let tokens = Tokenizer::tokenize(input).unwrap();
        let parsed = parse(ParserInput { raw: input, tokens: &tokens }).unwrap();

        assert_eq!(parsed.instructions.len(), 1);
        assert_eq!(parsed.unlinked_instructions.len(), 1);
        assert_eq!(parsed.instructions[0], Instruction::Adr { reg: Register::R0, addr: u64::MAX });
    }
}
