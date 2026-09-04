use std::fmt::Display;

use thiserror::Error;

use ars::range::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range,
}

impl Token {
    // skips forrmatting the match
    #[inline]
    pub fn resolve(self, input: &[u8]) -> String {
        use TokenKind as TK;

        match self {
            Self { kind: TK::Identifier, span } => format!("Identifier: '{}'", String::from_utf8_lossy(&input[span])),
            Self { kind: TK::ImmediateLiteral(literal), span } => format!("ImmediateLiteral: {}", literal.resolve(input, span)),
            Self { kind: TK::StringLiteral, span } => format!("StringLiteral: '{}'", String::from_utf8_lossy(&input[span])),
            Self { kind: TK::Comma, .. } => "Comma".to_string(),
            Self { kind: TK::Colon, .. } => "Colon".to_string(),
            Self { kind: TK::OpenBracket, .. } => "OpenBracket".to_string(),
            Self { kind: TK::ClosedBracket, .. } => "ClosedBracket".to_string(),
            Self { kind: TK::End, .. } => "End".to_string(),
            Self { kind: TK::Directive, span } => format!("Directive: '{}'", String::from_utf8_lossy(&input[span])),
            Self { kind: TK::Newline, .. } => "Newline".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TokenKind {
    Identifier,
    ImmediateLiteral(ImmediateLiteralKind),
    StringLiteral,
    Comma,
    Colon,
    OpenBracket,
    ClosedBracket,
    End,
    Directive,
    Newline,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Identifier => writeln!(f, "Identifier"),
            Self::ImmediateLiteral(lit) => writeln!(f, "ImmediateLiteral: {lit}"),
            Self::StringLiteral => writeln!(f, "StringLiteral"),
            Self::Comma => writeln!(f, "Comma"),
            Self::Colon => writeln!(f, "Colon"),
            Self::OpenBracket => writeln!(f, "OpenBracket"),
            Self::ClosedBracket => writeln!(f, "ClosedBracket"),
            Self::End => writeln!(f, "End"),
            Self::Directive => writeln!(f, "Directive"),
            Self::Newline => writeln!(f, "Newline"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImmediateLiteralKind {
    Decimal,
    Binary,
    Hexadecimal,
    Octal,
    Char,
}

impl Display for ImmediateLiteralKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Decimal => write!(f, "Decimal"),
            Self::Binary => write!(f, "Binary"),
            Self::Hexadecimal => write!(f, "Hexadecimal"),
            Self::Octal => write!(f, "Octal"),
            Self::Char => write!(f, "Char"),
        }
    }
}

impl ImmediateLiteralKind {
    #[inline]
    pub fn resolve(self, input: &[u8], span: Range) -> String {
        match self {
            Self::Decimal => format!("Decimal: '{}'", String::from_utf8_lossy(&input[span])),
            Self::Binary => format!("Binary: '{}'", String::from_utf8_lossy(&input[span])),
            Self::Hexadecimal => format!("Hexadecimal: '{}'", String::from_utf8_lossy(&input[span])),
            Self::Octal => format!("Octal: '{}'", String::from_utf8_lossy(&input[span])),
            Self::Char => format!("Char: '{}'", String::from_utf8_lossy(&input[span])),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tokenizer<'input> {
    tokens: Vec<Token>,
    curr_idx: usize,
    token_start_idx: usize,
    input: &'input [u8],
    input_len: usize,
    errors: Vec<TokenizerError>,
}

impl Tokenizer<'_> {
    #[inline]
    fn from(input: &[u8]) -> Tokenizer<'_> {
        Tokenizer {
            tokens: Vec::with_capacity(input.len()),
            curr_idx: 0,
            token_start_idx: 0,
            input_len: input.len(),
            input,
            errors: Vec::new(),
        }
    }

    pub fn tokenize(input: &[u8]) -> Result<Vec<Token>, Vec<TokenizerError>> {
        let mut tokenizer = Self::from(input);

        tokenizer.run();

        tokenizer.finalize()
    }

    fn finalize(mut self) -> Result<Vec<Token>, Vec<TokenizerError>> {
        if !self.errors.is_empty() {
            Err(self.errors)?;
        }

        // last token must be `End`
        match self.tokens.last() {
            Some(Token{kind:TokenKind::End, ..}) => {}
            Some(Token{kind:TokenKind::Newline, ..})
            // If there is no input just a simple `End`
            | None => self.tokens.push(Token { kind: TokenKind::End, span: Range(self.curr_idx + 1, self.curr_idx+1) }),
            // Add the `End` on a new line
            Some(_) => self.tokens.extend_from_slice(&[
                Token { kind: TokenKind::Newline, span: Range(self.curr_idx, self.curr_idx) },
                Token { kind: TokenKind::End, span: Range(self.curr_idx + 1 , self.curr_idx + 1) }
            ]),
        }

        Ok(self.tokens)
    }

    fn run(&mut self) {
        while self.curr_idx < self.input_len {
            self.process_next_token();
        }
    }

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
            b'\n' => self.expect_newline(),
            b if b == b'-' || b.is_ascii_digit() => self.expect_numeric_literal(),
            b if Self::is_valid_char(b) => self.expect_identifier(),
            b if b.is_ascii_whitespace() => self.curr_idx += 1,
            b => {
                self.curr_idx += 1;
                self.add_error(TokenizerError::TokenStart { start: char::from(b), idx: self.curr_idx });
            }
        }
    }

    #[inline]
    fn expect_newline(&mut self) {
        self.tokens.push(Token { kind: TokenKind::Newline, span: Range(self.curr_idx, self.curr_idx + 1) });
        self.curr_idx += 1;
    }

    /// Valid chars in labels and instructions
    #[inline]
    const fn is_valid_char(b: u8) -> bool {
        b.is_ascii_alphanumeric() || b == b'-' || b == b'_'
    }

    #[inline]
    fn add_error(&mut self, err: TokenizerError) {
        self.errors.push(err);
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

        self.tokens.push(Token { kind: TokenKind::Directive, span: Range(start, end) });
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

        let kind = if self.input[start..end].eq_ignore_ascii_case(b"end") { TokenKind::End } else { TokenKind::Identifier };

        self.tokens.push(Token { kind, span: Range(start, end) });
    }

    fn expect_comma(&mut self) {
        self.tokens.push(Token { kind: TokenKind::Comma, span: Range(self.curr_idx, self.curr_idx + 1) });
        self.curr_idx += 1;
    }

    fn expect_colon(&mut self) {
        self.tokens.push(Token { kind: TokenKind::Colon, span: Range(self.curr_idx, self.curr_idx + 1) });
        self.curr_idx += 1;
    }

    fn expect_open_bracket(&mut self) {
        self.tokens.push(Token { kind: TokenKind::OpenBracket, span: Range(self.curr_idx, self.curr_idx + 1) });
        self.curr_idx += 1;
    }

    fn expect_closed_bracket(&mut self) {
        self.tokens.push(Token { kind: TokenKind::ClosedBracket, span: Range(self.curr_idx, self.curr_idx + 1) });
        self.curr_idx += 1;
    }

    fn expect_char_literal(&mut self) {
        self.curr_idx += 1; // skip start "'"
        self.curr_idx += 1; // skip char

        match self.get_curr_byte() {
            b'\'' => self.tokens.push(Token {
                kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Char),
                span: Range(
                    self.curr_idx - 1, // get back to char
                    self.curr_idx,
                ),
            }),
            _ => self.add_error(TokenizerError::CharLiteral { idx: self.curr_idx }),
        }

        self.curr_idx += 1; // skip end "'"
    }

    fn expect_string_literal(&mut self) {
        let start_idx = self.curr_idx;
        self.curr_idx += 1; // skip start '"'

        let mut found = false;
        while self.curr_idx < self.input_len {
            if self.get_curr_byte() == b'"' {
                found = true;
                break;
            }
            self.curr_idx += 1;
        }

        if !found {
            return self.add_error(TokenizerError::UnterminatedString { start_idx });
        }

        self.tokens.push(Token {
            kind: TokenKind::StringLiteral,
            span: Range(
                self.token_start_idx + 1, // skip start '"'
                self.curr_idx,            // end '"' is excluded by exclusive range
            ),
        });

        self.curr_idx += 1; // skip end '"'
    }

    fn expect_numeric_literal(&mut self) {
        let (literal, span) = if self.get_curr_byte() == b'0' && self.curr_idx + 1 != self.input_len {
            self.curr_idx += 1; // skip uninteresting '0'
            self.token_start_idx = self.curr_idx; // token_start can be moved, beginning '0' can be ignored
            match self.get_curr_byte() {
                b'B' | b'b' => {
                    self.curr_idx += 1; // skip 'b'/'B'
                    self.set_curr_idx_to_immediate_literal_end();
                    (
                        ImmediateLiteralKind::Binary,
                        Range(
                            self.token_start_idx + 1, // skip 'b'/'B'
                            self.curr_idx + 1,        // exclusive
                        ),
                    )
                }
                b'X' | b'x' => {
                    self.curr_idx += 1; // skip 'x'/'X'
                    self.set_curr_idx_to_immediate_literal_end();
                    (
                        ImmediateLiteralKind::Hexadecimal,
                        Range(
                            self.token_start_idx + 1, // skip 'x'/'X'
                            self.curr_idx + 1,        // exclusive
                        ),
                    )
                }
                b'O' | b'o' => {
                    self.curr_idx += 1; // skip 'o'/'O'
                    self.set_curr_idx_to_immediate_literal_end();
                    (
                        ImmediateLiteralKind::Octal,
                        Range(
                            self.token_start_idx + 1, // skip 'o'/'O'
                            self.curr_idx + 1,        // exclusive
                        ),
                    )
                }
                b'D' | b'd' => {
                    self.curr_idx += 1; // skip 'd'/'D'
                    self.set_curr_idx_to_immediate_literal_end();
                    (
                        ImmediateLiteralKind::Decimal,
                        Range(
                            self.token_start_idx + 1, // skip 'd'/'D'
                            self.curr_idx + 1,        // exclusive
                        ),
                    )
                }
                b if b.is_ascii_whitespace() => {
                    self.curr_idx -= 1;
                    // space immediately behind starting 0
                    (
                        ImmediateLiteralKind::Decimal,
                        Range(
                            self.curr_idx,
                            self.curr_idx + 1, // exclusive
                        ),
                    )
                }
                b if b.is_ascii_digit() => {
                    self.set_curr_idx_to_immediate_literal_end();
                    (
                        ImmediateLiteralKind::Decimal,
                        Range(
                            self.token_start_idx - 1, // if only '0' then we need to include it for '042' the '0' could have been ignored
                            self.curr_idx + 1,        // exclusive
                        ),
                    )
                }

                b',' => {
                    // only '0' with following ','
                    self.curr_idx -= 1;
                    (ImmediateLiteralKind::Decimal, Range(self.curr_idx, self.curr_idx + 1))
                }
                _ => return self.add_error(TokenizerError::InvalidNumber { idx: self.curr_idx - 1 }),
            }
        } else {
            if self.get_curr_byte() == b'-' {
                self.curr_idx += 1; // skip starting '-'
            }

            self.set_curr_idx_to_immediate_literal_end();
            (ImmediateLiteralKind::Decimal, Range(self.token_start_idx, self.curr_idx + 1))
        };

        let lit = Token { kind: TokenKind::ImmediateLiteral(literal), span };

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
    InvalidLabelName { token_start_idx: usize, idx: usize, character: char },
    #[error("Invalid number at idx {idx}")]
    InvalidNumber { idx: usize },
    #[error("Unterminated string at idx: {start_idx}")]
    UnterminatedString { start_idx: usize },
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions_sorted::assert_eq;
    use std::{panic, slice::Iter};

    #[test]
    fn some_code() {
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

        fn expect_newline(asm: &[u8], tokens: &mut Iter<Token>) {
            match tokens.next().unwrap() {
                Token { kind: TokenKind::Newline, .. } => {}
                t => panic!("expected Newline, got {}", t.resolve(asm)),
            }
        }

        expect_newline(asm, &mut tokens);
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Directive, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "code"),
            t => panic!("expected Directive, got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "main"),
            t => panic!("expected Identifier (Label), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Colon, .. } => {}
            t => panic!("expected Colon, got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "MOV"),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("R0")),
            t => panic!("expected Identifier (Register), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Comma, .. } => {}
            t => panic!("expected Comma, got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), "5")
            }
            t => panic!("expected ImmediateLiteral(ImmediateLiteralKind::Decimal), got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("nop")),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("MOV")),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("R256")),
            t => panic!("expected Identifier (Register), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Comma, .. } => {}
            t => panic!("expected Comma, got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Hexadecimal), span: r } => {
                assert_eq!(String::from_utf8_lossy(&asm[r]), ("Bc2a"))
            }
            t => panic!("expected ImmediateLiteral(ImmediateLiteralKind::Hexadecimal), got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("Mul")),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("R0")),
            t => panic!("expected Identifier (Register), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Comma, .. } => {}
            t => panic!("expected Comma, got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("r256")),
            t => panic!("expected Identifier (Register), got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("JMP")),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "main"),
            t => panic!("expected Identifier (Label), got {}", t.resolve(asm)),
        }
    }

    #[test]
    fn some_code_2() {
        let asm = b"
        .code
        _start:
            mov R0, 0
            mov R1, 5
        loop:
            add R0, 1
            subs R1, 1
            jnz loop
        ";

        let tokens = Tokenizer::tokenize(asm.as_slice()).unwrap();

        println!("{tokens:?}");

        let mut tokens = tokens.iter().into_iter();

        fn expect_newline(asm: &[u8], tokens: &mut Iter<Token>) {
            match tokens.next().unwrap() {
                Token { kind: TokenKind::Newline, .. } => {}
                t => panic!("expected Newline, got {}", t.resolve(asm)),
            }
        }

        expect_newline(asm, &mut tokens);

        match tokens.next().unwrap() {
            Token { kind: TokenKind::Directive, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "code"),
            t => panic!("expected Directive, got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);

        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "_start"),
            t => panic!("expected Identifier (Label), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Colon, .. } => {}
            t => panic!("expected Colon, got {}", t.resolve(asm)),
        }
        println!("{}", tokens.clone().next().unwrap().resolve(asm));
        expect_newline(asm, &mut tokens);
        println!("{}", tokens.clone().next().unwrap().resolve(asm));
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "mov"),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        println!("{}", tokens.clone().next().unwrap().resolve(asm));
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("R0")),
            t => panic!("expected Identifier (Register), got {}", t.resolve(asm)),
        }
        println!("{}", tokens.clone().next().unwrap().resolve(asm));
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Comma, .. } => {}
            t => panic!("expected Comma, got {}", t.resolve(asm)),
        }
        println!("3: {}", tokens.clone().next().unwrap().resolve(asm));
        match tokens.next().unwrap() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), "0")
            }
            t => panic!("expected ImmediateLiteral(ImmediateLiteralKind::Decimal), got {}", t.resolve(asm)),
        }
        println!("2: {}", tokens.clone().next().unwrap().resolve(asm));
        expect_newline(asm, &mut tokens);
        println!("1: {}", tokens.clone().next().unwrap().resolve(asm));
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "mov"),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("R1")),
            t => panic!("expected Identifier (Register), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Comma, .. } => {}
            t => panic!("expected Comma, got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), "5")
            }
            t => panic!("expected ImmediateLiteral(ImmediateLiteralKind::Decimal), got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);

        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "loop"),
            t => panic!("expected Identifier (Label), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Colon, .. } => {}
            t => panic!("expected Colon, got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);

        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "add"),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("R0")),
            t => panic!("expected Identifier (Register), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Comma, .. } => {}
            t => panic!("expected Comma, got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), "1")
            }
            t => panic!("expected ImmediateLiteral(ImmediateLiteralKind::Decimal), got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);

        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "subs"),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("R1")),
            t => panic!("expected Identifier (Register), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Comma, .. } => {}
            t => panic!("expected Comma, got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), "1")
            }
            t => panic!("expected ImmediateLiteral(ImmediateLiteralKind::Decimal), got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);

        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("jnz")),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        match tokens.next().unwrap() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("loop")),
            t => panic!("expected Identifier (Instruction), got {}", t.resolve(asm)),
        }
        expect_newline(asm, &mut tokens);

        match tokens.next().unwrap() {
            Token { kind: TokenKind::End, .. } => {}
            t => panic!("expected End, got {}", t.resolve(asm)),
        }
    }

    #[test]
    fn add_error() {
        let asm: [u8; 0] = [];
        let mut t = Tokenizer::from(asm.as_slice());
        let err = TokenizerError::TokenStart { start: ' ', idx: 0 };
        assert!(t.errors.is_empty());
        t.add_error(err);
        assert_eq!(t.errors, vec![err.into()]);
    }

    #[test]
    fn get_curr_byte() {
        let asm = b".main mov";
        let t = Tokenizer::from(asm.as_slice());
        assert_eq!(t.get_curr_byte(), b'.');
    }

    #[test]
    #[should_panic]
    fn get_curr_byte_out_of_bounds() {
        let asm = b"main:";
        let mut t = Tokenizer::from(asm.as_slice());
        assert_eq!(t.get_curr_byte(), b'm');
        t.curr_idx += 5;
        let _ = t.get_curr_byte(); // panic
    }

    #[test]
    fn expect_label() {
        let asm = b"main:";
        let mut t = Tokenizer::from(asm.as_slice());
        t.run();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "main"),
            t => panic!("Expected Identifier got {t:?}"),
        }
        match t.tokens[1].clone() {
            Token { kind: TokenKind::Colon, span: Range(4, 5) } => {}
            t => panic!("Expected Colon got {t:?}"),
        }
        let asm = b"MAIN:";
        t = Tokenizer::from(asm.as_slice());
        t.run();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "MAIN"),
            t => panic!("Expected Identifier got {t:?}"),
        }
        match t.tokens[1].clone() {
            Token { kind: TokenKind::Colon, span: Range(4, 5) } => {}
            t => panic!("Expected Colon got {t:?}"),
        }
    }

    #[test]
    fn expect_directive() {
        let asm = b".code";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_directive();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::Directive, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "code"),
            t => panic!("Expected Directive got {t:?}"),
        }
        let asm = b".DATA";
        t = Tokenizer::from(asm.as_slice());
        t.expect_directive();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::Directive, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "DATA"),
            t => panic!("Expected Directive got {t:?}"),
        }
    }

    #[test]
    fn expect_end() {
        let asm = b"end";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        assert_eq!(t.tokens[0].clone(), Token { kind: TokenKind::End, span: Range(0, 3) });
        let asm = b"END";
        t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        assert_eq!(t.tokens[0].clone(), Token { kind: TokenKind::End, span: Range(0, 3) });
    }

    #[test]
    fn expect_instruction() {
        let asm = b"mov";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "mov"),
            t => panic!("Expected Identifier got {t:?}"),
        }
        let asm = b"JMP";
        t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "JMP"),
            t => panic!("Expected Identifier got {t:?}"),
        }
    }

    #[test]
    fn expect_register() {
        let asm = b"R0";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "R0"),
            t => panic!("Expected Identifier got {t:?}"),
        }
        let asm = b"R4242";
        t = Tokenizer::from(asm.as_slice());
        t.expect_identifier();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::Identifier, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), "R4242"),
            t => panic!("Expected Identifier got {t:?}"),
        }
    }

    #[test]
    fn expect_comma() {
        let asm = b",";
        let mut t = Tokenizer::from(asm.as_slice());
        t.expect_comma();
        assert_eq!(t.tokens[0], Token { kind: TokenKind::Comma, span: Range(0, 1) });
    }

    #[test]
    fn expect_literal() {
        let asm = b"42";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("42"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Decimal) got {}", t.resolve(asm)),
        }
        let asm = b"0x4F";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Hexadecimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("4F"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Hexadecimal) got {}", t.resolve(asm)),
        }
        let asm = b"0b010110";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Binary), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("010110"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Binary) got {}", t.resolve(asm)),
        }
        let asm = b"0o743";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Octal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("743"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Octal) got {}", t.resolve(asm)),
        }
        let asm = b"\"Hello, there\"";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::StringLiteral, span } => assert_eq!(String::from_utf8_lossy(&asm[span]), ("Hello, there")),
            t => panic!("Expected StringLiteral got {t:?}"),
        }
        let asm = b"\'7\'";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0] {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Char), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), "7")
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Char) got {t:?}"),
        }
    }

    #[test]
    fn expect_char_literal() {
        let asm = b"\'B\'";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0] {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Char), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), "B")
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteral::Char) got {t:?}"),
        }
    }

    #[test]
    fn expect_string_literal() {
        let asm = b"\"Jajajajaja2498291849102+#amfl929r2jlsamfa3\"";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::StringLiteral, span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("Jajajajaja2498291849102+#amfl929r2jlsamfa3"))
            }
            t => panic!("Expected StringLiteral got: {t:?}"),
        }
    }

    #[test]
    fn expect_unterminated_string() {
        let asm = b"\"Test";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        assert!(matches!(t.errors[0], TokenizerError::UnterminatedString { start_idx: 0 }));
        assert_eq!(t.curr_idx, asm.len());
    }

    #[test]
    fn expect_numeric_literal() {
        let asm = b"42";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("42"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Decimal) got {t:?}"),
        }
        let asm = b"0d42";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("42"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Decimal) got {t:?}"),
        }
        let asm = b"-42";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("-42"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Decimal) got {t:?}"),
        }
        let asm = b"0x4F";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Hexadecimal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("4F"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Hexadecimal) got {t:?}"),
        }
        let asm = b"0b010110";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Binary), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("010110"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Binary) got {t:?}"),
        }
        let asm = b"0o743";
        t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        match t.tokens[0].clone() {
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Octal), span } => {
                assert_eq!(String::from_utf8_lossy(&asm[span]), ("743"))
            }
            t => panic!("Expected ImmediateLiteral(ImmediateLiteralKind::Octal) got {t:?}"),
        }
    }

    #[test]
    fn expect_zero_decimal_literal() {
        let asm = b"0";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        assert_eq!(t.tokens[0], Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span: Range(0, 1) });
    }

    #[test]
    fn expect_bracket() {
        let asm = b"[]]";
        let mut t = Tokenizer::from(asm.as_slice());

        t.process_next_token();
        t.process_next_token();
        t.process_next_token();
        assert_eq!(t.tokens[0], Token { kind: TokenKind::OpenBracket, span: Range(0, 1) });
        assert_eq!(t.tokens[1], Token { kind: TokenKind::ClosedBracket, span: Range(1, 2) });
        assert_eq!(t.tokens[2], Token { kind: TokenKind::ClosedBracket, span: Range(2, 3) });
    }

    #[test]
    fn expect_colon() {
        let asm = b":";
        let mut t = Tokenizer::from(asm.as_slice());
        t.process_next_token();
        assert_eq!(t.tokens[0], Token { kind: TokenKind::Colon, span: Range(0, 1) });
    }

    #[test]
    fn newline_after_number_immediate_literal() {
        let asm = b"0
1
-1
01
0xF
0o7
0d1
0b1
";

        let mut t = Tokenizer::from(asm.as_slice());
        t.run();
        println!("{:?}", t.tokens);
        assert_eq!(t.tokens[0], Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span: Range(0, 1) });
        assert_eq!(t.tokens[0].resolve(asm), "ImmediateLiteral: Decimal: '0'");
        assert_eq!(t.tokens[1], Token { kind: TokenKind::Newline, span: Range(1, 2) });
        assert_eq!(t.tokens[2], Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span: Range(2, 3) });
        assert_eq!(t.tokens[2].resolve(asm), "ImmediateLiteral: Decimal: '1'");
        assert_eq!(t.tokens[3], Token { kind: TokenKind::Newline, span: Range(3, 4) });
        assert_eq!(t.tokens[4], Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span: Range(4, 6) });
        assert_eq!(t.tokens[4].resolve(asm), "ImmediateLiteral: Decimal: '-1'");
        assert_eq!(t.tokens[5], Token { kind: TokenKind::Newline, span: Range(6, 7) });
        assert_eq!(t.tokens[6], Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span: Range(7, 9) });
        assert_eq!(t.tokens[6].resolve(asm), "ImmediateLiteral: Decimal: '01'");
        assert_eq!(t.tokens[7], Token { kind: TokenKind::Newline, span: Range(9, 10) });
        assert_eq!(
            t.tokens[8],
            Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Hexadecimal), span: Range(12, 13) } // skip '0x' prefix
        );
        assert_eq!(t.tokens[8].resolve(asm), "ImmediateLiteral: Hexadecimal: 'F'");
        assert_eq!(t.tokens[9], Token { kind: TokenKind::Newline, span: Range(13, 14) });
        assert_eq!(t.tokens[10], Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Octal), span: Range(16, 17) }); // skip '0o' prefix
        assert_eq!(t.tokens[10].resolve(asm), "ImmediateLiteral: Octal: '7'");
        assert_eq!(t.tokens[11], Token { kind: TokenKind::Newline, span: Range(17, 18) });
        assert_eq!(t.tokens[12], Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Decimal), span: Range(20, 21) }); // skip '0d' prefix
        assert_eq!(t.tokens[12].resolve(asm), "ImmediateLiteral: Decimal: '1'");
        assert_eq!(t.tokens[13], Token { kind: TokenKind::Newline, span: Range(21, 22) });
        assert_eq!(t.tokens[14], Token { kind: TokenKind::ImmediateLiteral(ImmediateLiteralKind::Binary), span: Range(24, 25) }); // skip '0b' prefix
        assert_eq!(t.tokens[14].resolve(asm), "ImmediateLiteral: Binary: '1'");
        assert_eq!(t.tokens[15], Token { kind: TokenKind::Newline, span: Range(25, 26) });
    }
}
