use crate::{
    instruction::{
        Directive, Instruction,
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
        literal::FromImmediateLiteral,
        primitives::{ColonParser, CommaParser, DirectiveParser, IdentParser, ImmediateLiteralParser, NewlineParser},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct CodeParser;

impl<'input> Parser<'input> for CodeParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let mnemonic =
            Check(|state: &ParserState| check_section(Section::Code, state)).and(MnemonicParser).right().parse(input, state)?;

        let inst = match mnemonic {
            Mnemonic::NoArg(mnemonic) => match mnemonic {
                NoArgMnemonic::Nop => Instruction::Nop,
                NoArgMnemonic::Ret => Instruction::Ret,
            },
            // `_mnemonic` will be needed if there are future instructions like Adr
            Mnemonic::RegLabel(_mnemonic) => {
                let (reg, span) = RegisterParser.and(CommaParser).left().and(IdentParser).commit().parse(input, state)?;

                state.unlinked_instructions.push(UnlinkedInstruction::new(state.instructions.len(), span));
                Instruction::Adr { reg, addr: u64::MAX }
            }
            Mnemonic::RegOperand(mnemonic) => RegisterParser
                .and(CommaParser)
                .left()
                .and(OperandParser)
                .map(|(reg, op)| Instruction::from_reg_operand_mnemonic(mnemonic, reg, op))
                .commit()
                .parse(input, state)?,
            Mnemonic::Jump(mnemonic) => {
                let span = IdentParser.commit().parse(input, state)?;

                state.unlinked_instructions.push(UnlinkedInstruction::new(state.instructions.len(), span));
                Instruction::from_jump_mnemonic(mnemonic, u64::MAX)
            }
            Mnemonic::TwoOperand(mnemonic) => OperandParser
                .and(CommaParser)
                .left()
                .and(OperandParser)
                .map(|ops| Instruction::from_two_operand_mnemonic(mnemonic, ops.0, ops.1))
                .commit()
                .parse(input, state)?,
            Mnemonic::SingleOperand(mnemonic) => {
                OperandParser.map(|op| Instruction::from_single_operand_mnemonic(mnemonic, op)).commit().parse(input, state)?
            }

            Mnemonic::SingleReg(mnemonic) => {
                RegisterParser.map(|reg| Instruction::from_single_reg_mnemonic(mnemonic, reg)).commit().parse(input, state)?
            }
            Mnemonic::Rotate(mnemonic) => {
                let (reg, (literal, span, _token_idx)) =
                    RegisterParser.and(CommaParser).left().and(ImmediateLiteralParser).commit().parse(input, state)?;

                u64::from_immediate_literal(literal, span, state.idx, input.raw)
                    .and_then(|lit| {
                        lit.try_into().map_err(|err| ParserError::CannotConvertImmediateLiteralToU32 {
                            token_idx: state.idx,
                            lit,
                            err,
                        })
                    })
                    .map(|lit| Instruction::from_rotate_mnemonic(mnemonic, reg, lit))
                    .map_err(Error::IncompleteMatch)?
            }
            Mnemonic::Shift(mnemonic) => {
                let (reg, (lit, span, _token_idx)) =
                    RegisterParser.and(CommaParser).left().and(ImmediateLiteralParser).commit().parse(input, state)?;

                u32::from_immediate_literal(lit, span, state.idx, input.raw)
                    .map(|word| Instruction::from_shift_mnemonic(mnemonic, reg, word))
                    .map_err(Error::IncompleteMatch)?
            }
            Mnemonic::LoadOrStore(mnemonic) => RegisterParser
                .and(CommaParser)
                .left()
                .and(MemoryLocationParser)
                .map(|(reg, mem_location)| Instruction::from_ldr_or_str_mnemonic(mnemonic, reg, mem_location))
                .commit()
                .parse(input, state)?,
        };

        state.instructions.push(inst);

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.commit().parse(input, state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct DataParser;

impl<'input> Parser<'input> for DataParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let directive =
            Check(|state: &ParserState| check_section(Section::Data, state)).and(DirectiveParser).right().parse(input, state)?;

        match directive {
            Directive::Byte => ByteListParser.commit().parse(input, state),
            Directive::Hword => HwordListParser.commit().parse(input, state),
            Directive::Word => WordListParser.commit().parse(input, state),
            Directive::Dword => DwordListParser.commit().parse(input, state),
            Directive::Qword => QwordListParser.commit().parse(input, state),
            Directive::Ascii => AsciiListParser.commit().parse(input, state),
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
                token_idx: state.idx,
                directive,
                expected: "Only .byte, .hword, .word, .dword, .qword and .ascii are allowed in .data sections.",
            })),
        }?;

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.commit().parse(input, state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct BssParser;

impl<'input> Parser<'input> for BssParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let directive =
            Check(|state: &ParserState| check_section(Section::Bss, state)).and(DirectiveParser).right().parse(input, state)?;

        match directive {
            Directive::Space => SpaceListParser.commit().parse(input, state)?,
            Directive::Code => state.section = Section::Code,
            Directive::Data => state.section = Section::Data,
            Directive::Bss => state.section = Section::Bss,
            Directive::Byte | Directive::Hword | Directive::Word | Directive::Dword | Directive::Qword | Directive::Ascii => {
                return Err(Error::IncompleteMatch(ParserError::WrongDirective {
                    token_idx: state.idx,
                    directive,
                    expected: "Only .space is allowed in .bss section.",
                }));
            }
        }

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.commit().parse(input, state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct SectionParser;

impl<'input> Parser<'input> for SectionParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let directive = DirectiveParser.parse(input, state)?;

        match directive {
            Directive::Code => state.section = Section::Code,
            Directive::Data => state.section = Section::Data,
            Directive::Bss => state.section = Section::Bss,
            directive => Err(Error::NoMatch(ParserError::InvalidSection { token_idx: state.idx, section: directive }))?,
        }

        // Every line must end with a newline, because the tokenizer adds one to the last line in the file if its missing
        NewlineParser.commit().parse(input, state)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct LabelParser;

impl<'input> Parser<'input> for LabelParser {
    type Output = ();

    fn parse(self, input: ParserInput<'input>, state: &mut ParserState<'input>) -> Result<Self::Output, Error> {
        let span = IdentParser.and(ColonParser).left().parse(input, state)?;
        let ident = &input.raw[span];

        let mem_idx = match state.section {
            Section::Code => state.instructions.len() as u64,
            Section::Data => state.data.len() as u64,
            Section::Bss => state.bss,
            Section::Undefined => {
                return Err(Error::IncompleteMatch(ParserError::LabelBeforeFirstSection { token_idx: state.idx }));
            }
        };

        if let Some((_old_mem_idx, old_token_idx)) = state.labels.insert(ident, (mem_idx, state.idx)) {
            Err(Error::IncompleteMatch(ParserError::DuplicateLabel {
                first_token_idx: old_token_idx,
                second_token_idx: state.idx,
            }))?;
        }

        Ok(())
    }
}

fn check_section(expected: Section, state: &ParserState) -> Result<(), Error> {
    if state.section == expected {
        Ok(())
    } else {
        Err(Error::NoMatch(ParserError::WrongSection { token_idx: state.idx, section: state.section, expected }))
    }
}
