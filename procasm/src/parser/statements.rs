use crate::{
    instruction::{
        Instruction,
        directive::Directive,
        mnemonics::{Mnemonic, NoArgMnemonic},
        unlinked::UnlinkedInstruction,
    },
    parser::{
        ParserError, ParserInput, ParserState, Section,
        combinators::{Check, Error, Parser},
        components::{
            AsciiListParser, ByteListParser, DwordListParser, HwordListParser, MemoryLocationParser, MnemonicParser,
            OperandParser, QwordListParser, RegisterParser, SpaceListParser, WordListParser,
        },
        primitives::{ColonParser, CommaParser, DirectiveParser, IdentParser, ImmediateLiteralParser, NewlineParser},
        u64_from_literal,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct CodeParser;

impl<'input> Parser<'input> for CodeParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let mnemonic = Check(|state: &ParserState| check_section(Section::Code, state))
            .and(MnemonicParser)
            .right()
            .parse(input, state)?;

        let inst = match mnemonic {
            Mnemonic::NoArg(mnemonic) => match mnemonic {
                NoArgMnemonic::Nop => Instruction::Nop,
                NoArgMnemonic::Ret => Instruction::Ret,
            },
            // `_mnemonic` will be needed if there are future instructions like Adr
            Mnemonic::RegLabel(_mnemonic) => {
                let (reg, range) = RegisterParser
                    .and(CommaParser)
                    .left()
                    .and(IdentParser)
                    .parse(input, state)
                    .map_err(Error::into_incomplete_match)?;

                state
                    .unlinked_instructions
                    .push(UnlinkedInstruction::new(state.instructions.len(), range));
                Instruction::Adr { reg, addr: u64::MAX }
            }
            Mnemonic::RegOperand(mnemonic) => RegisterParser
                .and(CommaParser)
                .left()
                .and(OperandParser)
                .map(|(reg, op)| Instruction::from_reg_operand_mnemonic(mnemonic, reg, op))
                .parse(input, state)
                .map_err(Error::into_incomplete_match)?,
            Mnemonic::Jump(mnemonic) => {
                let range = IdentParser.parse(input, state).map_err(Error::into_incomplete_match)?;

                state
                    .unlinked_instructions
                    .push(UnlinkedInstruction::new(state.instructions.len(), range));
                Instruction::from_jump_mnemonic(mnemonic, u64::MAX)
            }
            Mnemonic::TwoOperand(mnemonic) => OperandParser
                .and(CommaParser)
                .left()
                .and(OperandParser)
                .map(|ops| Instruction::from_two_operand_mnemonic(mnemonic, ops.0, ops.1))
                .parse(input, state)
                .map_err(Error::into_incomplete_match)?,
            Mnemonic::SingleOperand(mnemonic) => OperandParser
                .map(|op| Instruction::from_single_operand_mnemonic(mnemonic, op))
                .parse(input, state)
                .map_err(Error::into_incomplete_match)?,
            Mnemonic::SingleReg(mnemonic) => RegisterParser
                .map(|reg| Instruction::from_single_reg_mnemonic(mnemonic, reg))
                .parse(input, state)
                .map_err(Error::into_incomplete_match)?,
            Mnemonic::Rotate(mnemonic) => {
                let (reg, literal) = RegisterParser
                    .and(CommaParser)
                    .left()
                    .and(ImmediateLiteralParser)
                    .parse(input, state)
                    .map_err(Error::into_incomplete_match)?;

                u64_from_literal(literal, input.raw)
                    .and_then(|literal| {
                        literal
                            .try_into()
                            .map_err(|err| ParserError::CannotConvertLiteralToU32 { literal, err })
                    })
                    .map(|lit| Instruction::from_rotate_mnemonic(mnemonic, reg, lit))
                    .map_err(Error::IncompleteMatch)?
            }
            Mnemonic::Shift(mnemonic) => {
                let (reg, lit) = RegisterParser
                    .and(CommaParser)
                    .left()
                    .and(ImmediateLiteralParser)
                    .parse(input, state)
                    .map_err(Error::into_incomplete_match)?;

                u64_from_literal(lit, input.raw)
                    .map(|word| Instruction::from_shift_mnemonic(mnemonic, reg, word))
                    .map_err(Error::IncompleteMatch)?
            }
            Mnemonic::LoadOrStore(mnemonic) => RegisterParser
                .and(CommaParser)
                .left()
                .and(MemoryLocationParser)
                .map(|(reg, mem_location)| Instruction::from_ldr_or_str_mnemonic(mnemonic, reg, mem_location))
                .parse(input, state)
                .map_err(Error::into_incomplete_match)?,
        };

        state.instructions.push(inst);

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.parse(input, state).map_err(Error::into_incomplete_match)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct DataParser;

impl<'input> Parser<'input> for DataParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let directive = Check(|state: &ParserState| check_section(Section::Data, state))
            .and(DirectiveParser)
            .right()
            .parse(input, state)?;

        match directive {
            Directive::Byte => ByteListParser.parse(input, state),
            Directive::Hword => HwordListParser.parse(input, state),
            Directive::Word => WordListParser.parse(input, state),
            Directive::Dword => DwordListParser.parse(input, state),
            Directive::Qword => QwordListParser.parse(input, state),
            Directive::Ascii => AsciiListParser.parse(input, state),
            Directive::Code => {
                state.section = Section::Code;
                Ok(())
            }
            Directive::Data => {
                state.section = Section::Data;
                Ok(())
            }
            Directive::Bss => {
                state.section = Section::Bss;
                Ok(())
            }
            Directive::Space => Err(Error::IncompleteMatch(ParserError::WrongDirective {
                idx: state.idx,
                got: directive.to_string(),
                expected: "Only .byte, .hword, .word, .dword, .qword and .ascii are allowed in .data sections.",
            })),
        }
        .map_err(Error::into_incomplete_match)?;

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.parse(input, state).map_err(Error::into_incomplete_match)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct BssParser;

impl<'input> Parser<'input> for BssParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let directive = Check(|state: &ParserState| check_section(Section::Bss, state))
            .and(DirectiveParser)
            .right()
            .parse(input, state)?;

        match directive {
            Directive::Space => SpaceListParser
                .parse(input, state)
                .map_err(Error::into_incomplete_match)?,
            Directive::Code => state.section = Section::Code,
            Directive::Data => state.section = Section::Data,
            Directive::Bss => state.section = Section::Bss,
            Directive::Byte
            | Directive::Hword
            | Directive::Word
            | Directive::Dword
            | Directive::Qword
            | Directive::Ascii => {
                return Err(Error::IncompleteMatch(ParserError::WrongDirective {
                    idx: state.idx,
                    got: directive.to_string(),
                    expected: "Only .space is allowed in .bss section.",
                }));
            }
        }

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.parse(input, state).map_err(Error::into_incomplete_match)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct SectionParser;

impl<'input> Parser<'input> for SectionParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        match DirectiveParser.parse(input, state)? {
            Directive::Code => state.section = Section::Code,
            Directive::Data => state.section = Section::Data,
            Directive::Bss => state.section = Section::Bss,
            dir => Err(Error::NoMatch(ParserError::InvalidSection {
                idx: state.idx,
                identifier: dir.to_string(),
            }))?,
        }

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.parse(input, state).map_err(Error::into_incomplete_match)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub(crate) struct LabelParser;

impl<'input> Parser<'input> for LabelParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let range = IdentParser.and(ColonParser).left().parse(input, state)?;
        let ident = &input.raw[range];

        let idx = match state.section {
            Section::Code => state.instructions.len() as u64,
            Section::Data => state.data.len() as u64,
            Section::Bss => state.bss,
            Section::Undefined => {
                return Err(Error::IncompleteMatch(ParserError::LabelBeforeFirstSection {
                    idx: state.idx,
                }));
            }
        };

        if let Some(old_idx) = state.labels.insert(ident, idx) {
            Err(Error::IncompleteMatch(ParserError::DuplicateLabel {
                idx: state.idx,
                old_idx,
            }))?;
        }

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.parse(input, state).map_err(Error::into_incomplete_match)
    }
}

fn check_section(expected: Section, state: &ParserState) -> Result<(), Error> {
    if state.section == expected {
        Ok(())
    } else {
        Err(Error::NoMatch(ParserError::WrongSection {
            current: state.section.to_string(),
            expected: expected.to_string(),
        }))
    }
}
