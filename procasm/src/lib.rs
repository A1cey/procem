//! **`procasm`** is a toy Rust library that provides a default implementation of the [`Instruction`](../procem/instruction/trait.Instruction.html) trait of the [`procem`](../procem/index.html) library.
//!
//! # Instruction Set
//!
//! ## Syntax
//!
//! All assembly is interpreted as ASCII.
//! All instructions, registers and immediate values can be written in mixed case.
//! All operations that can be suffixed with an 'S' set the flag registers depending on the operation.
//!
//! - *Labels* (**\<LABEL>**) are used to mark specific locations in the program. They are denoted by a string of alphanumeric or underscore ('_') or dash ('-') characters followed by a colon (':') (e.g., 'label:'). Labels are case-sensitive.
//! - *Registers* (**\<REG>**) must be a valid register name (e.g., 'R0', 'r1', 'R2', 'PC', 'sp').
//! - *Literals* (**\<LIT>**) are decimal, binary, hexadecimal, octal, boolean or char constants.
//!   - Decimal values start with '0d' (optional), followed by a sequence of '0's through '9's. Decimal values can be negative.
//!   - Binary values start with '0b', followed by a sequence of '0's and '1's.
//!   - Hexadecimal values start with '0x', followed by a sequence of digits from '0' through '9' and letters from 'a' through 'f'.
//!   - Octal values start with '0o', followed by a sequence of '0's through '7's.
//!   - Character values are enclosed in single quotes, e.g., 'a', 'B', '5'.
//! - *Operands* (**\<OP>**) can be a register name or a literal.
//! - Compiler *directives* are special instructions that the assembler uses to control the assembly process. They start with a '.' followed by a string of alphanumeric or underscore ('_') or dash ('-') characters.
//!   - There can be three *Sections* in a program. Sections can be in any order and occur multiple times. A valid program must have at least one *.code* section.
//!      - *.code*: This section is mandatory and contains executable instructions.
//!      - *.data*: This section is optional and contains data declarations.
//!      - *.bss*: This section is optional and contains uninitialized data declarations.
//!
//! 'END' marks the end of the program. It is only used as a guide for the assembler and not part of the assembled program.
//!
//! ### Directives
//!
//! - *.data*: This section is optional and contains data declarations.
//! - *.bss*: This section is optional and contains uninitialized data declarations.
//! - *.code*: This section is mandatory and contains executable instructions.
//! - *.byte*: Usable only in *.data* sections. Declares a byte-sized data item (8-bit).
//! - *.hword*: Usable only in *.data* sections. Declares a hword-sized data item (16-bit).
//! - *.word*: Usable only in *.data* sections. Declares a word-sized data item (32-bit).
//! - *.dword*: Usable only in *.data* sections. Declares a dword-sized data item (64-bit).
//! - *.qword*: Usable only in *.data* sections. Declares a qword-sized data item (128-bit).
//! - *.ascii*: Usable only in *.data* sections. Declares an ASCII string.
//! - *.space*: Usable only in *.bss* sections. Declares a block of memory.
//!
//! ### Data Section
//!
//! Use *.byte*, *.hword*, *.word*, *.dword*, *.qword*, or *.ascii* followed by a *Literal* to declare data in the *.data* section. To declare an array of data multiple literals separated by commas can be used.
//!
//! Example:
//! ```asm
//! .data
//!     .byte 5
//!     .word 10, 20, 30, 40, 50
//!     .ascii "Hello, World!"
//! ```
//!
//! ### Bss Section
//!
//! Use *.space* followed by a numeric *Literal* (Decimal, Octal, Hexadecimal, Binary) to declare uninitialized data in the *.bss* section. The *Literal* specifies the number of bytes to allocate.
//!
//! # Usage
//! To assemble a [`Program`](../procem/program/struct.Program.html) from assembly code use the [`assemble`] function.
//!
//! # Example
//! ```
//! use procem::{processor::Processor, register::Register};
//! use procasm::assemble;
//!
//! const MEM_SIZE: usize = 1024;
//!
//! // Assemble a program from asm
//! let program = assemble::<MEM_SIZE>(
//!     "
//!     .code
//!     _start:
//!         mov R0, 10
//!         mov R1, 5
//!         add R0, R1
//!         sub R0, 3
//!         mul R0, 2
//!         div R0, 4
//!     "
//! ).unwrap();
//!
//! // Create a processor and run the program
//! let mut processor = Processor::builder().with_program(&program).build();
//!
//! let _ = processor.run_program();
//!
//! // Inspect register values
//! assert_eq!(processor.registers.get_reg(Register::R0), 6);
//! ```

use crate::instruction::Instruction;
use crate::linker::{Linker, LinkerError};
use crate::parser::parse;
use crate::tokenizer::{Tokenizer, TokenizerError};
use procem::program::Program;
use thiserror::Error;

pub mod instruction;
mod linker;
pub(crate) mod parser;
pub(crate) mod tokenizer;

pub type AssembledProgram<const MEM_SIZE: usize> = Program<MEM_SIZE, Instruction, Vec<Instruction>, Vec<u8>>;

/// Assembles Program from assembly code.
///
/// # Errors
/// Returns a vector of all errors that a happened during either the tokenizing or the parsing.
///
/// # Example
/// ```
/// use procem::{program::{Program, Code, Header, Bss, Data}, register::Register};
/// use procasm::{assemble, AssembledProgram, instruction::{Instruction, jump_condition::JumpCondition, operand::Operand}};
///
/// const MEM_SIZE: usize = 1024;
///
/// let program = assemble::<MEM_SIZE>(
///     "
///     .code
///     _start:
///         mov R0, 2
///         add R1, R0
///         jmp _start
///     ",
/// )
/// .unwrap();
///
/// assert_eq!(
///     program,
///     AssembledProgram::<MEM_SIZE>::new(
///         Header::new(0, MEM_SIZE as u64 - 1),
///         Data::default(),
///         Bss::default(),
///         Code::from(
///             vec![
///                  Instruction::Mov {
///                      to: Register::R0,
///                      from: Operand::Value(2)
///                  },
///                  Instruction::Add {
///                      acc: Register::R1,
///                      rhs: Operand::Register(Register::R0),
///                      set_flags: false
///                  },
///                  Instruction::Jump {
///                      to: 0,
///                      condition: JumpCondition::Unconditional
///                  }
///             ]
///         )
///     )
/// );
/// ```
pub fn assemble<const MEM_SIZE: usize>(input: impl AsRef<str>) -> Result<AssembledProgram<MEM_SIZE>, Vec<AssemblerError>> {
    let input = input.as_ref().as_bytes();

    let tokens = Tokenizer::tokenize(input).map_err(|err| err.into_iter().map(Into::into).collect::<Vec<AssemblerError>>())?;

    let parsed = parse(&tokens, input).map_err(|err| {
        err.into_iter().map(|err| AssemblerError::Parser { err: err.render(input) }).collect::<Vec<AssemblerError>>()
    })?;

    Linker::<'_, MEM_SIZE>::link(input, parsed).map_err(|err| err.into_iter().map(Into::into).collect::<Vec<AssemblerError>>())
}

#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum AssemblerError {
    #[error("Error during tokenization: {err}")]
    Tokenizer {
        #[from]
        err: TokenizerError,
    },
    #[error("Error during parsing: {err}")]
    Parser { err: String },
    #[error("Error during linking: {err}")]
    Linker {
        #[from]
        err: LinkerError,
    },
}
