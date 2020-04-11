use crate::{
    lexer::tokens::{
        LexerToken
    },
};

use std::sync::mpsc::{
    Receiver, Sender
};

use crate::hir::ChannelIr;

mod rules;

const CURRENT_TOKEN: u8 = 0;
const NEXT_TOKEN: u8 = 1;

pub struct Parser<'a>{
    pub(crate) name: String,
    pub(crate) ir_tx: Sender<Option<ChannelIr>>,
    pub(crate) token_rx: &'a Receiver<LexerToken>,

    active_tokens: [LexerToken; 2]
}

impl<'a> Parser<'a>{
    pub(crate) fn new(
        name: String,
        ir_tx: Sender<Option<ChannelIr>>,
        token_rx: &'a Receiver<LexerToken>
    ) -> Self{
        Parser{
            name,
            ir_tx,
            token_rx,
            active_tokens: [LexerToken::default(), LexerToken::default()]
        }
    }

    #[inline]
    pub(crate) fn advance(&mut self){
        self.active_tokens[CURRENT_TOKEN] = self.active_tokens[NEXT_TOKEN];
        self.active_tokens[NEXT_TOKEN] = self.token_rx.recv().expect("Failed to receive a token from lexer.");
    }
}