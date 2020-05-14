use core::pos::BiPos;
use lazy_static::lazy_static;
use std::collections::{
    HashMap,
};
use std::io::Result;
use std::sync::{mpsc::Sender, Arc, Mutex};
use futures::{
  executor::ThreadPool,
};

use notices::*;

pub mod tokens;
use tokens::{
    LexerToken,
};

lazy_static! {
    static ref IDENT_MAP: HashMap<&'static str, tokens::TokenType> = {
        let mut m = HashMap::new();
        m.insert("let", tokens::TokenType::KwLet);
        m.insert("val", tokens::TokenType::KwVal);
        m.insert("var", tokens::TokenType::KwVar);
        m.insert("mut", tokens::TokenType::KwMut);
        m.insert("native", tokens::TokenType::KwNative);
        m.insert("fun", tokens::TokenType::KwFun);
        m.insert("None", tokens::TokenType::KwNone);
        m
    };
}

pub struct LexerManager{
    thread_pool: ThreadPool,
    notice_tx: Arc<Mutex<Sender<Option<Notice>>>>,
}

impl LexerManager{
    pub fn new(notice_tx: Sender<Option<Notice>>) -> Self{
        LexerManager{
            thread_pool: ThreadPool::new().unwrap(),
            notice_tx: Arc::new(Mutex::new(notice_tx))
        }
    }

    pub fn enqueue_module(&self, module_name: String, input: String, parser_tx: Sender<LexerToken>){
        let notice_tx_clone = self.notice_tx.clone();
        let parser_tx_clone = parser_tx.clone();
        let input_clone = input.clone();
        self.thread_pool.spawn_ok(async move{
            let lexer_result = Lexer::new(input_clone, Arc::new(Mutex::new(parser_tx_clone)));
            let mut lexer = if let Ok(l) = lexer_result{
                l
            }else{
                let notice = Notice{
                    from: "Lexer".to_string(),
                    file: module_name.clone().to_string(),
                    level: NoticeLevel::Error,
                    msg: "Failed to start lexer. Unknown reasons. Please report this to author.".to_string(),
                    pos: BiPos::default()
                };
                notice_tx_clone.lock().expect("Failed to acquire lock on notice sender.").send(Some(notice)).unwrap();
                return
            };
            let tokenizer_result = lexer.start_tokenizing();
            if let Err(msg) = tokenizer_result{
                let notice = Notice{
                    from: "Lexer".to_string(),
                    file: module_name.clone().to_string(),
                    level: NoticeLevel::Error,
                    msg,
                    pos: BiPos::default()
                };
                notice_tx_clone.lock().expect("Failed to acquire lock on notice sender.").send(Some(notice)).unwrap();
            };
        });
    }
}

pub struct Lexer {
    input: String,
    source: Option<String>,
    char_idx: usize,
    current_pos: BiPos,

    pub token_sender: Arc<Mutex<Sender<tokens::LexerToken>>>,
}

impl<'a, 'b> Lexer{
    pub fn new(
        input: String,
        token_tx: Arc<Mutex<Sender<LexerToken>>>,
    ) -> Result<Box<Lexer>> {
        let input_str = input.clone();
        let lexer = Box::new(Lexer {
            input: input_str.clone(),
            source: Some(input_str),
            char_idx: 0,
            current_pos: BiPos::default(),
            token_sender: token_tx,
        });
        Ok(lexer)
    }

    fn advance_end(&mut self) -> Option<char> {
        match &self.source{
            Some(src) => {
                let mut chars = src.chars();
                let new_c = chars.next();
                self.char_idx += 1;
                self.source = Some(chars.as_str().to_string());
                match new_c {
                    Some(c) if c.is_whitespace() => {
                        self.current_pos.next_col_end();
                        if c == '\n' || c == '\r' {
                            return new_c;
                        }
                    }
                    Some(_) => {
                        self.current_pos.next_col_end();
                    }
                    _ => return new_c,
                }
                new_c
            },
            None => None
        }
    }

    fn peek(&mut self) -> Option<char>{
        match &self.source{
            Some(src) => src.chars().next(),
            None => None
        }
    }

    fn advance(&mut self) -> Option<char> {
        match &self.source{
            Some(src) => {
                let mut chars = src.chars();
                let new_c = chars.next();
                self.char_idx += 1;
                self.source = Some(chars.as_str().to_string());
                match new_c {
                    Some(c) if c.is_whitespace() => {
                        self.current_pos.next_col();
                        if c == '\n' || c == '\r' {
                            return new_c;
                        }
                    }
                    Some(_) => {
                        self.current_pos.next_col();
                    }
                    _ => return new_c,
                }
                new_c
            },
            None => None,
        }
    }

    fn is_delimiter(&self, c: char) -> Option<tokens::TokenType> {
        match c {
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
            _ => None,
        }
    }

    fn is_keyword(&self, identifier: &str) -> tokens::TokenType {
        match IDENT_MAP.get(identifier) {
            Some(token_type) => *token_type,
            None => tokens::TokenType::Identifier,
        }
    }

    fn number(&mut self) -> Option<tokens::LexerToken> {
        let start_idx = self.char_idx;
        let mut is_float = false;
        while let Some(c) = self.peek() {
            if c == '.' {
                is_float = true;
                self.advance_end();
            } else if c.is_digit(10) {
                self.advance_end();
            } else {
                break;
            }
        }
        let slice = match self.input.get(start_idx - 1..self.char_idx) {
            Some(s) => s,
            None => {
                return Some(tokens::LexerToken {
                    type_: tokens::TokenType::Err,
                    data: tokens::TokenData::String("Failed to index into source".to_string()),
                    pos: self.current_pos,
                })
            }
        };
        let num_str = String::from(slice);
        let number = num_str.trim();
        // println!("number slice: {}", number);
        Some(tokens::LexerToken {
            type_: tokens::TokenType::Number,
            data: if is_float {
                tokens::TokenData::Float(match number.parse::<f32>() {
                    Ok(f) => f,
                    Err(e) => {
                        return Some(tokens::LexerToken {
                            type_: tokens::TokenType::Err,
                            data: tokens::TokenData::String(format!(
                                "Failed to parse float from source: {}",
                                e
                            )),
                            pos: self.current_pos,
                        })
                    }
                })
            } else {
                tokens::TokenData::Integer(match number.parse::<i32>() {
                    Ok(f) => f,
                    Err(e) => {
                        return Some(tokens::LexerToken {
                            type_: tokens::TokenType::Err,
                            data: tokens::TokenData::String(format!(
                                "Failed to parse integer from source: {}",
                                e
                            )),
                            pos: self.current_pos,
                        })
                    }
                })
            },
            pos: self.current_pos,
        })
    }

    #[inline]
    fn string(&mut self) -> Option<tokens::LexerToken> {
        let start_idx = self.char_idx;
        self.advance().unwrap();
        while let Some(c) = self.advance_end() {
            if c == '\"' {
                break;
            }
        }

        let slice = match self.input.get(start_idx..self.char_idx) {
            Some(s) => s,
            None => {
                return Some(tokens::LexerToken {
                    type_: tokens::TokenType::Err,
                    data: tokens::TokenData::String("Failed to extract string from input source.".to_string()),
                    pos: self.current_pos,
                })
            }
        };

        Some(tokens::LexerToken {
            type_: tokens::TokenType::String,
            data: tokens::TokenData::String(slice.to_owned()),
            pos: self.current_pos,
        })
    }

    fn get_token(&mut self) -> Option<tokens::LexerToken> {
        // println!("Processing char: {:?}", self.peek());
        match self.peek() {
            Some(c) => {
                match c {
                    c if c.is_whitespace() => {
                        if c == '\n'{
                            self.current_pos.next_line();
                        }
                        return None
                    }
                    c if c.is_alphabetic() => {
                        let start = self.char_idx;
                        let end = loop {
                            match self.peek() {
                                Some(c) if self.is_delimiter(c).is_some() || c.is_whitespace() => break self.char_idx,
                                Some(_) =>{
                                    self.advance_end();
                                    continue
                                },
                                None => break self.input.len(),
                            }
                        };
                        let identifier = &self.input[start..end];
                        let type_ = self.is_keyword(identifier);
                        return Some(tokens::LexerToken {
                            type_,
                            data: tokens::TokenData::String(identifier.to_string()),
                            pos: self.current_pos,
                        });
                    }
                    '\"' => {
                        return match self.string() {
                            Some(t) => {
                                Some(t)
                            },
                            None => {
                                return Some(tokens::LexerToken {
                                    data: tokens::TokenData::String(format!(
                                        "Unable to get string from input @ {:?}",
                                        self.current_pos
                                    )),
                                    type_: tokens::TokenType::Err,
                                    pos: self.current_pos,
                                })
                            }
                        }
                    }
                    c if c.is_digit(10) => {
                        let number = self.number();
                        return number;
                    },
                    c if self.is_delimiter(c).is_some() => {
                        // println!("Found a delimiter: {}", c.clone());
                        let token = tokens::LexerToken {
                            data: tokens::TokenData::String(c.to_string()),
                            type_: self.is_delimiter(c).unwrap(),
                            pos: self.current_pos,
                        };
                        self.advance();
                        return Some(token);
                    }
                    _ => {
                        return Some(tokens::LexerToken {
                            type_: tokens::TokenType::Err,
                            data: tokens::TokenData::String(format!("Invalid character")),
                            pos: self.current_pos,
                        })
                    }
                }
                
            },
            None => Some(tokens::LexerToken{
                type_: tokens::TokenType::Eof, 
                data: tokens::TokenData::None, 
                pos: self.current_pos
            })
        }
    }

    pub fn start_tokenizing(&mut self) -> std::result::Result<(), String> {
        let token_sender_clone = self.token_sender.clone();
        let guard = token_sender_clone.lock().unwrap();
        loop {
            let token = self.get_token();
            match token {
                Some(t) => {
                    // self.advance();
                    // println!("{}", t.clone());
                    match guard.send(t.clone()){
                        Ok(()) => {
                            match &t.type_ {
                                tokens::TokenType::Eof => {
                                    break;
                                }
                                tokens::TokenType::Err => {
                                    return Err(format!(
                                        "An error occurred while tokenizing input: {:?}",
                                        t
                                    )
                                    .to_string())
                                }
                                _ => continue,
                            };
                        }
                        Err(_) => return Ok(())
                    }
                }
                None => {
                    self.advance();
                    continue;
                }
            }
        }
        Ok(())
    }
}
