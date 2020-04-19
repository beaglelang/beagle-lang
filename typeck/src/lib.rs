use ir::{
    type_signature::{
        TypeSignature,
        PrimitiveType,
    },
    hir::{
        HIR,
        HIRInstruction
    },
};
use notices::*;
use std::sync::{
    mpsc::{
        Sender, Receiver
    },
    Arc, Mutex
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

    pub fn enqueue_module(&self, module_name: String, hir_rx: Receiver<Option<HIR>>, typeck_tx: Sender<Option<HIR>>){
        let notice_tx_clone = self.notice_tx.clone();
        let module_name_clone = module_name.clone();
        self.thread_pool.spawn_ok(async move{
            let parser = Typeck::start_checking(module_name_clone.clone(), hir_rx, notice_tx_clone.clone(), typeck_tx);
            if let Err(msg) = parser{
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
    ir_stack: Vec<HIR>,
    ir_rx: Receiver<Option<HIR>>,
    notice_tx: Sender<Option<Notice>>,
    typeck_tx: Sender<Option<HIR>>,
}

impl Typeck{
    fn emit_notice(&mut self, msg: String, level: NoticeLevel, pos: Position) -> Result<(),()>{
        if level == NoticeLevel::Error{
            if self.notice_tx.send(
                Some(notices::Notice{
                    from: "Type checker came back with an error.".to_string(),
                    msg: msg.clone(),
                    file: self.module_name.clone(),
                    level,
                    pos
                })
            ).is_err(){
                return Err(())
            }

        }
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

    fn cmp_types(&mut self) -> Result<(), ()>{
        Ok(())
    }

    fn check(&mut self) -> Result<(),()>{
        loop{
            let ir = if let Ok(Some(ir)) = self.ir_rx.recv(){
                ir
            }else{
                return Ok(())
            };
            let ir_clone = ir.clone();
            let ins = ir.ins;
            if ins == HIRInstruction::Halt{
                self.typeck_tx.send(Some(ir_clone)).unwrap();
                break
            }
            let sig = ir.sig.clone();
            match &sig{
                TypeSignature::Primitive(p) => {
                    match ins{
                        HIRInstruction::FnParam(_) => self.ir_stack.push(ir_clone),
                        _ => {
                            let next_ir = if let Ok(Some(ir)) = self.ir_rx.recv(){
                                ir
                            }else{
                                return Ok(())
                            };
                            match p{
                                PrimitiveType::Integer => {
                                    match ins{
                                        HIRInstruction::Integer(_) => self.ir_stack.push(HIR{
                                            pos: ir.pos.clone(),
                                            sig,
                                            ins
                                        }),
                                        _ => {
                                            if self.emit_notice(
                                                format!("Expected an expression of type Integer but instead got {:?}", next_ir.sig),
                                                NoticeLevel::Error,
                                                ir.pos
                                            ).is_err(){
                                                return Err(())
                                            }
                                            return Err(())
        
                                        }
                                    };
                                },
                                PrimitiveType::Float => {
                                    match ins{
                                        HIRInstruction::Float(_) => self.ir_stack.push(HIR{
                                            pos: ir.pos.clone(),
                                            sig,
                                            ins
                                        }),
                                        _ => {
                                            if self.emit_notice(
                                                format!("Expected an expression of type Float but instead got {:?}", next_ir.sig),
                                                NoticeLevel::Error,
                                                ir.pos
                                            ).is_err(){
                                                return Err(())
                                            }
                                            return Err(())
        
                                        }
                                    };
                                },
                                PrimitiveType::String => {
                                    match ins{
                                        HIRInstruction::String(_) => self.ir_stack.push(HIR{
                                            pos: ir.pos.clone(),
                                            sig,
                                            ins
                                        }),
                                        _ => {
                                            if self.emit_notice(
                                                format!("Expected an expression of type String but instead got {:?}", next_ir.sig),
                                                NoticeLevel::Error,
                                                ir.pos
                                            ).is_err(){
                                                return Err(())
                                            }
                                            return Err(())
                                        }
                                    };
                                }
                                _ => {
                                    if self.emit_notice(
                                        format!("Unexpected type: {:?}", next_ir.sig),
                                        NoticeLevel::Error,
                                        ir.pos
                                    ).is_err(){
                                        return Err(())
                                    }
                                    return Err(())
        
                                }
                            }
                        }
                    }
                },
                TypeSignature::Untyped => {
                    let next_ir = if let Ok(Some(ir)) = self.ir_rx.recv(){
                        ir
                    }else{
                        return Ok(())
                    };
                    match &next_ir.ins{
                        HIRInstruction::Integer(_) => {
                            self.ir_stack.push(HIR{
                                pos: ir.pos,
                                sig: TypeSignature::Primitive(PrimitiveType::Integer),
                                ins
                            });
                        },
                        HIRInstruction::Float(_) => {
                            self.ir_stack.push(HIR{
                                pos: ir.pos,
                                sig: TypeSignature::Primitive(PrimitiveType::String),
                                ins
                            });
                        },
                        HIRInstruction::String(_) => {
                            self.ir_stack.push(HIR{
                                pos: ir.pos,
                                sig: TypeSignature::Primitive(PrimitiveType::String),
                                ins
                            });
                        }
                        HIRInstruction::Bool(_) => {
                            self.ir_stack.push(HIR{
                                pos: ir.pos,
                                sig: TypeSignature::Primitive(PrimitiveType::Bool),
                                ins
                            });
                        }
                        _ => {
                            self.ir_stack.push(HIR{
                                pos: ir.pos,
                                sig: TypeSignature::Primitive(PrimitiveType::Unit),
                                ins
                            });
                        }
                    }
                    self.ir_stack.push(next_ir);
                },
                _ => self.ir_stack.push(ir_clone)
            }
        }
        Ok(())
    }

    pub fn start_checking(module_name: String, ir_rx: Receiver<Option<HIR>>, notice_tx: Sender<Option<Notice>>, typeck_tx: Sender<Option<HIR>>) -> Result<(), String>{
        let mut typeck = Self{
            module_name: module_name.clone(),
            ir_stack: Vec::new(),
            ir_rx,
            notice_tx,
            typeck_tx
        };

        if typeck.check().is_err(){
            return Ok(())
        }

        for ir in typeck.ir_stack{
            if let Err(_) = typeck.typeck_tx.send(Some(ir)){
                return Err(format!("Failed to send type checked ir for module '{}' because the IR channel was expected to be open but is instead closed.", module_name.clone()))
            }
        }
        
        
        Ok(())
    }
}