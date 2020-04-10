#[derive(Debug, Copy, Clone)]
pub enum TokenType {
    LParen,
    RParen,
    LCurly,
    RCurly,
    LBracket,
    RBracket,

    Comma,
    Dot,
    Semicolon,

    Plus,
    Minus,
    Star,
    Slash,

    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Colon,
    RArrow,

    Identifier,
    String,
    Number,

    KwVal,
    KwVar,

    KwFun,

    KwLet,
    KwMut,

    KwSelf,
    KwStruct,
    KwReturn,
    KwModule,
    KwNative,
    KwPub,

    KwIf,
    KwElse,
    KwLoop,
    KwWhile,
    KwFor,
    KwBreak,
    KwContinue,

    KwTrue,
    KwFalse,
    KwNil,

    KwAs,

    Err,
    Eof,
}

#[derive(Debug, Clone)]
pub enum TokenData<'a> {
    None,
    Integer(isize),
    Float(f64),
    Str(&'a str),
    String(String),
}

#[derive(Debug, Clone)]
pub struct LexerToken<'a> {
    pub type_: TokenType,
    pub data: TokenData<'a>,
    pub pos: super::LexerPos,
}

impl<'a> Default for LexerToken<'a> {
    fn default() -> Self {
        LexerToken {
            type_: TokenType::Eof,
            data: TokenData::None,
            pos: super::LexerPos::default(),
        }
    }
}

impl<'a> std::fmt::Display for LexerToken<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?},\n {:?},\n {}", self.type_, self.data, self.pos)
    }
}
