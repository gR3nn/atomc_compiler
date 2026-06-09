#[derive(Debug, Clone, PartialEq)]
pub enum TokenCode {
    //identifiers and constants
    ID(String),
    CtInt(i64),
    CtReal(f64),
    CtChar(char),
    CtString(String),

    //keywords
    BREAK,
    CHAR,
    DOUBLE,
    ELSE,
    FOR,
    IF,
    INT,
    RETURN,
    STRUCT,
    VOID,
    WHILE,

    //operators and delimiters
    ADD,
    SUB,
    MUL,
    DIV,
    DOT,
    AND,
    OR,
    NOT,
    ASSIGN,
    EQUAL,
    NOTEQ,
    LESS,
    LESSEQ,
    GREATER,
    GREATEREQ,
    COMMA,
    SEMICOLON,
    LPAR,
    RPAR,
    LBRACKET,
    RBRACKET,
    LACC,
    RACC,

    END, //end of input
}

#[derive(Debug, Clone)]
pub struct Token {
    pub code: TokenCode,
    pub line: usize,
}
