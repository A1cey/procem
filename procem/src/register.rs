//! The [`Registers`] struct, [`Register`] enum and [`Flag`] enum.
use ars::fmt::slice::FmtSlice;
use core::fmt::Debug;
use core::str::FromStr;
use thiserror::Error;

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

pub const GENERAL_REGISTER_COUNT: usize = 16;

/// The `Registers` struct provides general purpose registers,
/// a program counter, a stack pointer and flags.
///
/// There are [`GENERAL_REGISTER_COUNT`] general purpose registers (R1 - Rn).
/// They can be accessed with the [`get_reg`](Registers::get_reg) and [`set_reg`](Registers::set_reg) methods by providing the corresponding [`Register`] value.
///
/// The program counter (pc) can be read with the [`pc`](Registers::pc) method and the stack pointer (sp) can be read with the [`sp`](Registers::sp) method.
/// Both of these registers can also be accessed with the [`get_reg`](Registers::get_reg) and [`set_reg`](Registers::set_reg) methods.
///
/// Note: [`sp`](Registers::sp) is a bare register. A stack is not directly provided by this emulator. The instruction set defines how the memory is used.
///
/// The register sizes correspond to the memory word size.
///
/// The flags are carry flag ([`C`](Flag::C)), signed flag ([`S`](Flag::S)), overflow flag ([`V`](Flag::V)) and zero condition flag ([`Z`](Flag::Z)).
/// They can be accessed with the [`get_flag`](Registers::get_flag) and [`set_flag`](Registers::set_flag) methods by providing the corresponding [`Flag`] value.
///
/// There are two convenience methods for incrementing and decrementing registers: [`inc`](Registers::inc) and [`dec`](Registers::dec).
#[derive(Debug, PartialEq, Eq, Clone, Hash, PartialOrd, Ord, Default)]
pub struct Registers {
    // General purpose registers, program counter (pc) and stack pointer (sp).
    registers: [usize; GENERAL_REGISTER_COUNT + 2],
    // Flags: carry flag (C), signed flag (S), overflow flag (V), zero condition flag (Z).
    flags: [bool; 4],
}

impl Registers {
    /// Create a new set of registers with all values initialized to the default value.
    #[must_use]
    pub fn new() -> Self {
        Self {
            registers: [usize::default(); GENERAL_REGISTER_COUNT + 2],
            flags: [false; 4],
        }
    }

    /// Get the value of a register.
    #[inline]
    pub fn get_reg(&self, reg: Register) -> usize {
        self.registers[usize::from(reg)]
    }

    /// Get the value of the program counter register.
    #[inline]
    pub fn pc(&self) -> usize {
        self.registers[usize::from(Register::PC)]
    }

    /// Get the value of the stack pointer register.
    #[inline]
    pub fn sp(&self) -> usize {
        self.registers[usize::from(Register::SP)]
    }

    /// Set the value of a register.
    #[inline]
    pub fn set_reg(&mut self, reg: Register, val: usize) {
        self.registers[usize::from(reg)] = val;
    }

    /// Increment the value in a register by one.
    #[inline]
    pub fn inc(&mut self, reg: Register) {
        self.registers[usize::from(reg)] += 1;
    }

    /// Decrement the value in a register by one.
    #[inline]
    pub fn dec(&mut self, reg: Register) {
        self.registers[usize::from(reg)] -= 1;
    }

    /// Get the value of a flag.
    #[inline]
    pub fn get_flag(&self, flag: Flag) -> bool {
        self.flags[usize::from(flag)]
    }

    /// Set the value of a flag.
    #[inline]
    pub fn set_flag(&mut self, flag: Flag, val: bool) {
        self.flags[usize::from(flag)] = val;
    }
}

impl core::fmt::Display for Registers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "general:\t")?;
        writeln!(f, "{}", FmtSlice(self.registers.as_slice()))?;
        writeln!(f, "pc:\t\t{}\nsp:\t\t{}", self.pc(), self.sp())?;
        writeln!(
            f,
            "flags:\t\t[C: {}, S: {}, V: {}, Z: {}]",
            self.flags[0], self.flags[1], self.flags[2], self.flags[3]
        )
    }
}

/// Register enum.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Hash, PartialOrd, Ord)]
pub enum Register {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
    R8,
    R9,
    R10,
    R11,
    R12,
    R13,
    R14,
    R15,
    PC,
    SP,
}

impl FromStr for Register {
    type Err = RegisterError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "R0" | "r0" => Ok(Self::R0),
            "R1" | "r1" => Ok(Self::R1),
            "R2" | "r2" => Ok(Self::R2),
            "R3" | "r3" => Ok(Self::R3),
            "R4" | "r4" => Ok(Self::R4),
            "R5" | "r5" => Ok(Self::R5),
            "R6" | "r6" => Ok(Self::R6),
            "R7" | "r7" => Ok(Self::R7),
            "R8" | "r8" => Ok(Self::R8),
            "R9" | "r9" => Ok(Self::R9),
            "R10" | "r10" => Ok(Self::R10),
            "R11" | "r11" => Ok(Self::R11),
            "R12" | "r12" => Ok(Self::R12),
            "R13" | "r13" => Ok(Self::R13),
            "R14" | "r14" => Ok(Self::R14),
            "R15" | "r15" => Ok(Self::R15),
            "PC" | "pc" => Ok(Self::PC),
            "SP" | "sp" => Ok(Self::SP),
            _ => Err(
                #[cfg(feature = "alloc")]
                RegisterError::ConversionFailed {
                    input: value.to_string(),
                },
                #[cfg(not(feature = "alloc"))]
                RegisterError::ConversionFailed,
            ),
        }
    }
}

impl TryFrom<&[u8]> for Register {
    type Error = RegisterError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        match value {
            b"R0" | b"r0" => Ok(Self::R0),
            b"R1" | b"r1" => Ok(Self::R1),
            b"R2" | b"r2" => Ok(Self::R2),
            b"R3" | b"r3" => Ok(Self::R3),
            b"R4" | b"r4" => Ok(Self::R4),
            b"R5" | b"r5" => Ok(Self::R5),
            b"R6" | b"r6" => Ok(Self::R6),
            b"R7" | b"r7" => Ok(Self::R7),
            b"R8" | b"r8" => Ok(Self::R8),
            b"R9" | b"r9" => Ok(Self::R9),
            b"R10" | b"r10" => Ok(Self::R10),
            b"R11" | b"r11" => Ok(Self::R11),
            b"R12" | b"r12" => Ok(Self::R12),
            b"R13" | b"r13" => Ok(Self::R13),
            b"R14" | b"r14" => Ok(Self::R14),
            b"R15" | b"r15" => Ok(Self::R15),
            b"PC" | b"pc" => Ok(Self::PC),
            b"SP" | b"sp" => Ok(Self::SP),
            _ => Err(
                #[cfg(feature = "alloc")]
                RegisterError::ConversionFailed {
                    input: String::from_utf8_lossy(value).to_string(),
                },
                #[cfg(not(feature = "alloc"))]
                RegisterError::ConversionFailed,
            ),
        }
    }
}

impl From<Register> for usize {
    fn from(reg: Register) -> Self {
        match reg {
            Register::R0 => 0,
            Register::R1 => 1,
            Register::R2 => 2,
            Register::R3 => 3,
            Register::R4 => 4,
            Register::R5 => 5,
            Register::R6 => 6,
            Register::R7 => 7,
            Register::R8 => 8,
            Register::R9 => 9,
            Register::R10 => 10,
            Register::R11 => 11,
            Register::R12 => 12,
            Register::R13 => 13,
            Register::R14 => 14,
            Register::R15 => 15,
            Register::PC => 16,
            Register::SP => 17,
        }
    }
}

/// Flag enum.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub enum Flag {
    /// Carry flag. Normally set when an addition results in a carry or a subtraction results in a borrow.
    C,
    /// Signed flag. Normally set when the last arithmetic computation resulted in a negative value.
    S,
    /// Overflow flag. Normally set when the last arithmetic computation resulted in an overflow.
    V,
    /// Zero condition flag. Normally set when the last arithmetic, logical or bitwise computation resulted in zero.
    Z,
}

impl From<Flag> for usize {
    fn from(reg: Flag) -> Self {
        match reg {
            Flag::C => 0,
            Flag::S => 1,
            Flag::V => 2,
            Flag::Z => 3,
        }
    }
}

#[derive(Debug, Error, Clone, PartialEq, Eq, Hash)]
pub enum RegisterError {
    #[cfg(feature = "alloc")]
    #[error("Failed to convert {input} into a register.")]
    ConversionFailed { input: String },
    #[cfg(not(feature = "alloc"))]
    #[error("Invalid register name. Conversion into register failed.")]
    ConversionFailed,
}
