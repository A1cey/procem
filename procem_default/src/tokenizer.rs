use thiserror::Error;

#[doc(hidden("Only public for benchmarks."))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token<'a> {
    Label(String),
    Register(String),
    Literal(Literal<'a>),
    Instruction(String),
    Comma,
    End,
}

#[doc(hidden("Only public for benchmarks."))]
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Literal<'a> {
    Decimal(&'a str),
    Binary(&'a str),
    Hexadecimal(&'a str),
    Octal(&'a str),
    Boolean(bool),
    String(&'a str),
    Char(char),
}

#[doc(hidden("Only public for benchmarks."))]
pub struct Tokenizer<'a> {
    tokens: Vec<Token<'a>>,
    curr_idx: usize,
    token_start_idx: usize,
    input: &'a str,
    bytes: &'a [u8],
    input_len: usize,
    errors: Option<Vec<TokenizerError>>,
}

impl Tokenizer<'_> {
    const fn from(input: &str) -> Tokenizer<'_> {
        Tokenizer {
            tokens: Vec::new(),
            curr_idx: 0,
            token_start_idx: 0,
            input,
            bytes: input.as_bytes(),
            input_len: input.len(),
            errors: None,
        }
    }

    #[doc(hidden("Only public for benchmarks."))]
    pub fn tokenize(input: &str) -> Result<Vec<Token<'_>>, Vec<TokenizerError>> {
        let mut tokenizer = Tokenizer::from(input);

        tokenizer.run();

        match tokenizer.errors {
            Some(errors) => Err(errors),
            None => Ok(tokenizer.tokens),
        }
    }

    #[inline(never)]
    fn run(&mut self) {
        while self.curr_idx < self.input_len {
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
    }

    #[inline(never)]
    fn add_error(&mut self, err: TokenizerError) {
        self.errors.get_or_insert_default().push(err);
    }

    #[inline(never)]
    fn get_curr_byte(&self) -> u8 {
        self.bytes.get(self.curr_idx).map_or_else(
            || {
                unreachable!(
                    "The index should not be greater or equal to the length of the input. This should never happen."
                )
            },
            |b| b.to_ascii_uppercase(),
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

        self.tokens.push(Token::Label(
            self.input[self.token_start_idx..self.curr_idx].to_uppercase(),
        ));
    }

    #[inline(never)]
    fn expect_instruction(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_alphabetic() {
            self.curr_idx += 1;
        }

        let inst = self.input[self.token_start_idx..self.curr_idx].to_uppercase();

        let token = if inst == "END" {
            Token::End
        } else {
            Token::Instruction(inst)
        };

        self.tokens.push(token);
    }

    #[inline(never)]
    fn expect_register(&mut self) {
        self.curr_idx += 1;

        while self.curr_idx < self.input_len && self.get_curr_byte().is_ascii_digit() {
            self.curr_idx += 1;
        }

        self.tokens.push(Token::Register(
            self.input[self.token_start_idx..self.curr_idx].to_uppercase(),
        ));
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
        self.tokens.push(Token::Literal(Literal::String(
            &self.input[self.token_start_idx + 2..self.curr_idx],
        )));
    }

    #[inline(never)]
    fn expect_numeric_literal(&mut self) {
        let literal = if self.get_curr_byte() == b'0' {
            self.curr_idx += 1;
            self.token_start_idx = self.curr_idx;
            match self.get_curr_byte() {
                b'B' => {
                    self.set_curr_idx_to_token_end();
                    Literal::Binary(&self.input[self.token_start_idx + 1..=self.curr_idx])
                }
                b'X' => {
                    self.set_curr_idx_to_token_end();
                    Literal::Hexadecimal(&self.input[self.token_start_idx + 1..=self.curr_idx])
                }
                b'O' => {
                    self.set_curr_idx_to_token_end();
                    Literal::Octal(&self.input[self.token_start_idx + 1..=self.curr_idx])
                }
                b'D' => {
                    self.set_curr_idx_to_token_end();
                    Literal::Decimal(&self.input[self.token_start_idx + 1..=self.curr_idx])
                }
                _ => {
                    self.set_curr_idx_to_token_end();
                    Literal::Decimal(&self.input[self.token_start_idx - 1..self.curr_idx])
                }
            }
        } else {
            self.set_curr_idx_to_token_end();
            Literal::Decimal(&self.input[self.token_start_idx + 1..=self.curr_idx])
        };

        self.tokens.push(Token::Literal(literal));
    }

    #[inline(never)]
    fn expect_boolean_true_literal(&mut self) {
        self.curr_idx += 4; // len of "true"

        // +1 to ignore prefix #
        match self.input[self.token_start_idx + 1..self.curr_idx]
            .to_uppercase()
            .as_str()
        {
            "TRUE" => self.tokens.push(Token::Literal(Literal::Boolean(true))),
            _ => self.add_error(TokenizerError::BooleanTrueLiteral {
                idx: self.token_start_idx,
            }),
        }
    }

    #[inline(never)]
    fn expect_boolean_false_literal(&mut self) {
        self.curr_idx += 5; // len of "false"

        // +1 to ignore prefix #
        match self.input[self.token_start_idx + 1..self.curr_idx]
            .to_uppercase()
            .as_str()
        {
            "FALSE" => self.tokens.push(Token::Literal(Literal::Boolean(false))),
            _ => self.add_error(TokenizerError::BooleanFalseLiteral {
                idx: self.token_start_idx,
            }),
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
    use super::*;

    #[test]
    fn test_tokenize() {}

    #[test]
    fn test_run() {
        let mut t = Tokenizer::from(
            "
            .main
                MOV R0, #5
                nop
                MOV R256, #0xBc2a
                Mul R0, r256
                JMP .main
            ",
        );
        t.run();
        assert_eq!(
            t.tokens,
            vec![
                Token::Label(".MAIN".into()),
                Token::Instruction("MOV".into()),
                Token::Register("R0".into()),
                Token::Comma,
                Token::Literal(Literal::Decimal("5")),
                Token::Instruction("NOP".into()),
                Token::Instruction("MOV".into()),
                Token::Register("R256".into()),
                Token::Comma,
                Token::Literal(Literal::Hexadecimal("Bc2a".into())),
                Token::Instruction("MUL".into()),
                Token::Register("R0".into()),
                Token::Comma,
                Token::Register("R256".into()),
                Token::Instruction("JMP".into()),
                Token::Label(".MAIN".into())
            ]
        );
    }

    #[test]
    fn test_add_error() {
        let mut t = Tokenizer::from("");
        let err = TokenizerError::TokenStart { start: ' ', idx: 0 };
        assert!(t.errors.is_none());
        t.add_error(err.clone());
        assert_eq!(t.errors.unwrap(), vec![err.into()]);
    }

    #[test]
    fn test_get_curr_byte() {
        let t = Tokenizer::from(".main mov");
        assert_eq!(t.get_curr_byte(), b'.');
    }

    #[test]
    #[should_panic]
    fn test_get_curr_byte_out_of_bounds() {
        let mut t = Tokenizer::from(".main");
        assert_eq!(t.get_curr_byte(), b'.');
        t.curr_idx += 5;
        let _ = t.get_curr_byte(); // panic
    }

    #[test]
    fn test_expect_label() {
        let mut t = Tokenizer::from(".main");
        t.expect_label();
        assert_eq!(t.tokens[0], Token::Label(".MAIN".into()));
        t = Tokenizer::from(".MAIN");
        t.expect_label();
        assert_eq!(t.tokens[0], Token::Label(".MAIN".into()))
    }

    #[test]
    fn test_expect_instruction() {
        let mut t = Tokenizer::from("mov");
        t.expect_instruction();
        assert_eq!(t.tokens[0], Token::Instruction("MOV".into()));
        t = Tokenizer::from("JMP");
        t.expect_instruction();
        assert_eq!(t.tokens[0], Token::Instruction("JMP".into()));
    }

    #[test]
    fn test_expect_register() {
        let mut t = Tokenizer::from("R0");
        t.expect_register();
        assert_eq!(t.tokens[0], Token::Register("R0".into()));
        t = Tokenizer::from("R4242");
        t.expect_register();
        assert_eq!(t.tokens[0], Token::Register("R4242".into()));
    }

    #[test]
    fn test_expect_comma() {
        let mut t = Tokenizer::from(",");
        t.expect_comma();
        assert_eq!(t.tokens[0], Token::Comma);
    }

    #[test]
    fn test_expect_literal() {
        let mut t = Tokenizer::from("#42");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Decimal("42")));
        let mut t = Tokenizer::from("#0x4H");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Hexadecimal("4H".into())));
        let mut t = Tokenizer::from("#0b010110");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Binary("010110")));
        let mut t = Tokenizer::from("#0o743");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Octal("743")));
        let mut t = Tokenizer::from("#true");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Boolean(true)));
        let mut t = Tokenizer::from("#false");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Boolean(false)));
        let mut t = Tokenizer::from("#\"Hello, there\"");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::String("Hello, there")));
        let mut t = Tokenizer::from("#\'7\'");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Char('7')));
    }

    #[test]
    fn test_expect_char_literal() {
        let mut t = Tokenizer::from("#\'B\'");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Char('B')));
    }

    #[test]
    fn test_expect_string_literal() {
        let mut t = Tokenizer::from("#\"Jajajajaja2498291849102+#amfl929r2jlsamfa3\"");
        t.expect_literal();
        assert_eq!(
            t.tokens[0],
            Token::Literal(Literal::String("Jajajajaja2498291849102+#amfl929r2jlsamfa3"))
        );
    }

    #[test]
    fn test_expect_numeric_literal() {
        let mut t = Tokenizer::from("#42");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Decimal("42")));
        t = Tokenizer::from("#0d42");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Decimal("42")));
        t = Tokenizer::from("#-42");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Decimal("-42")));
        t = Tokenizer::from("#0x4H");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Hexadecimal("4H".into())));
        t = Tokenizer::from("#0b010110");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Binary("010110")));
        t = Tokenizer::from("#0o743");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Octal("743")));
    }

    #[test]
    fn test_expect_boolean_true_literal() {
        let mut t = Tokenizer::from("#TRUE");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Boolean(true)));
    }

    #[test]
    fn test_expect_boolean_false_literal() {
        let mut t = Tokenizer::from("#FALSE");
        t.expect_literal();
        assert_eq!(t.tokens[0], Token::Literal(Literal::Boolean(false)));
    }
}
