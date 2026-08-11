use std::mem;

use procem::program::{Bss, Code, Data, Header, Program};
use thiserror::Error;

use crate::{
    AssembledProgram,
    instruction::{Instruction as Inst, memory_location::MemoryLocation as MemLoc, unlinked::UnlinkedInstruction},
    parser::Parsed,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Linker<'input, const MEM_SIZE: usize> {
    errors: Vec<LinkerError>,
    input: &'input [u8],
    parsed: Parsed<'input>,
}

impl<'input, const MEM_SIZE: usize> Linker<'input, MEM_SIZE> {
    pub fn link(input: &'input [u8], parsed: Parsed<'input>) -> Result<AssembledProgram<MEM_SIZE>, Vec<LinkerError>> {
        let mut linker = Self {
            errors: Vec::new(),
            input,
            parsed,
        };

        let program = linker.run();

        if !linker.errors.is_empty() {
            return Err(linker.errors);
        }

        Ok(program.expect("There were no errors, so this is not None."))
    }

    fn run(&mut self) -> Option<AssembledProgram<MEM_SIZE>> {
        let unlinked_instructions = mem::take(self.parsed.mut_unlinked_instructions());

        for unlinked_instruction in unlinked_instructions {
            self.link_instruction(unlinked_instruction);
        }

        let header = self.create_header();
        let data = self.create_data();
        let bss = self.create_bss();
        let code = self.create_code();

        if let Some(header) = header
            && let Some(bss) = bss
        {
            Some(Program::new(header, data, bss, code))
        } else {
            None
        }
    }

    fn link_instruction(&mut self, unlinked_instruction: UnlinkedInstruction) {
        let label = &self.input[unlinked_instruction.label()];

        let addr = match self.parsed.labels().get(label) {
            Some(addr) => *addr,
            None => {
                return self.errors.push(LinkerError::LabelNotFound {
                    idx: unlinked_instruction.instr_idx(),
                    label: String::from_utf8_lossy(label).to_string(),
                });
            }
        };

        let instruction = self
            .parsed
            .mut_instructions()
            .get_mut(unlinked_instruction.instr_idx())
            .expect("The instruction index is always in range of the instructions.");

        // skips forrmatting the match
        #[rustfmt::skip]
        let linked_instruction = match instruction {
            Inst::Jump { to: _, condition } => Inst::Jump { to: addr, condition: *condition },
            Inst::Adr { reg, addr: _ } => Inst::Adr { reg: *reg, addr },
            Inst::Str { from, to: MemLoc::Labeled(_) } => Inst::Str { from: *from, to: MemLoc::Labeled(addr) },
            Inst::Strb { from, to: MemLoc::Labeled(_) } => Inst::Strb { from: *from, to: MemLoc::Labeled(addr) },
            Inst::Strh { from, to: MemLoc::Labeled(_) } => Inst::Strh { from: *from, to: MemLoc::Labeled(addr) },
            Inst::Strw { from, to: MemLoc::Labeled(_) } => Inst::Strw { from: *from, to: MemLoc::Labeled(addr) },
            Inst::Strd { from, to: MemLoc::Labeled(_) } => Inst::Strd { from: *from, to: MemLoc::Labeled(addr) },
            Inst::Strq { from, to: MemLoc::Labeled(_) } => Inst::Strq { from: *from, to: MemLoc::Labeled(addr) },
            Inst::Ldr { to, from: MemLoc::Labeled(_) } => Inst::Ldr { to: *to, from: MemLoc::Labeled(addr) },
            Inst::Ldrb { to, from: MemLoc::Labeled(_) } => Inst::Ldrb { to: *to, from: MemLoc::Labeled(addr) },
            Inst::Ldrh { to, from: MemLoc::Labeled(_) } => Inst::Ldrh { to: *to, from: MemLoc::Labeled(addr) },
            Inst::Ldrw { to, from: MemLoc::Labeled(_) } => Inst::Ldrw { to: *to, from: MemLoc::Labeled(addr) },
            Inst::Ldrd { to, from: MemLoc::Labeled(_) } => Inst::Ldrd { to: *to, from: MemLoc::Labeled(addr) },
            Inst::Ldrq { to, from: MemLoc::Labeled(_) } => Inst::Ldrq { to: *to, from: MemLoc::Labeled(addr) },
            instruction => unreachable!("This instruction cannot be linked: {instruction:?}."),
        };

        *instruction = linked_instruction;
    }

    #[must_use]
    #[inline]
    fn create_header(&mut self) -> Option<Header> {
        let init_pc = self.get_init_pc();
        let init_sp = self.get_init_sp();

        if let Some(init_pc) = init_pc
            && let Some(init_sp) = init_sp
        {
            let header = Header::new(init_pc, init_sp);
            Some(header)
        } else {
            None
        }
    }

    #[inline]
    fn get_init_pc(&mut self) -> Option<u64> {
        const START_LABEL: &[u8] = b"_start";

        let pc = self.parsed.labels().get(START_LABEL);

        if pc.is_none() {
            self.errors.push(LinkerError::StartSymbolNotFound);
        }

        pc.copied()
    }

    #[inline]
    fn get_init_sp(&mut self) -> Option<u64> {
        // Stack starts at highest value in memory

        if MEM_SIZE as u128 - 1 > u128::from(u64::MAX) {
            self.errors.push(LinkerError::MemoryTooLarge { requested: MEM_SIZE });
            None
        } else {
            Some(MEM_SIZE as u64 - 1)
        }
    }

    #[must_use]
    #[inline]
    fn create_data(&mut self) -> Data<Vec<u8>> {
        Data::new(0, mem::take(self.parsed.mut_data()))
    }

    #[must_use]
    fn create_bss(&mut self) -> Option<Bss> {
        if self.parsed.bss() == 0 {
            return Some(Bss::new(0, 0));
        }

        let base_addr = self.parsed.data().len();

        if base_addr as u128 > u128::from(u64::MAX) {
            self.errors
                .push(LinkerError::DataSectionTooLarge { requested: base_addr });
            return None;
        }
        let base_addr = base_addr as u64;

        let bss_size = self.parsed.bss();

        Some(Bss::new(base_addr, bss_size))
    }

    #[must_use]
    #[inline]
    fn create_code(&mut self) -> Code<Inst, Vec<Inst>> {
        Code::new(mem::take(self.parsed.mut_instructions()))
    }
}

#[derive(Debug, Error, PartialEq, Eq, Clone, Hash)]
pub enum LinkerError {
    #[error("Label \".{label}\" not found. Needed at {idx}.")]
    LabelNotFound { idx: usize, label: String },
    #[error("Could not find start symbol \"_start\".")]
    StartSymbolNotFound,
    #[error(
        "Specified memory size is too large. Only 64bit can be addressed. Max memory size: {}, Requested: {requested}",
        u64::MAX
    )]
    MemoryTooLarge { requested: usize },
    #[error(
        "Specified data section is too large. Only 64bit can be addressed. Max memory size: {}, requested: {requested}",
        u64::MAX
    )]
    DataSectionTooLarge { requested: usize },
}
