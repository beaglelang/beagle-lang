use futures::executor::ThreadPool;

use std::sync::{
    Arc, Mutex,
    mpsc::{
        Sender,
        Receiver
    }
};

use notices::{
    Diagnostic,
    DiagnosticBuilder,
    DiagnosticLevel,
    DiagnosticSource,
    DiagnosticSourceBuilder
};

use ir::{
    Chunk,
};

use module_messages::ModuleMessage;

mod modules;
use modules::Module;

mod statement;
use statement::{
    Statement,
    StatementKind
};
mod property;
mod mutable;
mod ident;
mod ty;
mod fun;
mod local;
mod expr;
mod symbol;
use symbol::Symbol;

use core::pos::BiPos;

pub trait Load{
    ///The type being loaded/returned upon success by [load].
    type Output;
    ///Convert the given [chunk] to an IR for the given [typeck].
    fn load(chunk: &Chunk, typeck: &SymbolResolver) -> Result<Option<Self::Output>, ()>;
}

pub trait Unload{
    fn unload(&self, typeck: &SymbolResolver) -> Result<Chunk, ()>;
}

pub trait ResolveSymbols{
    fn resolve(&self, typeck: &SymbolResolver) -> Result<(), ()>;
}

pub trait PartialResolve{
    fn partial_resolve(&self, typeck: &SymbolResolver) -> Result<(), ()>;
}

pub struct SymbolResolverManager{
    ///The threadpool of typeck instances. This is populated by [enqueueModule].
    thread_pool: ThreadPool,
}

impl SymbolResolverManager{
    ///Create a new typeck manager with the given notice sender channel.
    pub fn new() -> Self{
        SymbolResolverManager{
            thread_pool: ThreadPool::new().unwrap(),
        }
    }

    ///Enqueue a module for being type checked in parallel to other stages. See [Driver] for more info.
    ///This will spawn a new task/thread in thread_pool which executes [Typeck::start_checking].
    pub fn enqueue_module(&self, module_name: String, diagnostics_tx: Sender<Option<Diagnostic>>, hir_rx: Receiver<Option<Chunk>>, typeck_tx: Sender<Option<Chunk>>, master_tx: Sender<ModuleMessage>, master_rx: Arc<Mutex<Receiver<ModuleMessage>>>){
        let module_name_clone = module_name.clone();
        self.thread_pool.spawn_ok(async move{
            let _ = SymbolResolver::start(module_name_clone.clone(), hir_rx, typeck_tx, master_tx, master_rx, diagnostics_tx);
        });
    }
}

pub struct SymbolResolver{
    module_name: String,
    module: Module,
    ir_rx: Receiver<Option<Chunk>>,
    sr_tx: Sender<Option<Chunk>>,
    master_tx: Sender<ModuleMessage>,
    master_rx: Arc<Mutex<Receiver<ModuleMessage>>>,
    diagnostic_tx: Sender<Option<Diagnostic>>
}

impl<'a> SymbolResolver{
    pub fn request_source_snippet(&self, pos: BiPos) -> Result<String, DiagnosticSource>{
        if let Err(_) = self.master_tx.send(ModuleMessage::SourceRequest(pos)){
            let diag = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                .level(DiagnosticLevel::Error)
                .message(format!("The master channel was closed??"))
                .build();
            return Err(diag);
        }
        let master_rx_lock = match self.master_rx.lock(){
            Ok(lock ) => lock,
            Err(err) => {
                return Err(DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(err.to_string())
                    .build());
            }
        };
        return match master_rx_lock.recv(){
            Ok(ModuleMessage::SourceResponse(source_snip)) => Ok(source_snip),
            Ok(thing) => {
                let diag = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                .level(DiagnosticLevel::Error)
                .message(format!("Not sure what we got but we shouldn't have: {:?}", thing))
                .build();
                return Err(diag)
            },
            Err(_) => {
                let diag = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                .level(DiagnosticLevel::Error)
                .message(format!("The master channel was closed??"))
                .build();
                return Err(diag);
            }
        };
    }

    pub fn emit_diagnostic(&self, notes: &[String], diag_sources: &[DiagnosticSource]){
        let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
                    .message(format!("An error occurred during type checking."))
                    .add_sources(diag_sources)
                    .add_notes(notes)
                    .build();
        self.diagnostic_tx.send(Some(diagnostic)).unwrap();
    }

    pub fn find_symbol(&self, symbol: String) -> Option<Symbol<'a>>{
        for stmt in self.module.statements.iter(){
            match stmt.kind{
                StatementKind::Property(property) => {
                    if property.ident.ident == symbol{
                        return Some(Symbol::Property(&property))
                    }
                }
                StatementKind::Fun(fun) => {
                    if fun.ident.ident == symbol{
                        return Some(Symbol::Fun(&fun))
                    }
                }
                StatementKind::Local(local) => {
                    if local.ident.ident == symbol{
                        return Some(Symbol::Local(&local))
                    }
                }
            }
        }
        return None
    }

    fn load(&mut self) -> Result<(),()>{
        let chunk = if let Ok(Some(chunk)) = self.ir_rx.recv(){
            chunk
        }else{
            return Ok(())
        };
        let statement = match Statement::load(&chunk, self){
            Ok(Some(statement)) => statement,
            Ok(None) => return Ok(()),
            Err(notice) => return Err(notice)
        };
        self.module.statements.push(statement);
        Ok(())
    }

    fn start(
        module_name: String, 
        ir_rx: Receiver<Option<Chunk>>, 
        sr_tx: Sender<Option<Chunk>>, 
        master_tx: Sender<ModuleMessage>, 
        master_rx: Arc<Mutex<Receiver<ModuleMessage>>>,
        diagnostic_tx: Sender<Option<Diagnostic>>
    ) -> Result<(),()>{
        let mut symbol_resolver = Self{
            module_name: module_name.clone(),
            module: Module{
                ident: module_name.clone(),
                statements: vec![]
            },
            ir_rx,
            sr_tx,
            master_tx,
            master_rx,
            diagnostic_tx
        };
        match symbol_resolver.load(){
            Ok(()) => {},
            Err(()) => return Err(()),
        };
        Ok(())
    }
}