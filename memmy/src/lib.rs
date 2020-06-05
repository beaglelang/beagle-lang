use ir::{
    Chunk,
};

use std::{
    sync::{
        mpsc::{
            Sender,
            Receiver
        },
        Arc, Mutex
    }
};

use notices::{
    Diagnostic,
    DiagnosticBuilder,
    DiagnosticLevel,
    DiagnosticSource,
};

use core::pos::BiPos;

use futures::executor::ThreadPool;

use module_messages::ModuleMessage;

mod statements;
mod ident;
mod property;
mod fun;
mod local;
mod expr;
mod module;
mod lifetime;
mod ty;
mod mutability;

pub trait Load{
    type Output;
    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()>;
}

pub trait Check{
    fn check(&self, memmy: &MemmyGenerator) -> Result<(), ()>;
}

pub trait Unload{
    fn unload(&self) -> Result<Chunk, ()>;
}

#[derive(Debug, Clone)]
struct Mutability{
    mutable: bool,
    pos: BiPos,
}

pub struct MemmyManager{
    ///The threadpool of typeck instances. This is populated by [enqueueModule].
    thread_pool: ThreadPool,
}

impl MemmyManager{
    ///Create a new memmy manager with the given notice sender channel.
    pub fn new() -> Self{
        MemmyManager{
            thread_pool: ThreadPool::new().unwrap(),
        }
    }

    ///Enqueue a module for being type checked in parallel to other stages. See [Driver] for more info.
    ///This will spawn a new task/thread in thread_pool which executes [Typeck::start_checking].
    pub fn enqueue_module(&self, module_name: String, diagnostics_tx: Sender<Option<Diagnostic>>, typeck_rx: Receiver<Option<Chunk>>, mir_tx: Sender<Option<Chunk>>, master_tx: Sender<ModuleMessage>, master_rx: Arc<Mutex<Receiver<ModuleMessage>>>){
        let module_name_clone = module_name.clone();
        self.thread_pool.spawn_ok(async move{
            let typeck = MemmyGenerator::start(module_name_clone.clone(), mir_tx, diagnostics_tx, typeck_rx, master_tx, master_rx);
            if let Err(()) = typeck{
                return
            };
        });
    }
}

#[allow(dead_code)]
pub struct MemmyGenerator{
    module_name: String,
    final_chunk: Chunk,
    mir_tx: Sender<Option<Chunk>>,
    typeck_rx: Receiver<Option<Chunk>>,
    diagnostic_tx: Sender<Option<Diagnostic>>,
    master_tx: Sender<ModuleMessage>,
    master_rx: Arc<Mutex<Receiver<ModuleMessage>>>,
}

impl MemmyGenerator{

    pub fn emit_diagnostic(&self, notes: &[String], diag_sources: &[DiagnosticSource]){
        let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
                .message(format!("An error occurred during memory analysis"))
                .add_sources(diag_sources)
                .add_notes(notes)
                .build();
        self.diagnostic_tx.send(Some(diagnostic)).unwrap();
    }

    pub fn start(module_name: String, mir_tx: Sender<Option<Chunk>>, diagnostic_tx: Sender<Option<Diagnostic>>, typeck_rx: Receiver<Option<Chunk>>, master_tx: Sender<ModuleMessage>, master_rx: Arc<Mutex<Receiver<ModuleMessage>>>) -> Result<(),()>{
        let memmy = Self{
            module_name,
            mir_tx,
            diagnostic_tx: diagnostic_tx.clone(),
            typeck_rx,
            final_chunk: Chunk::new(),
            master_tx,
            master_rx
        };
        let mut statements = vec![];
        loop{
            let chunk = if let Ok(Some(chunk)) = memmy.typeck_rx.recv(){
                chunk
            }else{
                break
            };
            let statement = match statements::Statement::load(&chunk, &memmy){
                Ok(statement) => statement,
                Err(()) => {
                    return Err(())
                }
            };
            statements.push(statement);
        }
        for statement in statements.iter(){
            println!("{:?}", statement);
        }
        Ok(())
    }
}
