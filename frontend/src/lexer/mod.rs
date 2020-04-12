use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Result;
use std::sync::{mpsc, Arc, Mutex};

pub mod tokens;

lazy_static! {
    static ref IDENT_MAP: HashMap<&'static str, tokens::TokenType> = {
        let mut m = HashMap::new();
        m.insert("let", tokens::TokenType::KwLet);
        m.insert("val", tokens::TokenType::KwVal);
        m.insert("var", tokens::TokenType::KwVar);
        m.insert("mut", tokens::TokenType::KwMut);
        m.insert("native", tokens::TokenType::KwNative);
        m.insert("fun", tokens::TokenType::KwFun);
        m
    };
}

#[derive(Debug, Clone, Copy)]
pub struct Position(pub usize, pub usize);
impl Default for Position {
    fn default() -> Self {
        Self(1, 1)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LexerPos {
    pub start: Position,
    pub end: Position,
}

impl std::fmt::Display for LexerPos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "start: line {}, col {}\n\tend: line {} col {}",
            self.start.0, self.start.1, self.end.0, self.end.1
        )
    }
}

impl Default for LexerPos {
    fn default() -> Self {
        Self {
            start: Position::default(),
            end: Position::default(),
        }
    }
}

impl LexerPos {
    fn next_line(&mut self) {
        self.start.0 += 1;
        self.start.1 = 0;
        self.end = self.start;
    }

    fn next_col(&mut self) {
        self.start = self.end;
        self.start.1 += 1;
        self.end.1 += 1;
    }

    fn next_col_end(&mut self) {
        self.end.1 += 1;
    }
}

pub struct Lexer<'a> {
    input: &'a str,
    chars: std::str::CharIndices<'a>,
    current_char: Option<(usize, char)>,
    current_pos: LexerPos,

    pub token_sender: Arc<Mutex<mpsc::Sender<tokens::LexerToken<'a>>>>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Result<(Box<Self>, mpsc::Receiver<tokens::LexerToken<'a>>)> {
        let (tx, rx) = mpsc::channel();
        let mut chars = input.char_indices();
        let current_char = chars.next();
        let lexer = Box::new(Lexer {
            input,
            chars,
            current_char,
            current_pos: LexerPos::default(),
            token_sender: Arc::new(Mutex::new(tx)),
        });
        Ok((lexer, rx))
    }

    fn advance_end(&mut self) -> Option<(usize, char)> {
        let prev_char = self.current_char;
        self.current_char = self.chars.next();
        // println!("advance_end: prev_char {:?}", prev_char);
        // println!("advance_end: current_char {:?}", self.current_char);
        match self.current_char {
            Some((_, c)) if c.is_whitespace() => {
                self.current_pos.next_col_end();
                if c == '\n' || c == '\r'{
                    return self.current_char;
                }
            }
            Some(_) => {
                self.current_pos.next_col_end();
            }
            _ => return prev_char
        }
        self.current_char
    }

    fn advance(&mut self) -> Option<(usize, char)>{
        let prev_char = self.current_char;
        self.current_char = self.chars.next();
        // println!("advance: current_char {:?}", self.current_char);
        match self.current_char {
            Some(_) => {
                self.current_pos.next_col();
            }
            _ => return prev_char
        }
        self.current_char
    }

    fn is_delimiter(&self, c: char) -> Option<tokens::TokenType>{
        match c{
            '=' => Some(tokens::TokenType::Equal),
            '(' => Some(tokens::TokenType::LParen),
            ')' => Some(tokens::TokenType::RParen),
            ']' => Some(tokens::TokenType::RBracket),
            '[' => Some(tokens::TokenType::LBracket),
            '{' => Some(tokens::TokenType::LCurly),
            '}' => Some(tokens::TokenType::RCurly),
            '|' => Some(tokens::TokenType::Pipe),
            '/' => Some(tokens::TokenType::Slash),
            '?' => Some(tokens::TokenType::QMark),
            '\\' => Some(tokens::TokenType::Backslash),
            ';' => Some(tokens::TokenType::Semicolon),
            ':' => Some(tokens::TokenType::Colon),
            '\'' => Some(tokens::TokenType::Apost),
            '"' => Some(tokens::TokenType::Quote),
            '>' => Some(tokens::TokenType::RAngle),
            '<' => Some(tokens::TokenType::LAngle),
            '.' => Some(tokens::TokenType::Dot),
            ',' => Some(tokens::TokenType::Comma),
            '-' => Some(tokens::TokenType::Minus),
            '+' => Some(tokens::TokenType::Plus),
            '_' => Some(tokens::TokenType::Underscore),
            '*' => Some(tokens::TokenType::Star),
            '%' => Some(tokens::TokenType::Percent),
            '$' => Some(tokens::TokenType::Dollar),
            '#' => Some(tokens::TokenType::Hash),
            '@' => Some(tokens::TokenType::At),
            '!' => Some(tokens::TokenType::Bang),
            '&' => Some(tokens::TokenType::And),
            '^' => Some(tokens::TokenType::Caret),
            '`' => Some(tokens::TokenType::Tick),
            _ => None
        }
    }

    fn is_keyword(&self, identifier: &str) -> tokens::TokenType{
        match IDENT_MAP.get(identifier) {
            Some(token_type) => *token_type,
            None => tokens::TokenType::Identifier
        }
    }

    fn number(&mut self) -> Option<tokens::LexerToken<'a>>{
        let start_idx = self.current_char?.0;
        let mut is_float = false;
        while let Some((_, c)) = self.advance_end(){
            if c == '.' {
                is_float = true;
                self.advance_end();
            }else if c.is_digit(10){
                self.advance_end();
            }else{
                break;
            }
        };
        let slice = match self.input.get(start_idx - 1..self.current_char?.0){
            Some(s) => s,
            None => {
                return Some(tokens::LexerToken{
                    type_: tokens::TokenType::Err,
                    data: tokens::TokenData::String("Failed to index into source".to_string()),
                    pos: self.current_pos
                })
            }
        };
        let num_str = String::from(slice);
        let number = num_str.trim();
        // println!("number slice: {}", number);
        Some(tokens::LexerToken{
            type_: tokens::TokenType::Number,
            data: if is_float{
                tokens::TokenData::Float(match number.parse::<f64>(){
                    Ok(f) => f,
                    Err(e) => {
                        return Some(tokens::LexerToken{
                            type_: tokens::TokenType::Err,
                            data: tokens::TokenData::String(format!("Failed to parse float from source: {}", e)),
                            pos: self.current_pos
                        })
                    }
                })
            }else{
                tokens::TokenData::Integer(match number.parse::<isize>(){
                    Ok(f) => f,
                    Err(e) => {
                        return Some(tokens::LexerToken{
                            type_: tokens::TokenType::Err,
                            data: tokens::TokenData::String(format!("Failed to parse integer from source: {}", e)),
                            pos: self.current_pos
                        })
                    }
                })
            },
            pos: self.current_pos
        })
    }

    #[inline]
    fn string(&mut self) -> Option<tokens::LexerToken<'a>>{
        let start_idx = self.current_char?.0;
        self.advance().unwrap();
        while let Some((_, c)) = self.advance_end(){
            if c != '\"'{
                self.advance_end().expect("Failed to advance the end position.");
            }else{
                break;
            }
        }
        
        let slice = match self.input.get(start_idx+1..self.current_char?.0){
            Some(s) => s,
            None => {
                return Some(tokens::LexerToken{
                    type_: tokens::TokenType::Err,
                    data: tokens::TokenData::Str("Failed to extract string from input source."),
                    pos: self.current_pos
                })
            }
        };
        
        Some(tokens::LexerToken{
            type_: tokens::TokenType::String,
            data: tokens::TokenData::Str(slice),
            pos: self.current_pos
        })

    }

    fn get_token(&mut self) -> Option<tokens::LexerToken<'a>> {
        let (i, c) = match self.current_char {
            Some(ic) => ic,
            _ => return Some(tokens::LexerToken::default()),
        };

        match c {
            c if c.is_whitespace() => {
                if c == '\n'{
                    self.current_pos.next_line();
                }
                self.advance();
                return None;
            }
            c if c.is_alphabetic() => {
                let start = i;
                let end = loop {
                    match self.advance_end() {
                        Some((i, c)) if !c.is_alphanumeric() => {
                            break i
                        },
                        Some(_) => continue,
                        None => break self.input.len(),
                    }
                };
                let identifier = &self.input[start..end];
                let type_ = self.is_keyword(identifier);
                return Some(tokens::LexerToken {
                    type_,
                    data: tokens::TokenData::Str(identifier),
                    pos: self.current_pos,
                })
            },
            '\"' => return match self.string(){
                Some(t) => Some(t),
                None => return Some(tokens::LexerToken{
                    data: tokens::TokenData::String(format!("Unable to get string from input @ {:?}", self.current_pos)),
                    type_: tokens::TokenType::Err,
                    pos: self.current_pos
                })
            },
            c if c.is_digit(10) => {
                return self.number()
            },
            c if self.is_delimiter(c).is_some() => {
                return Some(tokens::LexerToken{
                    data: tokens::TokenData::String(c.to_string()),
                    type_: self.is_delimiter(c).unwrap(),
                    pos: self.current_pos
                });
            },
            _ => return Some(tokens::LexerToken {
                type_: tokens::TokenType::Err,
                data: tokens::TokenData::Str("Invalid character"),
                pos: self.current_pos,
            }),
        };
    }

    pub async fn start_tokenizing(&mut self) -> std::result::Result<(), String> {
        let sender_arc = self.token_sender.clone();
        let guard = sender_arc
        .lock()
        .expect("Unable to get token sender from arc mutex.");
        loop {
            let token = self.get_token();
            match token{
                Some(t) => {
                    guard
                        .send(t.clone())
                        .expect("Failed to send token to token receiver.");
                    match &t.type_ {
                        tokens::TokenType::Eof => {
                            break;
                        }
                        tokens::TokenType::Err => {
                            return Err(format!("En error occurred while tokenizing input: {:?}", t).to_string())
                        }
                        _ => self.advance(),
                    };
                },
                None => continue
            }
            
        }
        Ok(())
    }
}
