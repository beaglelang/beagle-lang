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

pub struct TypeckManager{
    thread_pool: ThreadPool,
    notice_tx: Sender<Option<Notice>>,
}

impl TypeckManager{
    pub fn new(notice_tx: Sender<Option<Notice>>) -> Self{
        TypeckManager{
            thread_pool: ThreadPool::new().unwrap(),
            notice_tx,
        }
    }

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

pub trait Load{
    type Output;
    fn load(chunk: Chunk, typeck: &Typeck) -> Result<Self::Output, ()>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum TyValueElement{
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Custom(String),
    Unit
}

#[derive(Debug, Clone, PartialEq)]
pub struct TyValue{
    ty: Ty,
    elem: TyValueElement
}

trait Check<'a>{
    fn check(&self, typeck: &'a Typeck) -> Result<(), ()>;
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ty{
    pub ident: String,
    pub pos: BiPos
}

pub trait GetTy{
    fn get_ty(&self) -> &Ty;
}

#[derive(Debug, Clone)]
pub struct Mutability{
    pub mutable: bool,
    pub pos: BiPos,
}

#[derive(Debug, Clone)]
pub struct Identifier{
    pub ident: String,
    pub pos: BiPos,
}

pub struct Typeck{
    module_name: String,
    chunk_rx: Receiver<Option<Chunk>>,
    notice_tx: Sender<Option<Notice>>,
    typeck_tx: Sender<Option<Chunk>>,
    module_ir: modules::Module,
}

impl<'a> Typeck{
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

    fn load(&mut self) -> Result<(),()>{
        loop{
            let chunk = if let Ok(Some(chunk)) = self.chunk_rx.recv(){
                chunk
            }else{
                return Ok(())
            };
            let statement = match Statement::load(chunk, self){
                Ok(statement) => statement,
                Err(()) => return Err(())
            };
            self.module_ir.statements.push(statement);
        }
    }

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