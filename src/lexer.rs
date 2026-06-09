use crate::token::{Token, TokenCode};

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
    pub line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        let mut chars: Vec<char> = source.chars().collect();
        chars.push('\0'); //push null terminator
        Self {
            input: chars,
            pos: 0,
            line: 1,
        }
    }

    fn ch(&self) -> char {
        if self.pos < self.input.len() {
            self.input[self.pos]
        } else {
            '\0'
        }
    }

    fn consume(&mut self) {
        self.pos += 1;
    }

    fn err(&self, msg: &str) -> ! {
        eprintln!("error in line {}: {}", self.line, msg);
        std::process::exit(-1);
    }

    pub fn get_next_token(&mut self) -> Token {
        let mut state = 0;
        let mut start_pos = self.pos;

        loop {
            let ch = self.ch();

            match state {
                0 => {
                    //initial state
                    if ch.is_whitespace() {
                        if ch == '\n' {
                            self.line += 1;
                        }
                        self.consume();
                    } else if ch == '0' {
                        start_pos = self.pos;
                        self.consume();
                        state = 20; //number prefix
                    } else if ('1'..='9').contains(&ch) {
                        start_pos = self.pos;
                        self.consume();
                        state = 2; //decimal
                    } else if ch.is_alphabetic() || ch == '_' {
                        start_pos = self.pos;
                        self.consume();
                        state = 1; //id or keyword
                    } else if ch == '/' {
                        start_pos = self.pos;
                        self.consume();
                        state = 12; // comment or div
                    } else if ch == '"' {
                        start_pos = self.pos;
                        self.consume();
                        state = 10; // stirng
                    } else if ch == '\'' {
                        start_pos = self.pos;
                        self.consume();
                        state = 8; // char
                    } else if ch == '|' {
                        start_pos = self.pos;
                        self.consume();
                        state = 19; // OR
                    } else if ch == '&' {
                        start_pos = self.pos;
                        self.consume();
                        state = 18; // AND 
                    } else if ch == '>' {
                        start_pos = self.pos;
                        self.consume();
                        state = 17; // greater of greatereq 
                    } else if ch == '<' {
                        start_pos = self.pos;
                        self.consume();
                        state = 16; // less or lesseq
                    } else if ch == '=' {
                        start_pos = self.pos;
                        self.consume();
                        state = 15; // assign or equal
                    } else if ch == '!' {
                        start_pos = self.pos;
                        self.consume();
                        state = 14; // not or noteq
                    } else if ch == '\0' {
                        return Token {
                            code: TokenCode::END,
                            line: self.line,
                        };
                    } else {
                        self.consume();
                        let code = match ch {
                            '+' => TokenCode::ADD,
                            '-' => TokenCode::SUB,
                            '*' => TokenCode::MUL,
                            '.' => TokenCode::DOT,
                            ';' => TokenCode::SEMICOLON,
                            '(' => TokenCode::LPAR,
                            ')' => TokenCode::RPAR,
                            '{' => TokenCode::LACC,
                            '}' => TokenCode::RACC,
                            '[' => TokenCode::LBRACKET,
                            ']' => TokenCode::RBRACKET,
                            ',' => TokenCode::COMMA,
                            _ => self.err(&format!("Invalid character: {}", ch)),
                        };
                        return Token {
                            code,
                            line: self.line,
                        };
                    }
                }
                //State 1 : ID or keyword
                1 => {
                    if ch.is_alphanumeric() || ch == '_' {
                        self.consume()
                    } else {
                        let text: String = self.input[start_pos..self.pos].iter().collect();
                        let code = match text.as_str() {
                            "break" => TokenCode::BREAK,
                            "char" => TokenCode::CHAR,
                            "double" => TokenCode::DOUBLE,
                            "else" => TokenCode::ELSE,
                            "for" => TokenCode::FOR,
                            "if" => TokenCode::IF,
                            "int" => TokenCode::INT,
                            "return" => TokenCode::RETURN,
                            "struct" => TokenCode::STRUCT,
                            "void" => TokenCode::VOID,
                            "while" => TokenCode::WHILE,
                            _ => TokenCode::ID(text),
                        };
                        return Token {
                            code,
                            line: self.line,
                        };
                    }
                }
                //States 2-7 : Decimal and real numbers
                2 => {
                    //decimal loop
                    if ch.is_ascii_digit() {
                        self.consume()
                    } else if ch == '.' {
                        self.consume();
                        state = 3;
                    } else if ch == 'e' || ch == 'E' {
                        self.consume();
                        state = 5;
                    } else {
                        let text: String = self.input[start_pos..self.pos].iter().collect();
                        return Token {
                            code: TokenCode::CtInt(text.parse().unwrap_or(0)),
                            line: self.line,
                        };
                    }
                }
                3 => {
                    //check digit after .
                    if ch.is_ascii_digit() {
                        self.consume();
                        state = 4;
                    } else {
                        self.err("Expected digit after '.'")
                    }
                }
                4 => {
                    //real number calculator
                    if ch.is_ascii_digit() {
                        self.consume();
                    } else if ch == 'e' || ch == 'E' {
                        self.consume();
                        state = 5
                    } else {
                        //if no exponent is found go to final state CtReal
                        let text: String = self.input[start_pos..self.pos].iter().collect();
                        return Token {
                            code: TokenCode::CtReal(text.parse().unwrap_or(0.0)),
                            line: self.line,
                        };
                    }
                }
                5 => {
                    //saw exponent
                    if ch == '+' || ch == '-' {
                        self.consume();
                        state = 6;
                    } else if ch.is_ascii_digit() {
                        self.consume();
                        state = 7;
                    } else {
                        self.err("Expected '+', '-', or digit after exponent (e/E)");
                    }
                }
                6 => {
                    //next char after exponent sign must be a digit
                    if ch.is_ascii_digit() {
                        self.consume();
                        state = 7;
                    } else {
                        self.err("Expected digit after exponent sign");
                    }
                }
                7 => {
                    //exponent digit loop
                    if ch.is_ascii_digit() {
                        self.consume();
                    } else {
                        let text: String = self.input[start_pos..self.pos].iter().collect();
                        return Token {
                            code: TokenCode::CtReal(text.parse().unwrap_or(0.0)),
                            line: self.line,
                        };
                    }
                }
                // State 20-23: Hex, binary and octal numbers
                20 => {
                    //saw 0
                    match ch {
                        '0'..='7' => {
                            self.consume();
                            state = 21;
                        } //octal
                        'x' | 'X' => {
                            self.consume();
                            state = 22;
                        } //hexa
                        'b' | 'B' => {
                            self.consume();
                            state = 23;
                        } // binary
                        '8' | '9' => {
                            self.err("Invalid digit in octal constant");
                        }
                        'e' | 'E' => {
                            self.consume();
                            state = 5;
                        } // real with exponent
                        '.' => {
                            self.consume();
                            state = 3;
                        } //real
                        _ => {
                            return Token {
                                code: TokenCode::CtInt(0),
                                line: self.line,
                            };
                        }
                    }
                }
                21 => {
                    //octal
                    if ('0'..='7').contains(&ch) {
                        self.consume();
                    } else if ch == '8' || ch == '9' {
                        self.err("Invalid octal digit");
                    } else {
                        let text: String = self.input[start_pos + 1..self.pos].iter().collect();
                        return Token {
                            code: TokenCode::CtInt(i64::from_str_radix(&text, 8).unwrap_or(0)),
                            line: self.line,
                        };
                    }
                }
                22 => {
                    //hexa
                    if ch.is_ascii_hexdigit() {
                        self.consume();
                    } else {
                        if self.pos == start_pos + 2 {
                            self.err("Expected hexadecimal digits after 0x");
                        }
                        let text: String = self.input[start_pos + 2..self.pos].iter().collect();
                        return Token {
                            code: TokenCode::CtInt(i64::from_str_radix(&text, 16).unwrap_or(0)),
                            line: self.line,
                        };
                    }
                }
                23 => {
                    //binary
                    if ch == '0' || ch == '1' {
                        self.consume();
                    } else {
                        if self.pos == start_pos + 2 {
                            self.err("Expected binary digits after 0b");
                        }
                        let text: String = self.input[start_pos + 2..self.pos].iter().collect();
                        return Token {
                            code: TokenCode::CtInt(i64::from_str_radix(&text, 2).unwrap_or(0)),
                            line: self.line,
                        };
                    }
                }
                8 => {
                    //saw opening '
                    if ch == '\\' {
                        // It's an escape sequence like '\n' or '\\'
                        self.consume(); // Consume the '\'
                        self.consume(); // Consume the escaped character
                        state = 9; // Now wait for the closing quote
                    } else if ch != '\'' && ch != '\0' {
                        self.consume();
                        state = 9;
                    } else {
                        self.err("Empty or invalid character constant");
                    }
                }
                9 => {
                    //expecting closing '
                    if ch == '\'' {
                        self.consume();
                        // Handle extracting the correct char value (even if it was an escape sequence)
                        let val = if self.input[start_pos + 1] == '\\' {
                            match self.input[start_pos + 2] {
                                'n' => '\n',
                                't' => '\t',
                                'r' => '\r',
                                '0' => '\0',
                                '\\' => '\\',
                                '\'' => '\'',
                                _ => self.input[start_pos + 2],
                            }
                        } else {
                            self.input[start_pos + 1]
                        };
                        return Token {
                            code: TokenCode::CtChar(val),
                            line: self.line,
                        };
                    } else {
                        self.err("Unclosed char constant");
                    }
                }
                //State 10: strings
                10 => {
                    //inside string
                    if ch == '\\' {
                        // Jump over the escape slash AND the escaped character (like \")
                        self.consume();
                        self.consume();
                    } else if ch == '"' {
                        self.consume();
                        let text: String = self.input[start_pos + 1..self.pos - 1].iter().collect();
                        return Token {
                            code: TokenCode::CtString(text),
                            line: self.line,
                        };
                    } else if ch == '\0' {
                        self.err("Unclosed string literal");
                    } else {
                        self.consume();
                    }
                }
                //State 11
                11 => {
                    if ch == '\n' || ch == '\t' || ch == '\r' {
                        self.consume();
                    } else {
                        state = 0;
                    }
                }
                //State 12-13: comment or div
                12 => {
                    if ch == '/' {
                        self.consume();
                        state = 13;
                    } else {
                        return Token {
                            code: TokenCode::DIV,
                            line: self.line,
                        };
                    }
                }
                13 => {
                    if ch == '\n' || ch == '\0' || ch == '\r' {
                        state = 0;
                    } else {
                        self.consume();
                    }
                }
                //State 14-19: Logical & relational opperators
                14 => {
                    // !
                    if ch == '=' {
                        self.consume();
                        return Token {
                            code: TokenCode::NOTEQ,
                            line: self.line,
                        };
                    } else {
                        return Token {
                            code: TokenCode::NOT,
                            line: self.line,
                        };
                    }
                }
                15 => {
                    // =
                    if ch == '=' {
                        self.consume();
                        return Token {
                            code: TokenCode::EQUAL,
                            line: self.line,
                        };
                    } else {
                        return Token {
                            code: TokenCode::ASSIGN,
                            line: self.line,
                        };
                    }
                }
                16 => {
                    // <
                    if ch == '=' {
                        self.consume();
                        return Token {
                            code: TokenCode::LESSEQ,
                            line: self.line,
                        };
                    } else {
                        return Token {
                            code: TokenCode::LESS,
                            line: self.line,
                        };
                    }
                }
                17 => {
                    // >
                    if ch == '=' {
                        self.consume();
                        return Token {
                            code: TokenCode::GREATEREQ,
                            line: self.line,
                        };
                    } else {
                        return Token {
                            code: TokenCode::GREATER,
                            line: self.line,
                        };
                    }
                }
                18 => {
                    // &
                    if ch == '&' {
                        self.consume();
                        return Token {
                            code: TokenCode::AND,
                            line: self.line,
                        };
                    } else {
                        self.err("Expected '&' after '&'");
                    }
                }
                19 => {
                    // |
                    if ch == '|' {
                        self.consume();
                        return Token {
                            code: TokenCode::OR,
                            line: self.line,
                        };
                    } else {
                        self.err("Expected '|' after '|'");
                    }
                }

                _ => self.err(&format!("Unhandled state: {}", state)),
            }
        }
    }
}
