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

pub trait Load{
    type Output;
    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, DiagnosticSource>;
}

pub trait Check{
    fn check(&self, memmy: &MemmyGenerator) -> Result<(), DiagnosticSource>;
}

pub trait Unload{
    fn unload(&self) -> Result<Chunk, ()>;
}

#[derive(Debug, Clone)]
struct Mutability{
    mutable: bool,
    pos: BiPos,
}

#[allow(dead_code)]
struct MemmyElement{
    bc_index: usize,
    count: usize,
    //
    refs: Vec<(String, usize)>
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
    symbol_table: Vec<MemmyElement>,
    final_chunk: Chunk,
    mir_tx: Sender<Option<Chunk>>,
    typeck_rx: Receiver<Option<Chunk>>,
    notice_tx: Sender<Option<Diagnostic>>,
    master_tx: Sender<ModuleMessage>,
    master_rx: Arc<Mutex<Receiver<ModuleMessage>>>,
}

impl MemmyGenerator{

    pub fn start(module_name: String, mir_tx: Sender<Option<Chunk>>, notice_tx: Sender<Option<Diagnostic>>, typeck_rx: Receiver<Option<Chunk>>, master_tx: Sender<ModuleMessage>, master_rx: Arc<Mutex<Receiver<ModuleMessage>>>) -> Result<(),()>{
        let memmy = Self{
            module_name,
            symbol_table: Vec::new(),
            mir_tx,
            notice_tx: notice_tx.clone(),
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
                Err(diag) => {
                    let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
                        .add_source(diag)
                        .message(format!("An error occurred during memory analysis"))
                        .build();
                    notice_tx.send(Some(diagnostic)).unwrap();
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
