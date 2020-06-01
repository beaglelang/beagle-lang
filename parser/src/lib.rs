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
    Chunk,
};


use std::sync::{Arc, Mutex};

use notices::{
    DiagnosticLevel, DiagnosticBuilder, Diagnostic, DiagnosticSource, DiagnosticSourceBuilder
};

use futures::executor::ThreadPool;

use module_messages::ModuleMessage;

///A parse rule, which emits a chunk upon completion. Returns a notice upon error.
pub trait ParseRule{
    fn parse(p: &mut Parser) -> Result<(), ()>;
}

///A trait for parse rules that are part of another parse rule, and require a chunk to be returned as opposed to emitting it.
pub trait OwnedParse{
    fn owned_parse(parser: &mut Parser) -> Result<Chunk, DiagnosticSource>;
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
    notice_tx: Arc<Mutex<Sender<Option<Diagnostic>>>>,
}

impl ParseManager{
    pub fn new(notice_tx: Sender<Option<Diagnostic>>) -> Self{
        ParseManager{
            thread_pool: ThreadPool::new().unwrap(),
            notice_tx: Arc::new(Mutex::new(notice_tx)),
        }
    }

    pub fn enqueue_module(&self, module_name: String, token_rx: Receiver<LexerToken>, hir_tx: Sender<Option<Chunk>>, master_tx: Sender<ModuleMessage>, master_rx: Receiver<ModuleMessage>){
        let notice_tx_clone = self.notice_tx.lock().unwrap().clone();
        self.thread_pool.spawn_ok(async move{
            let _ = Parser::parse(module_name.clone(), hir_tx, token_rx, notice_tx_clone.clone(), master_tx, master_rx);
        });
    }
}

pub struct Parser {
    pub name: String,
    pub ir_tx: Arc<Mutex<Sender<Option<Chunk>>>>,
    pub token_rx: Receiver<LexerToken>,
    pub notice_tx: Sender<Option<Diagnostic>>,
    pub context: ParseContext,

    active_tokens: [LexerToken; 3],
    master_tx: Sender<ModuleMessage>,
    master_rx: Receiver<ModuleMessage>,
}

impl Parser {
    pub fn new(
        name: String,
        ir_tx: Sender<Option<Chunk>>,
        token_rx: Receiver<LexerToken>,
        notice_tx: Sender<Option<Diagnostic>>,
        master_tx: Sender<ModuleMessage>,
        master_rx: Receiver<ModuleMessage>,
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
            master_tx,
            master_rx
        }
    }

    pub fn request_source_snippet(&self) -> Result<String, DiagnosticSource>{
        if let Err(_) = self.master_tx.send(ModuleMessage::SourceRequest(self.current_token().pos)){
            let diag = DiagnosticSourceBuilder::new(self.name.clone(), 0)
                .level(DiagnosticLevel::Error)
                .message(format!("The master channel was closed??"))
                .build();
                return Err(diag);
        }
        return match self.master_rx.recv(){
            Ok(ModuleMessage::SourceResponse(source_snip)) => Ok(source_snip),
            Ok(thing) => {
                let diag = DiagnosticSourceBuilder::new(self.name.clone(), 0)
                .level(DiagnosticLevel::Error)
                .message(format!("Not sure what we got but we shouldn't have: {:?}", thing))
                .build();
                return Err(diag)
            },
            Err(_) => {
                let diag = DiagnosticSourceBuilder::new(self.name.clone(), 0)
                .level(DiagnosticLevel::Error)
                .message(format!("The master channel was closed??"))
                .build();
                return Err(diag);
            }
        };
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
    pub fn advance(&mut self) -> Result<(), DiagnosticSource> {
        self.active_tokens[PREV_TOKEN] = self.active_tokens[CURRENT_TOKEN].clone();
        self.active_tokens[CURRENT_TOKEN] = self.active_tokens[NEXT_TOKEN].clone();
        self.active_tokens[NEXT_TOKEN] = match self
            .token_rx
            .recv_timeout(std::time::Duration::from_secs(1))
        {
            Err(_) =>{ 
                let diag_source = DiagnosticSourceBuilder::new(self.name.clone(), self.current_token().pos.start.0)
                    .message(format!("Failed to receive token from tokenizer: token channel closed prematurely.\nReport this to the author:\n\tAlex Couch: alcouch65@gmail.com\n\tGithub Issues: https://github.com/beaglelang/beagle-lang/issues\n\tTuring Tarpit: https://discord.gg/RmgjcES"))
                    .build();
                return Err(diag_source)
            },
            Ok(t) => t,
        };

        Ok(())
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
    pub fn check_consume(&mut self, type_: TokenType) -> Result<bool, DiagnosticSource> {
        if self.check(type_) {
            if let Err(notice) = self.advance(){
                return Err(notice)
            };
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    pub fn check_consume_next(&mut self, type_: TokenType) -> Result<bool, DiagnosticSource> {
        if self.check_next(type_) {
            self.advance()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    #[inline]
    pub fn consume(
        &mut self,
        type_: TokenType
    ) -> Result<&TokenData, DiagnosticSource> {
        self.advance()?;
        if self.check(type_) {
            Ok(&self.current_token().data)
        } else {
            let source = match self.request_source_snippet(){
                Ok(source) => source,
                Err(diag) => return Err(diag)
            };
            let diag_source = DiagnosticSourceBuilder::new(self.name.clone(),self.current_token().pos.start.0)
                        .message(format!("Expected a '{:?}' but instead got {:?}", type_, self.current_token().type_))
                        .level(DiagnosticLevel::Error)
                        .range(self.current_token().pos.col_range())
                        .source(source)
                        .build();
            return Err(diag_source)
        }
    }

    pub fn emit_parse_diagnostic(&self, notes: &[String], source: &[DiagnosticSource]){
        let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                    .message(format!("An error occurred during parsing."))
                    .add_sources(source)
                    .add_notes(notes)
                    .build();
        self.notice_tx.send(Some(diag)).unwrap();
    }

    pub fn parse(
        name: String,
        ir_tx: Sender<Option<Chunk>>,
        token_rx: Receiver<LexerToken>,
        notice_tx: Sender<Option<Diagnostic>>,
        master_tx: Sender<ModuleMessage>,
        master_rx: Receiver<ModuleMessage>,
    ) -> Result<(), ()> {
        let mut parser = Parser::new(name, ir_tx, token_rx, notice_tx, master_tx, master_rx);
        if let Err(diag) = parser.advance(){
            parser.emit_parse_diagnostic(&[], &[diag]);
            return Err(())
        };
        if let Err(diag) = parser.advance(){
            parser.emit_parse_diagnostic(&[], &[diag]);
            return Err(())
        };
        while !parser.check(TokenType::Eof) {
            if let Err(()) = statements::StatementParser::parse(&mut parser) {
                if parser.ir_tx.lock().unwrap().send(None).is_err(){
                    parser.notice_tx.send(Some(DiagnosticBuilder::new(DiagnosticLevel::Error)
                        .message(format!("Failed to send halt signal to rest of compiler: channel closed prematurely.\n
                            Report this to the author:\n\t
                            Alex Couch: alcouch65@gmail.com\n\t
                            Github Issues: https://github.com/beaglelang/beagle-lang/issues\n\t
                            Turing Tarpit: https://discord.gg/RmgjcES"
                        ))
                        .build())).unwrap();
                }
                return Err(())
            }
        }
        if parser.ir_tx.lock().unwrap().send(None).is_err(){
            return Ok(())
        }
        Ok(())
    }
}

