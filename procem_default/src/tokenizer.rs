use thiserror::Error;

use ars::range::Range;

#[doc(hidden("Only public for benchmarks."))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    Label(Range),
    Register(Range),
    Literal(Literal),
    Instruction(Range),
    Comma,
    End,
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
pub struct Tokenizer<'asm> {
    tokens: Vec<Token>,
    curr_idx: usize,
    token_start_idx: usize,
    input: &'asm mut [u8],
    input_len: usize,
    errors: Option<Vec<TokenizerError>>,
}

impl Tokenizer<'_> {
    const fn from(input: &mut [u8]) -> Tokenizer<'_> {
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
    pub fn tokenize(input: &mut [u8]) -> Result<Vec<Token>, Vec<TokenizerError>> {
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
            b'.' => self.expect_label(),
            b'R' => self.expect_register(),
            b'#' => self.expect_literal(),
            b',' => self.expect_comma(),
            b if b.is_ascii_alphabetic() => self.expect_instruction(),
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

    #[inline]
    fn add_error(&mut self, err: TokenizerError) {
        self.errors.get_or_insert_default().push(err);
    }

    #[inline(never)]
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

    #[inline(never)]
    fn set_curr_idx_to_token_end(&mut self) {
        if self.get_curr_byte().is_ascii_whitespace() {
            return;
        }

        while self.curr_idx < self.input_len && !self.get_curr_byte().is_ascii_whitespace() {
            self.curr_idx += 1;
        }

        self.curr_idx -= 1;
    }

    #[inline(never)]
    fn expect_label(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_alphabetic() {
            self.curr_idx += 1;
        }

        let start = self.token_start_idx;
        let end = self.curr_idx;

        self.input[start..end].make_ascii_uppercase();

        self.tokens.push(Token::Label(Range(start, end)));
    }

    #[inline(never)]
    fn expect_instruction(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_alphabetic() {
            self.curr_idx += 1;
        }

        let start = self.token_start_idx;
        let end = self.curr_idx;

        self.input[start..end].make_ascii_uppercase();

        let token = if &self.input[start..end] == b"END" {
            Token::End
        } else {
            Token::Instruction(Range(start, end))
        };

        self.tokens.push(token);
    }

    #[inline(never)]
    fn expect_register(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_digit() {
            self.curr_idx += 1;
        }

        let start = self.token_start_idx;
        let end = self.curr_idx;

        self.input[start..end].make_ascii_uppercase();

        self.tokens.push(Token::Register(Range(start, end)));
    }

    #[inline(never)]
    fn expect_comma(&mut self) {
        self.tokens.push(Token::Comma);
        self.curr_idx += 1;
    }

    #[inline(never)]
    fn expect_literal(&mut self) {
        self.curr_idx += 1;

        match self.get_curr_byte() {
            b'\'' => self.expect_char_literal(),
            b'"' => self.expect_string_literal(),
            b'-' => self.expect_numeric_literal(),
            b if b.is_ascii_digit() => self.expect_numeric_literal(),
            b'T' => self.expect_boolean_true_literal(),
            b'F' => self.expect_boolean_false_literal(),
            _ => self.add_error(TokenizerError::Literal { idx: self.curr_idx }),
        }

        self.curr_idx += 1;
    }

    #[inline(never)]
    fn expect_char_literal(&mut self) {
        self.curr_idx += 1;

        let b = self.get_curr_byte();

        self.curr_idx += 1;

        match self.get_curr_byte() {
            b'\'' => self.tokens.push(Token::Literal(Literal::Char(char::from(b)))),
            _ => self.add_error(TokenizerError::CharLiteral { idx: self.curr_idx }),
        }
    }

    #[inline(never)]
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

    #[inline(never)]
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

    #[inline(never)]
    fn expect_boolean_true_literal(&mut self) {
        self.curr_idx += 4; // len of "true"

        // +1 to ignore prefix #
        let lit = &mut self.input[self.token_start_idx + 1..self.curr_idx];

        lit.make_ascii_uppercase();

        if lit == b"TRUE" {
            self.tokens.push(Token::Literal(Literal::Boolean(true)));
        } else {
            self.add_error(TokenizerError::BooleanTrueLiteral {
                idx: self.token_start_idx,
            });
        }
    }

    #[inline(never)]
    fn expect_boolean_false_literal(&mut self) {
        self.curr_idx += 5; // len of "false"

        // +1 to ignore prefix #
        let lit = &mut self.input[self.token_start_idx + 1..self.curr_idx];

        lit.make_ascii_uppercase();

        if lit == b"FALSE" {
            self.tokens.push(Token::Literal(Literal::Boolean(false)));
        } else {
            self.add_error(TokenizerError::BooleanFalseLiteral {
                idx: self.token_start_idx,
            });
        }
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq)]
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
        .main
            MOV R0, #5
            nop
            MOV R256, #0xBc2a
            Mul R0, r256
            JMP .main
        "
        .bytes()
        .collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.run();

        let mut tokens = t.tokens.iter().into_iter();

        match tokens.next().unwrap() {
            Token::Label(r) => assert_eq!(&asm[r], ".MAIN".as_bytes()),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Instruction(r) => assert_eq!(&asm[r], "MOV".as_bytes()),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Register(r) => assert_eq!(&asm[r], ("R0".as_bytes())),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Literal(l) => match l {
                Literal::Decimal(r) => assert_eq!(&asm[r], b"5"),
                _ => panic!(),
            },
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Instruction(r) => assert_eq!(&asm[r], ("NOP".as_bytes())),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Instruction(r) => assert_eq!(&asm[r], ("MOV".as_bytes())),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Register(r) => assert_eq!(&asm[r], ("R256".as_bytes())),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Literal(l) => match l {
                Literal::Hexadecimal(r) => assert_eq!(&asm[r], ("Bc2a".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Instruction(r) => assert_eq!(&asm[r], ("MUL".as_bytes())),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Register(r) => assert_eq!(&asm[r], ("R0".as_bytes())),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Comma => {}
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Register(r) => assert_eq!(&asm[r], ("R256".as_bytes())),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Instruction(r) => assert_eq!(&asm[r], ("JMP".as_bytes())),
            _ => panic!(),
        }
        match tokens.next().unwrap() {
            Token::Label(r) => assert_eq!(&asm[r], ".MAIN".as_bytes()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_add_error() {
        let mut asm: [u8; 0] = [];
        let mut t = Tokenizer::from(&mut asm);
        let err = TokenizerError::TokenStart { start: ' ', idx: 0 };
        assert!(t.errors.is_none());
        t.add_error(err.clone());
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
        let mut asm = ".main".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        assert_eq!(t.get_curr_byte(), b'.');
        t.curr_idx += 5;
        let _ = t.get_curr_byte(); // panic
    }

    #[test]
    fn test_expect_label() {
        let mut asm = ".main".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_label();
        match t.tokens[0].clone() {
            Token::Label(r) => assert_eq!(&asm[r.clone()], ".MAIN".as_bytes()),
            _ => panic!(),
        }
        asm = ".MAIN".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_label();
        match t.tokens[0].clone() {
            Token::Label(r) => assert_eq!(&asm[r.clone()], ".MAIN".as_bytes()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_instruction() {
        let mut asm = "mov".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_instruction();
        match t.tokens[0].clone() {
            Token::Instruction(r) => assert_eq!(&asm[r.clone()], "MOV".as_bytes()),
            _ => panic!(),
        }
        asm = "JMP".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_instruction();
        match t.tokens[0].clone() {
            Token::Instruction(r) => assert_eq!(&asm[r.clone()], "JMP".as_bytes()),
            _ => panic!(),
        }
    }

    #[test]
    fn test_expect_register() {
        let mut asm = "R0".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_register();
        match t.tokens[0].clone() {
            Token::Register(r) => assert_eq!(&asm[r.clone()], "R0".as_bytes()),
            _ => panic!(),
        }
        asm = "R4242".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_register();
        match t.tokens[0].clone() {
            Token::Register(r) => assert_eq!(&asm[r.clone()], "R4242".as_bytes()),
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
                Literal::Decimal(r) => assert_eq!(&asm[r.clone()], ("42".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let mut asm = "#0x4H".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Hexadecimal(r) => assert_eq!(&asm[r.clone()], ("4H".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let mut asm = "#0b010110".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Binary(r) => assert_eq!(&asm[r.clone()], ("010110".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        let mut asm = "#0o743".bytes().collect::<Vec<_>>();
        let mut t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Octal(r) => assert_eq!(&asm[r.clone()], ("743".as_bytes())),
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
                Literal::String(r) => assert_eq!(&asm[r.clone()], ("Hello, there".as_bytes())),
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
                    &asm[r.clone()],
                    ("Jajajajaja2498291849102+#amfl929r2jlsamfa3".as_bytes())
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
                Literal::Decimal(r) => assert_eq!(&asm[r.clone()], ("42".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#0d42".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Decimal(r) => assert_eq!(&asm[r.clone()], ("42".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#-42".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Decimal(r) => assert_eq!(&asm[r.clone()], ("-42".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#0x4H".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Hexadecimal(r) => assert_eq!(&asm[r.clone()], ("4H".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#0b010110".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Binary(r) => assert_eq!(&asm[r.clone()], ("010110".as_bytes())),
                _ => panic!(),
            },
            _ => panic!(),
        }
        asm = "#0o743".bytes().collect::<Vec<_>>();
        t = Tokenizer::from(&mut asm);
        t.expect_literal();
        match t.tokens[0].clone() {
            Token::Literal(l) => match l {
                Literal::Octal(r) => assert_eq!(&asm[r.clone()], ("743".as_bytes())),
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
