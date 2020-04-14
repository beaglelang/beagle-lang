use ir::{
    type_signature::{
        TypeSignature,
        PrimitiveType,
    },
    hir::{
        ChannelIr,
        Instruction
    },
};
use notices::*;
use std::sync::mpsc::{
    Sender, Receiver
};

use core::pos::BiPos;

pub struct TypeckVM{
    module_name: String,
    symbol_stack: Vec<ChannelIr>,
    ir_rx: Receiver<Option<ChannelIr>>,
    notice_tx: Sender<Option<Notice>>,
    typeck_tx: Sender<Option<ChannelIr>>,
}

impl TypeckVM{
    fn emit_notice(&mut self, msg: String, level: NoticeLevel, pos: BiPos) -> Result<(),()>{
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

    fn check(&mut self) -> Result<(),()>{
        loop{
            let ir = self.ir_rx.recv().unwrap().unwrap();
            let ins = ir.ins;
            if ins == Instruction::Halt{
                break
            }
            let sig = ir.sig;
            match &sig{
                TypeSignature::Primitive(p) => {
                    match p{
                        PrimitiveType::Integer => {
                            match ins{
                                Instruction::Integer(_) => self.symbol_stack.push(ChannelIr{
                                    pos: ir.pos.clone(),
                                    sig,
                                    ins
                                }),
                                _ => {
                                    if self.emit_notice(
                                        format!("Expected an expression of type Integer but instead got {:?}", ins),
                                        NoticeLevel::Error,
                                        ir.pos
                                    ).is_err(){
                                        return Err(())
                                    }
                                }
                            };
                        },
                        PrimitiveType::Float => {
                            match ins{
                                Instruction::Float(_) => self.symbol_stack.push(ChannelIr{
                                    pos: ir.pos.clone(),
                                    sig,
                                    ins
                                }),
                                _ => {
                                    if self.emit_notice(
                                        format!("Expected an expression of type Float but instead got {:?}", ins),
                                        NoticeLevel::Error,
                                        ir.pos
                                    ).is_err(){
                                        return Err(())
                                    }
                                }
                            };
                        },
                        PrimitiveType::String => {
                            match ins{
                                Instruction::String(_) => self.symbol_stack.push(ChannelIr{
                                    pos: ir.pos.clone(),
                                    sig,
                                    ins
                                }),
                                _ => {
                                    if self.emit_notice(
                                        format!("Expected an expression of type String but instead got {:?}", ins),
                                        NoticeLevel::Error,
                                        ir.pos
                                    ).is_err(){
                                        return Err(())
                                    }
                                }
                            };
                        }
                        _ => {
                            if self.emit_notice(
                                format!("Unexpected type: {:?}", ins),
                                NoticeLevel::Error,
                                ir.pos
                            ).is_err(){
                                return Err(())
                            }
                        }
                    }
                },
                TypeSignature::Untyped => {
                    match &ins{
                        Instruction::Integer(_) => {
                            self.typeck_tx.send(
                                Some(ChannelIr{
                                    pos: ir.pos.clone(),
                                    sig: TypeSignature::Primitive(PrimitiveType::Integer),
                                    ins
                                })
                            ).unwrap();
                        },
                        Instruction::Float(_) => {
                            self.typeck_tx.send(
                                Some(ChannelIr{
                                    pos: ir.pos.clone(),
                                    sig: TypeSignature::Primitive(PrimitiveType::Float),
                                    ins
                                })
                            ).unwrap();
                        },
                        Instruction::String(_) => {
                            self.typeck_tx.send(
                                Some(ChannelIr{
                                    pos: ir.pos.clone(),
                                    sig: TypeSignature::Primitive(PrimitiveType::String),
                                    ins
                                })
                            ).unwrap();
                        }
                        Instruction::Bool(_) => {
                            self.typeck_tx.send(
                                Some(ChannelIr{
                                    pos: ir.pos.clone(),
                                    sig: TypeSignature::Primitive(PrimitiveType::Bool),
                                    ins
                                })
                            ).unwrap();
                        }
                        _ => {
                            self.typeck_tx.send(
                                Some(ChannelIr{
                                    pos: ir.pos.clone(),
                                    sig: TypeSignature::Primitive(PrimitiveType::Unit),
                                    ins
                                })
                            ).unwrap();
                        }
                    }
                },
                _ => if self.emit_notice(
                    format!("Expected a primitive type signature but instead got {:?}", sig),
                    NoticeLevel::Error,
                    ir.pos
                ).is_err(){
                    return Err(())
                }
            }
        }
        Ok(())
    }

    pub async fn start_checking(module_name: String, ir_rx: Receiver<Option<ChannelIr>>, notice_tx: Sender<Option<Notice>>, typeck_tx: Sender<Option<ChannelIr>>) -> Result<(), ()>{
        let mut typeck = Self{
            module_name,
            symbol_stack: Vec::new(),
            ir_rx,
            notice_tx,
            typeck_tx
        };

        typeck.check()?;
        
        Ok(())
    }
}