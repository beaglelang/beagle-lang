use ir::{
    hir::{
        HIRInstruction
    },
    Chunk,
};

use notices::*;
use std::sync::{
    mpsc::{
        Sender, Receiver
    },
};

use ir_traits::{
    ReadInstruction,
    WriteInstruction,
};

use futures::executor::ThreadPool;

use core::pos::BiPos as Position;

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
                    level: NoticeLevel::Error,
                    msg,
                    pos: Position::default()
                };
                notice_tx_clone.clone().send(Some(notice)).unwrap();
            };
        });
    }
}

pub struct Typeck{
    module_name: String,
    chunk_rx: Receiver<Option<Chunk>>,
    notice_tx: Sender<Option<Notice>>,
    typeck_tx: Sender<Option<Chunk>>,
}

impl Typeck{
    fn emit_notice(&mut self, msg: String, level: NoticeLevel, pos: Position) -> Result<(),()>{
        if self.notice_tx.send(
            Some(notices::Notice{
                from: "Type checker".to_string(),
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

    ///We need to keep track of the expression chunk and return it after we type check it.
    fn check_expression(&mut self, mut chunk: Chunk) -> Result<Chunk,()>{
        let pos = chunk.read_pos();
        let mut ret_chunk = Chunk::new();
        loop{
            let current = chunk.read_instruction();
            let next = chunk.read_instruction();
            if next != current{
                self.emit_notice(format!("Expected a value of type {:?} but instead got {:?}", current, next), NoticeLevel::Error, pos)?;
                return Err(())
            }
            match &current {
                Some(HIRInstruction::Bool) => {
                    ret_chunk.write_instruction(current.unwrap());
                    let value = chunk.read_bool();
                    ret_chunk.write_bool(value);
                }
                Some(HIRInstruction::Integer) => {
                    ret_chunk.write_instruction(current.unwrap());
                    let value = chunk.read_int();
                    ret_chunk.write_int(value);
                }
                Some(HIRInstruction::Float) => {
                    ret_chunk.write_instruction(current.unwrap());
                    let value = chunk.read_float();
                    ret_chunk.write_float(value);
                }
                Some(HIRInstruction::String) => {
                    ret_chunk.write_instruction(current.unwrap());
                    let value = chunk.read_string();
                    ret_chunk.write_string(value);
                }
                _ => {
                    break;
                }
            }
        }
        Ok(ret_chunk)
    }

    fn check(&mut self) -> Result<(),()>{
        loop{
            let mut chunk = if let Ok(Some(chunk)) = self.chunk_rx.recv(){
                chunk
            }else{
                return Ok(())
            };
            let ins = chunk.read_instruction();
            match &ins{
                Some(HIRInstruction::Property) => {
                    chunk.advance();
                    let pos = chunk.read_pos();
                    chunk.advance();
                    let name = chunk.read_string();
                    chunk.advance();
                    let mutable = chunk.read_bool();
                    chunk.advance();
                    let value_chunk = self.check_expression(chunk)?;
                    let mut new_chunk = Chunk::new();
                    new_chunk.write_instruction(ins.unwrap());
                    new_chunk.write_pos(pos);
                    new_chunk.write_string(name);
                    new_chunk.write_bool(mutable);
                    new_chunk.write_chunk(value_chunk);
                    self.typeck_tx.send(Some(new_chunk)).unwrap();
                },
                Some(_) => return Ok(()),
                None => return Err(())
            }
        }
    }

    pub fn start_checking(module_name: String, ir_rx: Receiver<Option<Chunk>>, notice_tx: Sender<Option<Notice>>, typeck_tx: Sender<Option<Chunk>>) -> Result<(), String>{
        let mut typeck = Self{
            module_name: module_name.clone(),
            notice_tx,
            typeck_tx,
            chunk_rx: ir_rx
        };

        if typeck.check().is_err(){
            return Ok(())
        }        
        
        Ok(())
    }
}