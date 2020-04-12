use crate::lexer::tokens::{LexerToken, TokenData, TokenType};

use core::ansi::{Bg, Fg};

use std::sync::mpsc::{Receiver, Sender};

use ir::{
    hir::{ChannelIr, Instruction},
    type_signature::TypeSignature,
    Position,
};

pub mod functions;
pub mod rules;

const PREV_TOKEN: usize = 0;
const CURRENT_TOKEN: usize = 1;
const NEXT_TOKEN: usize = 2;

pub struct Parser<'a> {
    pub(crate) name: String,
    pub(crate) ir_tx: Sender<Option<ChannelIr>>,
    pub(crate) token_rx: Receiver<LexerToken<'a>>,

    active_tokens: [LexerToken<'a>; 3],
}

impl<'a> Parser<'a> {
    pub(crate) fn new(
        name: String,
        ir_tx: Sender<Option<ChannelIr>>,
        token_rx: Receiver<LexerToken<'a>>,
    ) -> Self {
        Parser {
            name,
            ir_tx,
            token_rx,
            active_tokens: [
                LexerToken::default(),
                LexerToken::default(),
                LexerToken::default(),
            ],
        }
    }

    #[inline]
    pub(crate) fn prev_token(&mut self) -> &LexerToken<'a> {
        &self.active_tokens[PREV_TOKEN]
    }

    #[inline]
    pub(crate) fn current_token(&mut self) -> &LexerToken<'a> {
        &self.active_tokens[CURRENT_TOKEN]
    }

    #[inline]
    pub(crate) fn next_token(&mut self) -> &LexerToken<'a> {
        &self.active_tokens[NEXT_TOKEN]
    }

    #[inline]
    pub(crate) fn advance(&mut self) -> Result<(), String> {
        self.active_tokens[PREV_TOKEN] = self.active_tokens[CURRENT_TOKEN].clone();
        self.active_tokens[CURRENT_TOKEN] = self.active_tokens[NEXT_TOKEN].clone();
        match self.token_rx.recv() {
            Ok(token) => self.active_tokens[NEXT_TOKEN] = token,
            Err(m) => return Err(m.to_string()),
        }

        Ok(())
    }

    #[inline]
    pub(crate) fn check(&mut self, type_: TokenType) -> bool {
        self.current_token().type_ == type_
    }

    pub(crate) fn emit_ir(&mut self, pos: Position, sig: TypeSignature, ins: Instruction) {
        let ir = ChannelIr { pos, sig, ins };
        self.ir_tx
            .send(Some(ir))
            .expect(format!("Failed to send IR through IR channel.").as_str())
    }

    #[inline]
    pub(crate) fn check_consume(&mut self, type_: TokenType) -> bool {
        if self.check(type_) {
            self.advance().unwrap();
            true
        } else {
            false
        }
    }

    #[inline]
    pub(crate) fn consume(
        &mut self,
        type_: TokenType,
        error_message: &'static str,
    ) -> Result<&TokenData, ()> {
        if self.check(type_) {
            self.advance().unwrap();
            Ok(&self.current_token().data)
        } else {
            panic!(error_message);
        }
    }

    pub async fn parse(&mut self) -> Result<(), String> {
        self.advance().unwrap();
        self.advance().unwrap();
        functions::module(self)
    }
}
