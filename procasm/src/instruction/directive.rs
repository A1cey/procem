use std::fmt::Display;

use crate::parser::ParserError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum Directive {
    Code,
    Data,
    Bss,
    Byte,
    Hword,
    Word,
    Dword,
    Qword,
    Ascii,
    Space,
}

impl TryFrom<&[u8]> for Directive {
    type Error = ParserError;

    fn try_from(value: &[u8]) -> Result<Self, Self::Error> {
        let directive = match value {
            s if s.eq_ignore_ascii_case(b"code") => Self::Code,
            s if s.eq_ignore_ascii_case(b"data") => Self::Data,
            s if s.eq_ignore_ascii_case(b"bss") => Self::Bss,
            s if s.eq_ignore_ascii_case(b"byte") => Self::Byte,
            s if s.eq_ignore_ascii_case(b"hword") => Self::Hword,
            s if s.eq_ignore_ascii_case(b"word") => Self::Word,
            s if s.eq_ignore_ascii_case(b"dword") => Self::Dword,
            s if s.eq_ignore_ascii_case(b"qword") => Self::Qword,
            s if s.eq_ignore_ascii_case(b"ascii") => Self::Ascii,
            s if s.eq_ignore_ascii_case(b"space") => Self::Space,
            s => Err(ParserError::InvalidDirective {
                got: String::from_utf8_lossy(s).to_string(),
                allowed: ".code, .data, .bss, .byte, .hword, .word, .dword, .qword, .ascii, or .space".to_string(),
            })?,
        };

        Ok(directive)
    }
}

impl Display for Directive {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Code => write!(f, ".code"),
            Self::Data => write!(f, ".data"),
            Self::Bss => write!(f, ".bss"),
            Self::Byte => write!(f, ".byte"),
            Self::Hword => write!(f, ".hword"),
            Self::Word => write!(f, ".word"),
            Self::Dword => write!(f, ".dword"),
            Self::Qword => write!(f, ".qword"),
            Self::Ascii => write!(f, ".ascii"),
            Self::Space => write!(f, ".space"),
        }
    }
}
