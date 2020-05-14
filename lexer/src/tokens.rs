use core::pos::BiPos;

#[repr(u8)]
#[derive(Debug, PartialEq, Clone, Copy, Hash, Eq)]
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
    Decimal,

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
pub enum TokenData {
    None,
    Integer(i32),
    Float(f32),
    String(String),
}

#[derive(Debug, Clone)]
pub struct LexerToken{
    pub type_: TokenType,
    pub data: TokenData,
    pub pos: BiPos,
}

impl Default for LexerToken {
    fn default() -> Self {
        LexerToken {
            type_: TokenType::Eof,
            data: TokenData::None,
            pos: BiPos::default(),
        }
    }
}

impl std::fmt::Display for LexerToken{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?},\n\t{:?},\n\t{}", self.type_, self.data, self.pos)
    }
}
