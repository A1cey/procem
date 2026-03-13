use thiserror::Error;

use ars::range::Range;

#[doc(hidden("Only public for benchmarks."))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Label(Range),
    Register(Range),
    Literal(Literal),
    LabelOrInstruction(Range), // labels after jump instructions for example do not end with ':' and cannot be distinguished from instructions by the tokenizer
    Comma,
    End,
    Section(Range),
}

#[doc(hidden("Only public for benchmarks."))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal {
    Decimal(Range),
    Binary(Range),
    Hexadecimal(Range),
    Octal(Range),
    Boolean(bool),
    String(Range),
    Char(char),
}

#[doc(hidden("Only public for benchmarks."))]
pub struct Tokenizer<'input> {
    tokens: Vec<Token>,
    curr_idx: usize,
    token_start_idx: usize,
    input: &'input [u8],
    input_len: usize,
    errors: Option<Vec<TokenizerError>>,
}

impl Tokenizer<'_> {
    const fn from(input: &[u8]) -> Tokenizer<'_> {
        Tokenizer {
            tokens: Vec::new(),
            curr_idx: 0,
            token_start_idx: 0,
            input_len: input.len(),
            input,
            errors: None,
        }
    }

    #[doc(hidden("Only public for benchmarks."))]
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

    fn process_next_token(&mut self) {
        self.token_start_idx = self.curr_idx;

        match self.get_curr_byte() {
            b'.' => self.expect_section(),
            b'R' => self.expect_register(),
            b'#' => self.expect_literal(),
            b',' => self.expect_comma(),
            b if Self::is_valid_char(b) => self.expect_instruction_or_label(),
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
        self.input.get(self.curr_idx).map_or_else(
            || {
                unreachable!(
                    "The index should not be greater or equal to the length of the input. This should never happen."
                )
            },
            u8::to_ascii_uppercase,
        )
    }

    fn set_curr_idx_to_token_end(&mut self) {
        if self.get_curr_byte().is_ascii_whitespace() {
            return;
        }

        while self.curr_idx < self.input_len && !self.get_curr_byte().is_ascii_whitespace() {
            self.curr_idx += 1;
        }

        self.curr_idx -= 1;
    }

    fn expect_section(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_alphabetic() {
            self.curr_idx += 1;
        }

        let start = self.token_start_idx + 1; // do not count '.'
        let end = self.curr_idx;

        self.tokens.push(Token::Section(Range(start, end)));
    }

    fn expect_instruction_or_label(&mut self) {
        self.curr_idx += 1;

        let mut is_label = false;

        while self.curr_idx < self.input_len {
            let b = self.get_curr_byte();
            if Self::is_valid_char(b) {
                self.curr_idx += 1;
            } else if b == b':' {
                is_label = true;
                break;
            } else {
                break;
            }
        }

        let start = self.token_start_idx;
        let end = self.curr_idx;

        let token = if is_label {
            self.curr_idx += 1; // skip over the ':'
            Token::Label(Range(start, end))
        } else {
            let is_end_literal = |token: &[u8]| {
                token.len() == 3
                    && (token[0] == b'e' || token[0] == b'E')
                    && (token[1] == b'n' || token[1] == b'N')
                    && (token[2] == b'd' || token[2] == b'D')
            };

            if is_end_literal(&self.input[start..end]) {
                Token::End
            } else {
                Token::LabelOrInstruction(Range(start, end))
            }
        };

        self.tokens.push(token);
    }

    fn expect_register(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_digit() {
            self.curr_idx += 1;
        }

        let start = self.token_start_idx;
        let end = self.curr_idx;

        self.tokens.push(Token::Register(Range(start, end)));
    }

    fn expect_comma(&mut self) {
        self.tokens.push(Token::Comma);
        self.curr_idx += 1;
    }

    fn expect_literal(&mut self) {
        self.curr_idx += 1;

        match self.get_curr_byte() {
            b'\'' => self.expect_char_literal(),
            b'"' => self.expect_string_literal(),
            b'-' => self.expect_numeric_literal(),
            b if b.is_ascii_digit() => self.expect_numeric_literal(),
            b'T' | b't' => self.expect_boolean_true_literal(),
            b'F' | b'f' => self.expect_boolean_false_literal(),
            _ => self.add_error(TokenizerError::Literal { idx: self.curr_idx }),
        }

        self.curr_idx += 1;
    }

    fn expect_char_literal(&mut self) {
        self.curr_idx += 1;

        let b = self.get_curr_byte();

        self.curr_idx += 1;

        match self.get_curr_byte() {
            b'\'' => self.tokens.push(Token::Literal(Literal::Char(char::from(b)))),
            _ => self.add_error(TokenizerError::CharLiteral { idx: self.curr_idx }),
        }
    }

    fn expect_string_literal(&mut self) {
        self.curr_idx += 1;

        while self.get_curr_byte() != b'"' {
            self.curr_idx += 1;
        }

        // +2 to ignore the prefix #"
        self.tokens.push(Token::Literal(Literal::String(Range(
            self.token_start_idx + 2,
            self.curr_idx,
        ))));
    }

    fn expect_numeric_literal(&mut self) {
        let literal = if self.get_curr_byte() == b'0' {
            self.curr_idx += 1;
            self.token_start_idx = self.curr_idx;
            match self.get_curr_byte() {
                b'B' => {
                    self.set_curr_idx_to_token_end();
                    Literal::Binary(Range(self.token_start_idx + 1, self.curr_idx + 1))
                }
                b'X' => {
                    self.set_curr_idx_to_token_end();
                    Literal::Hexadecimal(Range(self.token_start_idx + 1, self.curr_idx + 1))
                }
                b'O' => {
                    self.set_curr_idx_to_token_end();
                    Literal::Octal(Range(self.token_start_idx + 1, self.curr_idx + 1))
                }
                b'D' => {
                    self.set_curr_idx_to_token_end();
                    Literal::Decimal(Range(self.token_start_idx + 1, self.curr_idx + 1))
                }
                _ => {
                    self.set_curr_idx_to_token_end();
                    Literal::Decimal(Range(self.token_start_idx - 1, self.curr_idx))
                }
            }
        } else {
            self.set_curr_idx_to_token_end();
            Literal::Decimal(Range(self.token_start_idx + 1, self.curr_idx + 1))
        };

        let lit = Token::Literal(literal);

        self.tokens.push(lit);
    }

    fn expect_boolean_true_literal(&mut self) {
        self.curr_idx += 4; // len of "true"

        // +1 to ignore prefix #
        let lit = &self.input[self.token_start_idx + 1..self.curr_idx];

        let is_true_lit = |lit: &[u8]| {
            lit.len() == 4
                && (lit[0] == b't' || lit[0] == b'T')
                && (lit[1] == b'r' || lit[1] == b'R')
                && (lit[2] == b'u' || lit[2] == b'U')
                && (lit[3] == b'e' || lit[3] == b'E')
        };

        if is_true_lit(lit) {
            self.tokens.push(Token::Literal(Literal::Boolean(true)));
        } else {
            self.add_error(TokenizerError::BooleanTrueLiteral {
                idx: self.token_start_idx,
            });
        }
    }

    fn expect_boolean_false_literal(&mut self) {
        self.curr_idx += 5; // len of "false"

        // +1 to ignore prefix #
        let lit = &self.input[self.token_start_idx + 1..self.curr_idx];

        let is_false_lit = |lit: &[u8]| {
            lit.len() == 5
                && (lit[0] == b'f' || lit[0] == b'F')
                && (lit[1] == b'a' || lit[1] == b'A')
                && (lit[2] == b'l' || lit[2] == b'L')
                && (lit[3] == b's' || lit[3] == b'S')
                && (lit[4] == b'e' || lit[4] == b'E')
        };

        if is_false_lit(lit) {
            self.tokens.push(Token::Literal(Literal::Boolean(false)));
        } else {
            self.add_error(TokenizerError::BooleanFalseLiteral {
                idx: self.token_start_idx,
            });
        }
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
    #[error("Expected boolean literal TRUE/true at idx {idx}.")]
    BooleanTrueLiteral { idx: usize },
    #[error("Expected boolean literal FALSE/false at idx {idx}.")]
    BooleanFalseLiteral { idx: usize },
    #[error("Invalid character {character} at idx: {idx} in label name starting at idx {token_start_idx}.")]
    InvalidLabelName {
        token_start_idx: usize,
        idx: usize,
        character: char,
    },
}

#[cfg(test)]
mod test {
    use std::panic;

    use super::*;

    #[test]
    fn test_tokenize() {}

    #[test]
    fn test_run() {
        let mut asm = "
        .code
        main:
            MOV R0, #5
            nop
            MOV R256, #0xBc2a
            Mul R0, r256
            JMP main
        "
        .bytes()
        .collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.run();

        let mut tokens = t.tokens.iter().into_iter();

        match tokens.next().unwrap() {
            Token::Section(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "code"),
            t => panic!("expected Section, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Label(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "main"),
            t => panic!("expected Label, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::LabelOrInstruction(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "MOV"),
            t => panic!("expected LabelOrInstruction, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Register(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("R0")),
            t => panic!("expected Register, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            t => panic!("expected Comma, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Literal(l) => match l {
                Literal::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "5"),
                t => panic!("expected Decimal, got {t:?}"),
            },
            t => panic!("expected Literal, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::LabelOrInstruction(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("nop")),
            t => panic!("expected LabelOrInstruction, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::LabelOrInstruction(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("MOV")),
            t => panic!("expected LabelOrInstruction, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Register(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("R256")),
            t => panic!("expected Register, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            t => panic!("expected Comma, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Literal(l) => match l {
                Literal::Hexadecimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("Bc2a")),
                t => panic!("expected Hexadecimal, got {t:?}"),
            },
            t => panic!("expected Literal, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::LabelOrInstruction(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("Mul")),
            t => panic!("expected LabelOrInstruction, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Register(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("R0")),
            t => panic!("expected Register, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            t => panic!("expected Comma, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::Register(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("r256")),
            t => panic!("expected Register, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::LabelOrInstruction(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("JMP")),
            t => panic!("expected LabelOrInstruction, got {t:?}"),
        }
        match tokens.next().unwrap() {
            Token::LabelOrInstruction(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "main"),
            t => panic!("expected LabelOrInstruction, got {t:?}"),
        }
    }

    #[test]
    fn test_add_error() {
        let mut asm: [u8; 0] = [];
        let mut t = Tokenizer::from(&mut asm);
        let err = TokenizerError::TokenStart { start: ' ', idx: 0 };
        assert!(t.errors.is_none());
        t.add_error(err);
        assert_eq!(t.errors.unwrap(), vec![err.into()]);
    }

    #[test]
    fn test_get_curr_byte() {
        let mut asm = ".main mov".bytes().collect::<Vec<_>>();
        let t = Tokenizer::from(&mut asm);
        assert_eq!(t.get_curr_byte(), b'.');
    }

    #[test]
    #[should_panic]
    fn test_get_curr_byte_out_of_bounds() {
        let mut asm = "main:".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        assert_eq!(t.get_curr_byte(), b'm');
        t.curr_idx += 5;
        let _ = t.get_curr_byte(); // panic
    }

    #[test]
    fn test_expect_label() {
        let mut asm = "main:".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_instruction_or_label();
        match t.tokens[0].clone() {
            Token::Label(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "main"),
            _ => panic!(),
        }
        asm = "MAIN:".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_instruction_or_label();
        match t.tokens[0].clone() {
            Token::Label(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "MAIN"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_section() {
        let mut asm = ".code".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_section();
        match t.tokens[0].clone() {
            Token::Section(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "code"),
            _ => panic!(),
        }
        asm = ".DATA".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_section();
        match t.tokens[0].clone() {
            Token::Section(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "DATA"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_end() {
        let mut asm = "end".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_instruction_or_label();
        assert_eq!(t.tokens[0].clone(), Token::End);
        asm = "END".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_instruction_or_label();
        assert_eq!(t.tokens[0].clone(), Token::End);
    }

    #[test]
    fn test_expect_instruction() {
        let mut asm = "mov".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_instruction_or_label();
        match t.tokens[0].clone() {
            Token::LabelOrInstruction(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "mov"),
            _ => panic!(),
        }
        asm = "JMP".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_instruction_or_label();
        match t.tokens[0].clone() {
            Token::LabelOrInstruction(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "JMP"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_register() {
        let mut asm = "R0".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_register();
        match t.tokens[0].clone() {
            Token::Register(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "R0"),
            _ => panic!(),
        }
        asm = "R4242".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_register();
        match t.tokens[0].clone() {
            Token::Register(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), "R4242"),
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_comma() {
        let mut asm = ",".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_comma();
        assert_eq!(t.tokens[0], Token::Comma);
    }

    #[test]
    fn test_expect_literal() {
        let mut asm = "#42".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("42")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let mut asm = "#0x4H".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Hexadecimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("4H")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let mut asm = "#0b010110".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Binary(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("010110")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let mut asm = "#0o743".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Octal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("743")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let mut asm = "#true".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Boolean(true)));
        let mut asm = "#false".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Boolean(false)));
        let mut asm = "#\"Hello, there\"".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::String(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("Hello, there")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let mut asm = "#\'7\'".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Char('7')));
    }

    #[test]
    fn test_expect_char_literal() {
        let mut asm = "#\'B\'".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Char('B')));
    }

    #[test]
    fn test_expect_string_literal() {
        let mut asm = "#\"Jajajajaja2498291849102+#amfl929r2jlsamfa3\""
            .bytes()
            .collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::String(r) => assert_eq!(
                    String::from_utf8_lossy(&asm[r]),
                    ("Jajajajaja2498291849102+#amfl929r2jlsamfa3")
                ),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_numeric_literal() {
        let mut asm = "#42".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("42")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#0d42".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("42")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#-42".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Decimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("-42")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#0x4H".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Hexadecimal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("4H")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#0b010110".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Binary(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("010110")),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#0o743".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Octal(r) => assert_eq!(String::from_utf8_lossy(&asm[r]), ("743")),
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_boolean_true_literal() {
        let mut asm = "#TRUE".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Boolean(true)));
    }

    #[test]
    fn test_expect_boolean_false_literal() {
        let mut asm = "#FALSE".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Boolean(false)));
    }
}
