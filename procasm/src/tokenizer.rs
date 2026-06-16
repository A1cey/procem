use std::fmt::Display;

use thiserror::Error;

use ars::range::Range;

#[doc(hidden)] // Only public for benchmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Token {
    Identifier(Range),
    ImmediateLiteral(ImmediateLiteral),
    StringLiteral(Range),
    Comma,
    Colon,
    OpenBracket,
    ClosedBracket,
    End,
    Directive(Range),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier(range) => write!(f, "Identifier: {range:?}"),
            Self::ImmediateLiteral(literal) => write!(f, "Literal: {literal}"),
            Self::StringLiteral(range) => write!(f, "String: {range:?}"),
            Self::Comma => write!(f, "Comma"),
            Self::Colon => write!(f, "Colon"),
            Self::OpenBracket => write!(f, "OpenBracket"),
            Self::ClosedBracket => write!(f, "ClosedBracket"),
            Self::End => write!(f, "End"),
            Self::Directive(range) => write!(f, "Section: {range:?}"),
        }
    }
}

#[doc(hidden)] // Only public for benchmarks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImmediateLiteral {
    Decimal(Range),
    Binary(Range),
    Hexadecimal(Range),
    Octal(Range),
    Char(u8),
}

impl Display for ImmediateLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decimal(range) => write!(f, "Decimal: {range:?}"),
            Self::Binary(range) => write!(f, "Binary: {range:?}"),
            Self::Hexadecimal(range) => write!(f, "Hexadecimal: {range:?}"),
            Self::Octal(range) => write!(f, "Octal: {range:?}"),
            Self::Char(c) => write!(f, "Char: {c}"),
        }
    }
}

#[doc(hidden)] // Only public for benchmarks.
pub struct Tokenizer<'input> {
    tokens: Vec<Token>,
    curr_idx: usize,
    token_start_idx: usize,
    input: &'input [u8],
    input_len: usize,
    errors: Option<Vec<TokenizerError>>,
}

impl Tokenizer<'_> {
    fn from(input: &[u8]) -> Tokenizer<'_> {
        Tokenizer {
            tokens: Vec::with_capacity(input.len()),
            curr_idx: 0,
            token_start_idx: 0,
            input_len: input.len(),
            input,
            errors: None,
        }
    }

    #[doc(hidden)] // Only public for benchmarks.
    pub fn tokenize(input: &[u8]) -> Result<Vec<Token>, Vec<TokenizerError>> {
        let mut tokenizer = Self::from(input);

        tokenizer.run();

        match tokenizer.errors {
            Some(errors) => Err(errors),
            None => Ok(tokenizer.tokens),
        }
    }

    fn run(&mut self) {
        while self.curr_idx < self.input_len {
            self.process_next_token();
        }
    }

    // TODO: no labels or instructions can start with f or r as this will be interpreted as boolean literal or register
    fn process_next_token(&mut self) {
        self.token_start_idx = self.curr_idx;

        match self.get_curr_byte() {
            b'.' => self.expect_directive(),
            b'\'' => self.expect_char_literal(),
            b'"' => self.expect_string_literal(),
            b',' => self.expect_comma(),
            b':' => self.expect_colon(),
            b'[' => self.expect_open_bracket(),
            b']' => self.expect_closed_bracket(),
            b if b == b'-' || b.is_ascii_digit() => self.expect_numeric_literal(),
            b if Self::is_valid_char(b) => self.expect_identifier(),
            b if b.is_ascii_whitespace() => self.curr_idx += 1,
            b => {
                self.curr_idx += 1;
                self.add_error(TokenizerError::TokenStart {
                    start: char::from(b),
                    idx: self.curr_idx,
                });
            }
        }
    }

    /// Valid chars in labels and instructions
    #[inline]
    const fn is_valid_char(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
    }

    #[inline]
    fn add_error(&mut self, err: TokenizerError) {
        self.errors.get_or_insert_default().push(err);
    }

    fn get_curr_byte(&self) -> u8 {
        *self
            .input
            .get(self.curr_idx)
            .expect("The index should not be greater or equal to the length of the input. This should never happen.")
    }

    fn set_curr_idx_to_immediate_literal_end(&mut self) {
        if !self.get_curr_byte().is_ascii_hexdigit() {
            return;
        }

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_hexdigit() {
            self.curr_idx += 1;
        }
        self.curr_idx -= 1;
    }

    fn expect_directive(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_alphabetic() {
            self.curr_idx += 1;
        }

        let start = self.token_start_idx + 1; // do not count '.'
        let end = self.curr_idx;

        self.tokens.push(Token::Directive(Range(start, end)));
    }

    fn expect_identifier(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len {
            let b = self.get_curr_byte();
            if Self::is_valid_char(b) {
                self.curr_idx += 1;
            } else {
                break;
            }
        }

        let start = self.token_start_idx;
        let end = self.curr_idx;

        let token = if self.input[start..end].eq_ignore_ascii_case(b"end") {
            Token::End
        } else {
            Token::Identifier(Range(start, end))
        };

        self.tokens.push(token);
    }

    fn expect_comma(&mut self) {
        self.tokens.push(Token::Comma);
        self.curr_idx += 1;
    }

    fn expect_colon(&mut self) {
        self.tokens.push(Token::Colon);
        self.curr_idx += 1;
    }

    fn expect_open_bracket(&mut self) {
        self.tokens.push(Token::OpenBracket);
        self.curr_idx += 1;
    }

    fn expect_closed_bracket(&mut self) {
        self.tokens.push(Token::ClosedBracket);
        self.curr_idx += 1;
    }

    fn expect_char_literal(&mut self) {
        self.curr_idx += 1; // skip start "'"

        let b = self.get_curr_byte();

        self.curr_idx += 1;

        match self.get_curr_byte() {
            b'\'' => self.tokens.push(Token::ImmediateLiteral(ImmediateLiteral::Char(b))),
            _ => self.add_error(TokenizerError::CharLiteral { idx: self.curr_idx }),
        }

        self.curr_idx += 1; // skip end "'"
    }

    fn expect_string_literal(&mut self) {
        self.curr_idx += 1; // skip start '"'

        while self.get_curr_byte() != b'"' {
            self.curr_idx += 1;
        }

        self.tokens.push(Token::StringLiteral(Range(
            self.token_start_idx + 1, // skip start '"'
            self.curr_idx,            // end '"' is excluded by exclusive range
        )));

        self.curr_idx += 1; // skip end '"'
    }

    fn expect_numeric_literal(&mut self) {
        let literal = if self.get_curr_byte() == b'0' && self.curr_idx + 1 != self.input_len {
            self.curr_idx += 1; // skip uninteressting '0'
            self.token_start_idx = self.curr_idx; // token_start can be moved, beginning '0' can be ignored
            match self.get_curr_byte() {
                b'B' | b'b' => {
                    self.curr_idx += 1; // skip 'b'/'B'
                    self.set_curr_idx_to_immediate_literal_end();
                    ImmediateLiteral::Binary(Range(
                        self.token_start_idx + 1, // skip 'b'/'B'
                        self.curr_idx + 1,        // exclusive
                    ))
                }
                b'X' | b'x' => {
                    self.curr_idx += 1; // skip 'x'/'X'
                    self.set_curr_idx_to_immediate_literal_end();
                    ImmediateLiteral::Hexadecimal(Range(
                        self.token_start_idx + 1, // skip 'x'/'X'
                        self.curr_idx + 1,        // exclusive
                    ))
                }
                b'O' | b'o' => {
                    self.curr_idx += 1; // skip 'o'/'O'
                    self.set_curr_idx_to_immediate_literal_end();
                    ImmediateLiteral::Octal(Range(
                        self.token_start_idx + 1, // skip 'o'/'O'
                        self.curr_idx + 1,        // exclusive
                    ))
                }
                b'D' | b'd' => {
                    self.curr_idx += 1; // skip 'd'/'D'
                    self.set_curr_idx_to_immediate_literal_end();
                    ImmediateLiteral::Decimal(Range(
                        self.token_start_idx + 1, // skip 'd'/'D'
                        self.curr_idx + 1,        // exclusive
                    ))
                }
                b if b.is_ascii_whitespace() || b.is_ascii_digit() => {
                    self.set_curr_idx_to_immediate_literal_end();
                    ImmediateLiteral::Decimal(Range(
                        self.token_start_idx - 1, // if only '0' then we need to include it for '042' the '0' could have been ignored
                        self.curr_idx,
                    ))
                }

                _ => todo!("Error case no matching digit after 0 (allowed: x,d,b,o)"),
            }
        } else {
            if self.get_curr_byte() == b'-' {
                self.curr_idx += 1; // skip '-'
            }

            self.set_curr_idx_to_immediate_literal_end();
            ImmediateLiteral::Decimal(Range(self.token_start_idx, self.curr_idx + 1))
        };

        let lit = Token::ImmediateLiteral(literal);

        self.tokens.push(lit);

        self.curr_idx += 1;
    }
}

#[derive(Error, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerError {
    #[error("Token at idx {idx} is not allowed to start with {start}. ")]
    TokenStart { start: char, idx: usize },
    #[error("Invalid literal at idx: {idx}.")]
    Literal { idx: usize },
    #[error("Expected char literal at idx {idx} to end with \'.")]
    CharLiteral { idx: usize },
    #[error("Invalid character {character} at idx: {idx} in label name starting at idx {token_start_idx}.")]
    InvalidLabelName {
        token_start_idx: usize,
        idx: usize,
        character: char,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions_sorted::assert_eq;
    use std::panic;

    #[test]
    fn test_run() {
        let asm = b"
        .code
        main:
            MOV R0, 5
            nop
            MOV R256, 0xBc2a
            Mul R0, r256
            JMP main
        ";
        let mut t = Tokenizer::from(asm.as_slice());
        t.run();

        let mut tokens = t.tokens.iter().into_iter();

        match tokens.next().unwrap() {
            Token::Directive(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "code"),
            t => panic!("expected Directive, got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "main"),
            t => panic!("expected Identifier (Label), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Colon => {}
            t => panic!("expected Colon, got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "MOV"),
            t => panic!("expected Identifier (Instruction), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("R0")),
            t => panic!("expected Identifier (Register), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            t => panic!("expected Comma, got {t}"),
        }
        match tokens.next().unwrap() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "5"),
                t => panic!("expected Decimal, got {t}"),
            },
            t => panic!("expected ImmediateLiteral, got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("nop")),
            t => panic!("expected Identifier (Instruction), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("MOV")),
            t => panic!("expected Identifier (Instruction), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("R256")),
            t => panic!("expected Identifier (Register), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            t => panic!("expected Comma, got {t}"),
        }
        match tokens.next().unwrap() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Hexadecimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("Bc2a")),
                t => panic!("expected Hexadecimal, got {t}"),
            },
            t => panic!("expected ImmediateLiteral, got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("Mul")),
            t => panic!("expected Identifier (Instruction), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("R0")),
            t => panic!("expected Identifier (Register), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            t => panic!("expected Comma, got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("r256")),
            t => panic!("expected Identifier (Register), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("JMP")),
            t => panic!("expected Identifier (Instruction), got {t}"),
        }
        match tokens.next().unwrap() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "main"),
            t => panic!("expected Identifier (Label), got {t}"),
        }
    }

    #[test]
    fn test_add_error() {
        let asm: [u8; 0] = [];
        let mut t = Tokenizer::from(asm.as_slice());
        let err = TokenizerError::TokenStart { start: ' ', idx: 0 };
        assert!(t.errors.is_none());
        t.add_error(err);
        assert_eq!(t.errors.unwrap(), vec![err.into()]);
    }

    #[test]
    fn test_get_curr_byte() {
        let asm = b".main mov";
        let t = Tokenizer::from(asm.as_slice());
        assert_eq!(t.get_curr_byte(), b'.');
    }

    #[test]
    #[should_panic]
    fn test_get_curr_byte_out_of_bounds() {
        let asm = b"main:";
        let mut t = Tokenizer::from(asm.as_slice());
        assert_eq!(t.get_curr_byte(), b'm');
        t.curr_idx += 5;
        let _ = t.get_curr_byte(); // panic
    }

    #[test]
    fn test_expect_label() {
        let asm = b"main:";
        let mut t = Tokenizer::from(asm.as_slice());
        t.run();
        match t.tokens[0].clone() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "main"),
            t => panic!("Expected Identifier got {t:?}"),
        }
        match t.tokens[1].clone() {
            Token::Colon => {}
            t => panic!("Expected Colon got {t:?}"),
        }
        let asm = b"MAIN:";
        t = Tokenizer::from(asm.as_slice());
        t.run();
        match t.tokens[0].clone() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "MAIN"),
            t => panic!("Expected Identifier got {t:?}"),
        }
        match t.tokens[1].clone() {
            Token::Colon => {}
            t => panic!("Expected Colon got {t:?}"),
        }
    }

    #[test]
    fn test_expect_directive() {
        let asm = b".code";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_directive();
        match t.tokens[0].clone() {
            Token::Directive(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "code"),
            t => panic!("Expected Directive got {t:?}"),
        }
        let asm = b".DATA";
        t = Tokenizer::from(asm.as_slice());
        t.expect_directive();
        match t.tokens[0].clone() {
            Token::Directive(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "DATA"),
            t => panic!("Expected Directive got {t:?}"),
        }
    }

    #[test]
    fn test_expect_end() {
        let asm = b"end";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        assert_eq!(t.tokens[0].clone(), Token::End);
        let asm = b"END";
        t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        assert_eq!(t.tokens[0].clone(), Token::End);
    }

    #[test]
    fn test_expect_instruction() {
        let asm = b"mov";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        match t.tokens[0].clone() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "mov"),
            t => panic!("Expected Identifier got {t:?}"),
        }
        let asm = b"JMP";
        t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        match t.tokens[0].clone() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "JMP"),
            t => panic!("Expected Identifier got {t:?}"),
        }
    }

    #[test]
    fn test_expect_register() {
        let asm = b"R0";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        match t.tokens[0].clone() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "R0"),
            t => panic!("Expected Identifier got {t:?}"),
        }
        let asm = b"R4242";
        t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        match t.tokens[0].clone() {
            Token::Identifier(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "R4242"),
            t => panic!("Expected Identifier got {t:?}"),
        }
    }

    #[test]
    fn test_expect_comma() {
        let asm = b",";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_comma();
        assert_eq!(t.tokens[0], Token::Comma);
    }

    #[test]
    fn test_expect_literal() {
        let asm = b"42";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("42")),
                l => panic!("Expected Decimal got {l:?}"),
            },
            t => panic!("Expected ImmediateLiteral got {t}"),
        }
        let asm = b"0x4F";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Hexadecimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("4F")),
                l => panic!("Expected Hexadecimal got {l:?}"),
            },
            t => panic!("Expected ImmediateLiteral got {t}"),
        }
        let asm = b"0b010110";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Binary(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("010110")),
                l => panic!("Expected Binary got {l:?}"),
            },
            t => panic!("Expected ImmediateLiteral got {t}"),
        }
        let asm = b"0o743";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Octal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("743")),
                l => panic!("Expected Octal got {l:?}"),
            },
            t => panic!("Expected ImmediateLiteral got {t}"),
        }
        let asm = b"\"Hello, there\"";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::StringLiteral(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("Hello, there")),
            t => panic!("Expected StringLiteral got {t:?}"),
        }
        let asm = b"\'7\'";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        assert_eq!(t.tokens[0], Token::ImmediateLiteral(ImmediateLiteral::Char(b'7')));
    }

    #[test]
    fn test_expect_char_literal() {
        let asm = b"\'B\'";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        assert_eq!(t.tokens[0], Token::ImmediateLiteral(ImmediateLiteral::Char(b'B')));
    }

    #[test]
    fn test_expect_string_literal() {
        let asm = b"\"Jajajajaja2498291849102+#amfl929r2jlsamfa3\"";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::StringLiteral(r) => assert_eq!(
                String::from_utf8_lossy(&asm[r]),
                ("Jajajajaja2498291849102+#amfl929r2jlsamfa3")
            ),
            t => panic!("Expected StringLiteral got: {t:?}"),
        }
    }

    #[test]
    fn test_expect_numeric_literal() {
        let asm = b"42";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("42")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let asm = b"0d42";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("42")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let asm = b"-42";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("-42")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let asm = b"0x4F";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Hexadecimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("4F")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let asm = b"0b010110";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Binary(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("010110")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let asm = b"0o743";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token::ImmediateLiteral(l) => match l {
                ImmediateLiteral::Octal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("743")),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_zero_decimal_literal() {
        let asm = b"0";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        assert_eq!(
            t.tokens[0],
            Token::ImmediateLiteral(ImmediateLiteral::Decimal(Range(0, 1)))
        );
    }

    #[test]
    fn expect_bracket() {
        let asm = b"[]]";
        let mut t = Tokenizer::from(asm.as_slice());

        t.process_next_token();
        t.process_next_token();
        t.process_next_token();
        assert_eq!(t.tokens[0], Token::OpenBracket);
        assert_eq!(t.tokens[1], Token::ClosedBracket);
        assert_eq!(t.tokens[2], Token::ClosedBracket);
    }

    #[test]
    fn expect_colon() {
        let asm = b":";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        assert_eq!(t.tokens[0], Token::Colon);
    }
}
