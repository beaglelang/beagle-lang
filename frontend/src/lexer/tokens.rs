#[derive(Debug, Copy, Clone)]
pub enum TokenType {
    LParen,
    RParen,
    LCurly,
    RCurly,
    LBracket,
    RBracket,
    QMark,
    Hyphen,

    Comma,
    Dot,
    Semicolon,
    Pipe,

    Plus,
    Minus,
    Star,
    Slash,
    Backslash,
    Underscore,
    And,
    Caret,
    Tick,

    Bang,
    Equal,

    Colon,
    Apost,
    Quote,
    RAngle,
    LAngle,
    Percent,
    Dollar,
    Hash,
    At,

    Identifier,
    String,
    Number,

    KwVal,
    KwVar,

    KwFun,

    KwLet,
    KwMut,

    KwStruct,
    KwReturn,
    KwMod,
    KwNative,
    KwPublic,

    KwIf,
    KwElse,
    KwLoop,
    KwWhile,
    KwFor,
    KwBreak,
    KwContinue,

    KwTrue,
    KwFalse,
    KwNull,
    KwNone,

    KwAs,
    KwWith,

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
        write!(f, "{:?},\n\t{:?},\n\t{}", self.type_, self.data, self.pos)
    }
}
