mod statements;
mod properties;
mod modules;
mod functions;
mod expressions;
mod type_;
mod local_statements;



use lexer::tokens::{LexerToken, TokenData, TokenType};

use std::sync::mpsc::{Receiver, Sender};

use ir::{
    hir::{HIRInstruction},
    Chunk,
};

use core::{
    pos::BiPos as Position 
};

use ir_traits::{
    WriteInstruction,
};

use std::sync::{Arc, Mutex};

use notices::{Notice, NoticeLevel};

use futures::executor::ThreadPool;

pub trait ParseRule{
    fn parse(p: &mut Parser) -> Result<(), ()>;
}

pub trait TryParse{
    fn try_parse(p: &mut Parser) -> Result<Chunk, ParseError>;
}

pub struct ParseError{
    cause: Option<Box<ParseError>>,
    msg: String,
    pos: Position,
}

impl ParseError{
    fn emit_notice(&self, parser: &Parser){
        if let Some(cause) = &self.cause{
            cause.emit_notice(parser);
        }
        parser.emit_notice(self.pos, NoticeLevel::Error, self.msg.clone());
    }
}

const PREV_TOKEN: usize = 0;
const CURRENT_TOKEN: usize = 1;
const NEXT_TOKEN: usize = 2;

#[derive(Debug, PartialEq)]
pub enum ParseContext{
    TopLevel,
    Local
}

pub struct ParseManager{
    thread_pool: ThreadPool,
    notice_tx: Arc<Mutex<Sender<Option<Notice>>>>,
}

impl ParseManager{
    pub fn new(notice_tx: Sender<Option<Notice>>) -> Self{
        ParseManager{
            thread_pool: ThreadPool::new().unwrap(),
            notice_tx: Arc::new(Mutex::new(notice_tx)),
        }
    }

    pub fn enqueue_module(&self, module_name: String, token_rx: Receiver<LexerToken>, hir_tx: Sender<Option<Chunk>>){
        let notice_tx_clone = self.notice_tx.lock().unwrap().clone();
        self.thread_pool.spawn_ok(async move{
            let parser = Parser::parse(module_name.clone(), hir_tx, token_rx, notice_tx_clone.clone());
            if let Err(msg) = parser{
                let notice = Notice{
                    from: "Parser".to_string(),
                    file: module_name.clone(),
                    level: NoticeLevel::ErrorPrint,
                    msg,
                    pos: Position::default()
                };
                notice_tx_clone.clone().send(Some(notice)).unwrap();
            };
        });
    }
}

pub struct Parser {
    pub name: String,
    pub ir_tx: Arc<Mutex<Sender<Option<Chunk>>>>,
    pub token_rx: Receiver<LexerToken>,
    pub notice_tx: Sender<Option<Notice>>,
    pub context: ParseContext,

    active_tokens: [LexerToken; 3],
}

impl Parser {
    pub fn new(
        name: String,
        ir_tx: Sender<Option<Chunk>>,
        token_rx: Receiver<LexerToken>,
        notice_tx: Sender<Option<Notice>>,
    ) -> Self {
        Parser {
            name,
            ir_tx: Arc::new(Mutex::new(ir_tx)),
            token_rx,
            notice_tx,
            context: ParseContext::TopLevel,
            active_tokens: [
                LexerToken::default(),
                LexerToken::default(),
                LexerToken::default(),
            ],
        }
    }

    #[inline]
    pub fn current_token(&self) -> &LexerToken {
        &self.active_tokens[CURRENT_TOKEN]
    }

    #[inline]
    pub fn next_token(&self) -> &LexerToken {
        &self.active_tokens[NEXT_TOKEN]
    }

    #[inline]
    pub fn prev_token(&self) -> &LexerToken {
        &self.active_tokens[PREV_TOKEN]
    }

    #[inline]
    pub fn advance(&mut self) -> Result<(), String> {
        self.active_tokens[PREV_TOKEN] = self.active_tokens[CURRENT_TOKEN].clone();
        self.active_tokens[CURRENT_TOKEN] = self.active_tokens[NEXT_TOKEN].clone();
        self.active_tokens[NEXT_TOKEN] = match self
            .token_rx
            .recv_timeout(std::time::Duration::from_secs(1))
        {
            Err(_) => LexerToken::default(),
            Ok(t) => t,
        };

        Ok(())
    }

    pub fn emit_notice(&self, pos: Position, level: NoticeLevel, msg: String) {
        if level == NoticeLevel::Error {
            let mut halt_chunk = Chunk::new();
            halt_chunk.write_instruction(HIRInstruction::Halt);
            if let Err(e) = self.ir_tx.lock().unwrap().send(Some(halt_chunk)) {
                println!(
                    "{}Parser notice send error: {}{}",
                    core::ansi::Fg::Red,
                    e,
                    core::ansi::Fg::Reset
                );
            }
        }

        let notice = Notice {
            from: "Parser".to_string(),
            msg,
            pos,
            file: self.name.clone(),
            level,
        };

        if let Err(e) = self.notice_tx.send(Some(notice)) {
            eprintln!(
                "{}Parser notice send error: {}{}",
                core::ansi::Fg::BrightRed,
                e,
                core::ansi::Fg::Reset
            );
        }
    }

    #[inline]
    pub fn check(&mut self, type_: TokenType) -> bool {
        self.current_token().type_ == type_
    }

    #[inline]
    pub fn check_next(&mut self, type_: TokenType) -> bool {
        self.next_token().type_ == type_
    }

    #[inline]
    pub fn emit_ir_whole(&mut self, hir: Chunk){
        match self.ir_tx
            .lock()
            .expect("Failed to acquire lock on ir_tx sender.")
            .send(Some(hir)){
                Ok(()) => return,
                Err(_) => return,
            }
            
    }

    #[inline]
    pub fn check_consume(&mut self, type_: TokenType) -> bool {
        if self.check(type_) {
            self.advance().unwrap();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn check_consume_next(&mut self, type_: TokenType) -> bool {
        if self.check_next(type_) {
            self.advance().unwrap();
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn consume(
        &mut self,
        type_: TokenType
    ) -> Result<&TokenData, ()> {
        self.advance().unwrap();
        if self.check(type_) {
            Ok(&self.current_token().data)
        } else {
            self.emit_notice(
                self.current_token().pos,
                NoticeLevel::Error,
                format!("Expected a '{:?}' but instead got {:?}", type_, self.current_token().type_).to_string(),
            );
            return Err(());
        }
    }

    pub fn parse(
        name: String,
        ir_tx: Sender<Option<Chunk>>,
        token_rx: Receiver<LexerToken>,
        notice_tx: Sender<Option<Notice>>,
    ) -> Result<(), String> {
        let mut parser = Parser::new(name, ir_tx, token_rx, notice_tx);
        parser.advance().unwrap();
        parser.advance().unwrap();
        while !parser.check(TokenType::Eof) {
            if let Err(()) = statements::StatementParser::parse(&mut parser) {
                if parser.ir_tx.lock().unwrap().send(None).is_err(){
                    return Err("An error occurred while parsing module".to_string())
                }
                return Err("An error occurred while parsing module".to_string());
            }
        }
        if parser.ir_tx.lock().unwrap().send(None).is_err(){
            return Ok(())
        }
        Ok(())
    }
}

