use ir::{
    Chunk,
};

use notices::*;
use std::sync::{
    mpsc::{
        Sender, Receiver
    },
};

use core::pos::BiPos;
use futures::executor::ThreadPool;

use core::pos::BiPos as Position;

mod expressions;

mod statement;
use statement::Statement;
mod properties;
mod fun;
mod locals;
mod modules;
mod inference;

///A global manager for all [typeck]s. All typeck's are added to a threadpool upon a call to [enqueueModule].
///A single notice send channel is shared between all typeck's.
///The flow of notices currently is temporary but looks something like this
///```
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
    notice_tx: Sender<Option<Notice>>,
}

impl TypeckManager{
    ///Create a new typeck manager with the given notice sender channel.
    pub fn new(notice_tx: Sender<Option<Notice>>) -> Self{
        TypeckManager{
            thread_pool: ThreadPool::new().unwrap(),
            notice_tx,
        }
    }

    ///Enqueue a module for being type checked in parallel to other stages. See [Driver] for more info.
    ///This will spawn a new task/thread in thread_pool which executes [Typeck::start_checking].
    pub fn enqueue_module(&self, module_name: String, hir_rx: Receiver<Option<Chunk>>, typeck_tx: Sender<Option<Chunk>>){
        let notice_tx_clone = self.notice_tx.clone();
        let module_name_clone = module_name.clone();
        self.thread_pool.spawn_ok(async move{
            let typeck = Typeck::start_checking(module_name_clone.clone(), hir_rx, notice_tx_clone.clone(), typeck_tx);
            if let Err(msg) = typeck{
                let notice = Notice{
                    from: "Typeck".to_string(),
                    file: module_name_clone,
                    level: NoticeLevel::ErrorPrint,
                    msg,
                    pos: Position::default()
                };
                notice_tx_clone.clone().send(Some(notice)).unwrap();
            };
        });
    }
}

///This trait provides an associated function for loading typeck IR into the current typeck instance.
///Output is what type is being returned upon success. Due to the fact that traits don't have known sizes at compiletime, an associated type will do.
///This trait is implemented for different kinds of IR such as a [statements::Statement] or [properties::Property].
pub trait Load{
    ///The type being loaded/returned upon success by [load].
    type Output;
    ///Convert the given [chunk] to an IR for the given [typeck].
    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()>;
}

///An element of a [TyValue]. 
///This has to do with primitive data since primitives are built in. This also allows for convenience when checking for non-primitives types.
///If you have a class called `A`, then the TyValueElement for `A` would be `TyValueElement::Custom("A"). This will then be used during type checking to ensure that
///Values such as calling A's constructor is matched with the required type of the statement.
///```
/////This will have a TyValueElement of Custom("A") during the type checking of the constructor.
///let a = A()
///```
#[derive(Debug, Clone, PartialEq)]
pub enum TyValueElement{
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Custom(String),
    Unit
}

///A value from input and it's type.
///This can include primitive data such as literals, Unit values (aka, void or nothing), or custom types when parsing Constructors.
///See [TyValueElement] for more information.
#[derive(Debug, Clone, PartialEq)]
pub struct TyValue{
    ty: Ty,
    elem: TyValueElement
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

///A type, the meat of the sandwhich.
///Ty represents a type which is used to represent a specific type. Type info is generated or inferred by context.
///A Ty can be inferred or generated depending upon building blocks or sister componenets. Smart casting using the given context to ensure that while within a conditional block that checks for a type's instance, that we safely cast an object's type to the checked type.
///```
/// if(a is B){
///     //Object 'a' was checked in the condition against the type B, so therefore we can safely smart cast 'a' to type B.
///     a.doSomething()
/// }
///```
#[derive(Debug, Clone, PartialEq)]
pub struct Ty{
    ///The name of the type. This is used for comparison.
    pub ident: String,
    ///The location in source code of the type. This can be one of the following:
    /// * A type annotation
    ///     * Function return type
    ///     * Property/local type annotation
    ///     * Class/Struct constructor
    /// * Reference
    ///     * Referencing an object
    ///     * Calling a function
    ///     * Metaprogramming features
    pub pos: BiPos
}

///A trait that provides a method called `get_ty` which is a convenience method for quickly getting an IR element's type info.
pub trait GetTy{
    fn get_ty(&self) -> &Ty;
}


///A part of a local or property whichi contains information about it's mutability. Properties use `var` while locals use `let mut`.
#[derive(Debug, Clone)]
pub struct Mutability{
    pub mutable: bool,
    pub pos: BiPos,
}

///A part of an IR that contains an identifier.
#[derive(Debug, Clone)]
pub struct Identifier{
    ///The identifier
    pub ident: String,
    ///The in source location of the identifier.
    pub pos: BiPos,
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
    ///A clone of the notice_tx provided by [TypeckManager].
    notice_tx: Sender<Option<Notice>>,
    ///The outbound TIR chunks send channel. TIR is 
    typeck_tx: Sender<Option<Chunk>>,
    ///The main module IR instance which represents the entire file as a module. This is where child elements are added.
    module_ir: modules::Module,
}

impl<'a> Typeck{
    ///This method is used for emitting notices. See [Notice] for more information.
    fn emit_notice(&self, msg: String, level: NoticeLevel, pos: Position) -> Result<(),()>{
        if self.notice_tx.send(
            Some(notices::Notice{
                from: "Typeck".to_string(),
                msg,
                file: self.module_name.clone(),
                level,
                pos
            })
        ).is_err(){
            return Err(())
        }
        Ok(())
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
                Ok(statement) => statement,
                Err(()) => return Err(())
            };
            self.module_ir.statements.push(statement);
        }
    }

    ///This is the start of the entire typeck operation, which creates a new typeck object and procceeds to call it's load phase followed by its check phase.
    pub fn start_checking(module_name: String, ir_rx: Receiver<Option<Chunk>>, notice_tx: Sender<Option<Notice>>, typeck_tx: Sender<Option<Chunk>>) -> Result<(), String>{
        let mut typeck = Self{
            module_name: module_name.clone(),
            notice_tx: notice_tx.clone(),
            typeck_tx,
            chunk_rx: ir_rx,
            module_ir: modules::Module{
                ident: module_name.clone(),
                statements: vec![]
            },
        };

        if typeck.load().is_err(){
            return Err("An error occurred while loading bytecode into type analyzer".to_owned())
        }
        
        if typeck.module_ir.check(&typeck).is_err(){
            return Err("An error occurred during type checking".to_owned())
        }

        for elem in typeck.module_ir.statements.iter(){
            println!("{:#?}", elem)
        }

        typeck.typeck_tx.send(None).unwrap();
        
        Ok(())
    }
}