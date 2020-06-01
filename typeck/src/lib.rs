use ir::{
    Chunk,
};

use std::sync::{
    mpsc::{
        Sender, Receiver, 
    },
    Arc, Mutex
};

use futures::executor::ThreadPool;

use notices::{
    Diagnostic,
    DiagnosticBuilder,
    DiagnosticSource,
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

use module_messages::ModuleMessage;

use core::pos::BiPos;

mod expressions;

mod statement;
use stmt::Statement;
mod properties;
mod fun;
mod locals;
mod modules;
mod ty;
mod ident;
mod mutable;

///This trait provides an associated function for loading typeck IR into the current typeck instance.
///Output is what type is being returned upon success. Due to the fact that traits don't have known sizes at compiletime, an associated type will do.
///This trait is implemented for different kinds of IR such as a [statements::Statement] or [properties::Property].
pub trait Load{
    ///The type being loaded/returned upon success by [load].
    type Output;
    ///Convert the given [chunk] to an IR for the given [typeck].
    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, ()>;
}

pub trait Unload{
    fn unload(&self) -> Result<Chunk, ()>;
}

///A global manager for all [typeck]s. All typeck's are added to a threadpool upon a call to [enqueueModule].
///A single notice send channel is shared between all typeck's.
///The flow of notices currently is temporary but looks something like this
///```norust
///           ^
///           |
///     TypeckManager
///     /     |     \
///    /      |      \
/// Typeck  Typeck  Typeck
///```
///TODO:
/// * Implement notices stack and a global error handling system. 
///     * This will require that upon an error being emitted by a typeck, the manager will emit a shut down message to the compiler driver, which will commence a global shutdown process.
/// * Implement an intertypeck messaging system
///     * This will allow each individual typeck's to send a message to the manager which will send the message's payload to the targeted module/typeck. The payload is a request for type info on a given symbol.
///         ```
///             struct Message{
///                 //A CanonicalPath will have methods for acting upon a canonical path. A canonical path would be like `A::B::C`, where each `::` is a path separator. This is used for splitting the module's name from the parent's canonical name.
///                 module_path: CanonicalPath{
///                     ident: String
///                 }
///                 payload: MessagePayload
///             }
///             enum MessagePayload{
///                  //The identifier of a symbol being requested
///                 SymbolTyRequest(String),
///                 //A response to SymboltyRequest where a clone of the symbol's type is attached with the symbol's identifier.
///                 SymbolTyResponse(String, Ty)
///             }
///         ```
pub struct TypeckManager{
    ///The threadpool of typeck instances. This is populated by [enqueueModule].
    thread_pool: ThreadPool,
    ///A global copy of a notice sender channel that all typeck's are given clones of.
    notice_tx: Sender<Option<Diagnostic>>,
}

impl TypeckManager{
    ///Create a new typeck manager with the given notice sender channel.
    pub fn new(notice_tx: Sender<Option<Diagnostic>>) -> Self{
        TypeckManager{
            thread_pool: ThreadPool::new().unwrap(),
            notice_tx,
        }
    }

    ///Enqueue a module for being type checked in parallel to other stages. See [Driver] for more info.
    ///This will spawn a new task/thread in thread_pool which executes [Typeck::start_checking].
    pub fn enqueue_module(&self, module_name: String, hir_rx: Receiver<Option<Chunk>>, typeck_tx: Sender<Option<Chunk>>, master_tx: Sender<ModuleMessage>, master_rx: Arc<Mutex<Receiver<ModuleMessage>>>){
        let notice_tx_clone = self.notice_tx.clone();
        let module_name_clone = module_name.clone();
        self.thread_pool.spawn_ok(async move{
            let _ = Typeck::start_checking(module_name_clone.clone(), hir_rx, typeck_tx, master_tx, master_rx, notice_tx_clone.clone());
        });
    }
}

///This trait provides an associative function for checking an IR during the `check` phase.
///This trait has a lifetime paramter ['a]. This lifetime parameter should be the same lifetime as a Typeck::start_checking. 
///The current Typeck instance will only exist as long as Typeck::start_checking is still a valid scope. All loading and checking will only occur while Typeck::start_checking is valid. 
trait Check<'a>{
    ///Check the current IR and return `Err(())` if an error notice has been emitted to the typeck.
    ///This function will only ever be called after `load` phase has successfully completed.
    ///param: typeck The typeck instance involved in the checking phase.
    fn check(&self, typeck: &'a Typeck) -> Result<(), ()>;
}

///A single instance of a type checker, thus the shortened name *Typeck*. Each file is given its own Typeck. 
///Explicit source module declarations are part of the IR and are not given their own typecks.
///Example:
///```
/// //This is part of its parent module's typeck and will be represented internally as a `Module` instead of being given its own typeck.
/// mod A{
///     
/// }
///```
///This produces a modified version of HIR called TIR or Type-checked Intermediate Representation.
pub struct Typeck{
    ///The file name. Every file is a module.
    module_name: String,
    ///The inbound HIR chunk receive channel. `None` if there are no more chunks coming.
    chunk_rx: Receiver<Option<Chunk>>,
    ///The outbound TIR chunks send channel. TIR is 
    typeck_tx: Sender<Option<Chunk>>,
    ///The main module IR instance which represents the entire file as a module. This is where child elements are added.
    module_ir: modules::Module,

    diagnostic_tx: Sender<Option<Diagnostic>>,

    master_tx: Sender<ModuleMessage>,
    master_rx: Arc<Mutex<Receiver<ModuleMessage>>>
}

impl<'a> Typeck{
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
    
    ///This is the start of the load phase. This begins to take in HIR chunks and calls `Statement::load` with that chunk and the current typeck.
    //The produced Statement object is added to [module_ir].
    fn load(&mut self) -> Result<(),()>{
        loop{
            let chunk = if let Ok(Some(chunk)) = self.chunk_rx.recv(){
                chunk
            }else{
                return Ok(())
            };
            let statement = match Statement::load(&chunk, self){
                Ok(Some(statement)) => statement,
                Ok(None) => return Ok(()),
                Err(notice) => return Err(notice)
            };
            self.module_ir.statements.push(statement);
        }
    }

    fn unload(&self) -> Result<(),()>{
        for statement in self.module_ir.statements.iter(){
            let ch = match statement.unload(){
                Ok(chunk) => chunk,
                Err(notice) => return Err(notice)
            };
            self.typeck_tx.send(Some(ch)).unwrap();
        }
        Ok(())
    }

    ///This is the start of the entire typeck operation, which creates a new typeck object and procceeds to call it's load phase followed by its check phase.
    pub fn start_checking(
                        module_name: String, 
                        ir_rx: Receiver<Option<Chunk>>, 
                        typeck_tx: Sender<Option<Chunk>>, 
                        master_tx: Sender<ModuleMessage>, 
                        master_rx: Arc<Mutex<Receiver<ModuleMessage>>>,
                        diagnostic_tx: Sender<Option<Diagnostic>>
                    ) -> Result<(), ()>{
        let mut typeck = Self{
            module_name: module_name.clone(),
            typeck_tx,
            chunk_rx: ir_rx,
            module_ir: modules::Module{
                ident: module_name.clone(),
                statements: vec![]
            },
            master_tx,
            master_rx,
            diagnostic_tx
        };

        if let Err(()) = typeck.load(){
            return Err(())
        }
        
        if let Err(()) = typeck.module_ir.check(&typeck){
            return Err(())
        }

        if let Err(()) = typeck.unload(){
            return Err(())
        }

        typeck.typeck_tx.send(None).unwrap();
        
        Ok(())
    }
}