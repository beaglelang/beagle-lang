use lazy_static::lazy_static;
use std::collections::HashMap;
use std::io::Result;
use std::sync::{mpsc, Mutex};

mod tokens;

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
pub struct Position(usize, usize);
impl Default for Position {
    fn default() -> Self {
        Self(1, 0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LexerPos {
    start: Position,
    end: Position,
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
        self.start.1 = 1;
        self.end = self.start;
    }

    fn next_col(&mut self) {
        self.start.1 += 1;
        self.end = self.start;
    }

    fn next_line_end(&mut self) {
        self.end.0 += 1;
        self.end.1 = 1;
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

    pub token_sender: Mutex<mpsc::Sender<tokens::LexerToken<'a>>>,
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
            token_sender: Mutex::new(tx),
        });
        Ok((lexer, rx))
    }

    fn advance_end(&mut self) -> Option<(usize, char)> {
        self.current_char = self.chars.next();
        match self.current_char {
            Some((_, c)) if c == '\n' => return self.current_char,
            Some(_) => self.current_pos.next_col_end(),
            _ => self.current_pos.next_col_end(),
        }
        self.current_char
    }

    fn advance(&mut self) -> Option<(usize, char)> {
        self.current_char = self.chars.next();
        match self.current_char {
            Some((_, c)) if c == '\n' => {
                self.current_pos.next_line();
                self.chars.next();
            }
            Some(_) => self.current_pos.next_col(),
            _ => self.current_pos.next_col(),
        }
        self.current_char
    }

    fn get_token(&mut self) -> tokens::LexerToken<'a> {
        let (i, c) = match self.current_char {
            Some(ic) => ic,
            _ => return tokens::LexerToken::default(),
        };

        println!("Getting token for {}", c);

        match c {
            c if c.is_alphabetic() => {
                let start = i;
                let end = loop {
                    match self.advance_end() {
                        Some((i, c)) if !c.is_alphanumeric() => break i,
                        Some(_) => continue,
                        _ => break start,
                    }
                };
                let identifier = &self.input[start..end];
                println!("Found identifier {}", identifier);
                tokens::LexerToken {
                    type_: tokens::TokenType::Identifier,
                    data: tokens::TokenData::Str(identifier),
                    pos: self.current_pos,
                }
            }
            _ => tokens::LexerToken {
                type_: tokens::TokenType::Err,
                data: tokens::TokenData::Str("Invalid character"),
                pos: self.current_pos,
            },
        }
    }

    pub fn start_tokenizing(&mut self) -> Result<()> {
        println!("tokenizing file");
        loop {
            let token = self.get_token();
            self.token_sender
                .lock()
                .expect("Unable to acquire lock to token sender")
                .send(token.clone())
                .expect("Failed to send token to token receiver.");
            match &token.type_ {
                tokens::TokenType::Eof => break,
                _ => self.advance(),
            };
        }
        Ok(())
    }
}
