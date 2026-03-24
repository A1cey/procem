use std::mem;

use procem::program::{Bss, Code, Data, Header, Program};
use thiserror::Error;

use crate::{
    AssembledProgram,
    instruction::{Instruction, unlinked::UnlinkedInstruction},
    parser::Parsed,
    word::ProcasmWord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linker<'input, const MEM_SIZE: usize, W> {
    errors: Vec<LinkerError>,
    input: &'input [u8],
    parsed: Parsed<'input, W>,
}

impl<'input, const MEM_SIZE: usize, W: ProcasmWord> Linker<'input, MEM_SIZE, W> {
    pub fn link(
        input: &'input [u8],
        parsed: Parsed<'input, W>,
    ) -> Result<AssembledProgram<MEM_SIZE, W>, Vec<LinkerError>> {
        let mut linker = Self {
            errors: Vec::new(),
            input,
            parsed,
        };

        let program = linker.run();

        if !linker.errors.is_empty() {
            return Err(linker.errors);
        }

        Ok(program)
    }

    fn run(&mut self) -> AssembledProgram<MEM_SIZE, W> {
        let unlinked_instructions = mem::take(self.parsed.mut_unlinked_instructions());

        for unlinked_instruction in unlinked_instructions {
            self.link_instruction(unlinked_instruction);
        }

        let header = self.create_header();
        let data = self.create_data();
        let bss = self.create_bss();
        let code = self.create_code();

        Program::new(header, data, bss, code)
    }

    fn link_instruction(&mut self, unlinked_instruction: UnlinkedInstruction) {
        let label = &self.input[unlinked_instruction.label()];

        let destination = match self.parsed.instruction_labels().get(label) {
            Some(destination) => *destination,
            None => {
                return self.errors.push(LinkerError::LabelNotFound {
                    idx: unlinked_instruction.instr_idx(),
                    label: String::from_utf8_lossy(label).to_string(),
                });
            }
        };

        let Ok(destination) = destination.try_into() else {
            return self.errors.push(LinkerError::LabelIndexToWordConversionFailed {
                idx: unlinked_instruction.instr_idx(),
                label: String::from_utf8_lossy(label).to_string(),
            });
        };

        let instruction = self
            .parsed
            .mut_instructions()
            .get_mut(unlinked_instruction.instr_idx())
            .expect("The instruction index is always in range of the instructions.");

        let linked_instruction = match instruction {
            Instruction::Jump { to: _, condition } => Instruction::Jump {
                to: destination,
                condition: *condition,
            },
            instruction => unreachable!("Only jump instructions have to be linked, not {instruction:?} instructions."),
        };

        *instruction = linked_instruction;
    }

    #[must_use]
    #[inline]
    fn create_header(&mut self) -> Header<W> {
        let init_pc = self.get_init_pc();
        let init_sp = self.get_init_sp();
        Header::new(
            init_pc.unwrap_or_else(|| W::from(isize::MAX)),
            init_sp.unwrap_or_else(|| W::from(isize::MAX)),
        )
    }

    #[inline]
    fn get_init_pc(&mut self) -> Option<W> {
        const START_LABEL: &[u8] = b"_start";

        let idx = self.parsed.instruction_labels().get(START_LABEL);

        let init_pc = match idx {
            None => {
                self.errors.push(LinkerError::StartSymbolNotFound);
                return None;
            }
            Some(idx) => {
                if let Ok(idx) = (*idx).try_into() {
                    idx
                } else {
                    self.errors.push(LinkerError::LabelIndexToWordConversionFailed {
                        idx: *idx,
                        label: String::from_utf8_lossy(START_LABEL).to_string(),
                    });
                    return None;
                }
            }
        };

        Some(init_pc)
    }

    #[inline]
    fn get_init_sp(&mut self) -> Option<W> {
        // Stack starts at highest value in memory
        let Ok(init_sp) = { MEM_SIZE - 1 }.try_into() else {
            self.errors.push(LinkerError::MemorySizeToBig {
                mem_size: MEM_SIZE,
                max_word_value: W::from(isize::MAX).into(),
            });
            return None;
        };

        Some(init_sp)
    }

    #[must_use]
    #[inline]
    fn create_data(&mut self) -> Data<W, Vec<W>> {
        Data::new(W::from(0), mem::take(self.parsed.mut_data()))
    }

    #[must_use]
    fn create_bss(&mut self) -> Bss<W> {
        if self.parsed.bss() == 0 {
            return Bss::new(W::from(0), W::from(0));
        }

        let base_addr = if let Ok(base_addr) = self.parsed.data().len().try_into() {
            base_addr
        } else {
            self.errors.push(LinkerError::DataToBigForBss {
                data_size: self.parsed.data().len(),
                max_word_value: W::from(isize::MAX).into(),
                bss_size: self.parsed.bss(),
            });
            W::from(0)
        };

        let bss_size = if let Ok(base_addr) = self.parsed.bss().try_into() {
            base_addr
        } else {
            self.errors.push(LinkerError::BssToBig {
                bss_size: self.parsed.data().len(),
                max_word_value: W::from(isize::MAX).into(),
            });
            W::from(0)
        };

        Bss::new(base_addr, bss_size)
    }

    #[must_use]
    #[inline]
    fn create_code(&mut self) -> Code<Instruction<W>, Vec<Instruction<W>>, W> {
        Code::new(mem::take(self.parsed.mut_instructions()))
    }
}

#[derive(Debug, Error, PartialEq, Eq, Clone, Hash)]
pub enum LinkerError {
    #[error("Label \".{label}\" not found. Needed at {idx}.")]
    LabelNotFound { idx: usize, label: String },
    #[error("Index {idx} of label \".{label}\" cannot be converted to word.")]
    LabelIndexToWordConversionFailed { idx: usize, label: String },
    #[error("Could not find start symbol \"_start\".")]
    StartSymbolNotFound,
    #[error("The specified memory size ({mem_size}) exceeds the maximum value of Word: {max_word_value}")]
    MemorySizeToBig { mem_size: usize, max_word_value: usize },
    #[error(
        "The specified data section size ({data_size}) reaches the maximum value of Word: {max_word_value}. As a result there is no space available for the bss section of size {bss_size}."
    )]
    DataToBigForBss {
        data_size: usize,
        max_word_value: usize,
        bss_size: usize,
    },
    #[error("The specified bss size ({bss_size}) exceeds the maximum value of Word: {max_word_value}")]
    BssToBig { bss_size: usize, max_word_value: usize },
}
