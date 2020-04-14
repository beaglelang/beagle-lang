use crate::lexer::tokens::{LexerToken, TokenData, TokenType};

use core::ansi::{Bg, Fg};

use std::sync::mpsc::{Receiver, Sender};

use ir::{
    hir::{ChannelIr, Instruction},
    type_signature::TypeSignature,
};

use core::pos::BiPos as Position;

use std::sync::{Arc, Mutex};

use notices::{Notice, NoticeLevel};

pub mod functions;
pub mod rules;

const PREV_TOKEN: usize = 0;
const CURRENT_TOKEN: usize = 1;
const NEXT_TOKEN: usize = 2;

#[derive(Debug, PartialEq)]
pub enum ParseContext{
    TopLevel,
    Local
}

pub struct Parser<'a> {
    pub name: String,
    pub ir_tx: Arc<Mutex<Sender<Option<ChannelIr>>>>,
    pub token_rx: Receiver<LexerToken<'a>>,
    pub notice_tx: Sender<Option<Notice>>,
    pub context: ParseContext,

    active_tokens: [LexerToken<'a>; 3],
}

impl<'a> Parser<'a> {
    pub fn new(
        name: String,
        ir_tx: Sender<Option<ChannelIr>>,
        token_rx: Receiver<LexerToken<'a>>,
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
    pub fn current_token(&self) -> &LexerToken<'a> {
        &self.active_tokens[CURRENT_TOKEN]
    }

    #[inline]
    pub fn next_token(&self) -> &LexerToken<'a> {
        &self.active_tokens[NEXT_TOKEN]
    }

    #[inline]
    pub fn prev_token(&self) -> &LexerToken<'a> {
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
            if let Err(e) = self.ir_tx.lock().unwrap().send(Some(ChannelIr {
                pos,
                sig: TypeSignature::None,
                ins: Instruction::Halt,
            })) {
                eprintln!(
                    "{}Parser notice send error: {}{}",
                    core::ansi::Fg::BrightRed,
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
    pub fn emit_ir(&mut self, pos: Position, sig: TypeSignature, ins: Instruction) {
        let ir = ChannelIr { pos, sig, ins };
        self.ir_tx
            .lock()
            .expect("Failed to acquire lock on ir_tx sender.")
            .send(Some(ir))
            .expect(format!("Failed to send IR through IR channel.").as_str())
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

    pub async fn parse<'p>(
        name: String,
        ir_tx: Sender<Option<ChannelIr>>,
        token_rx: Receiver<LexerToken<'p>>,
        notice_tx: Sender<Option<Notice>>,
    ) -> Result<(), String> {
        let mut parser = Parser::new(name, ir_tx, token_rx, notice_tx);
        &parser.advance().unwrap();
        &parser.advance().unwrap();
        match functions::module(&mut parser) {
            Ok(()) => {
                parser.emit_notice(Position::default(), NoticeLevel::Halt, "Halt".to_string());
                return Ok(());
            }
            Err(_) => {
                parser.emit_notice(Position::default(), NoticeLevel::Halt, "Halt".to_string());
                return Err("An error occurred while parsing module".to_string());
            }
        }
    }
}
