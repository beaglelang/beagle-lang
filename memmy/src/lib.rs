use ir::{
    hir::{
        HIR,
        HIRInstruction,
    },
    mir::{
        MIR,
        MIRInstruction
    },
    type_signature::{
        TypeSignature,
        PrimitiveType
    }
};

use std::{
    sync::mpsc::{
        Sender,
        Receiver
    },
    mem::{
        size_of
    }
};

use notices::{
    Notice,
    NoticeLevel
};

use core::pos::BiPos;

struct MemmyElement{
    hir: HIR,
    count: usize,
    refs: Vec<HIR>
}

pub struct MemmyGenerator{
    module_name: String,
    symbol_table: Vec<MemmyElement>,
    completed_mir: Vec<MIR>,
    mir_tx: Sender<Option<MIR>>,
    typeck_rx: Receiver<Option<HIR>>,
    notice_tx: Sender<Option<Notice>>
}

impl MemmyGenerator{
    fn emit_error(&mut self, msg: String, pos: BiPos) -> Result<(),()>{
        self.emit_notice(msg, NoticeLevel::Error, pos)
    }

    fn emit_notice(&mut self, msg: String, level: NoticeLevel, pos: BiPos) -> Result<(),()>{
        if self.notice_tx.send(
            Some(Notice{
                from: "Memmy".to_string(),
                pos,
                msg,
                file: self.module_name.clone(),
                level
            })
        ).is_err(){
            return Err(())
        }
        return Ok(())
    }

    fn determine_alloc_size(&mut self, hir: HIR) -> Option<(usize, Option<MIR>)>{
        match hir.sig{
            TypeSignature::Primitive(t) => match t{
                PrimitiveType::Float => Some((size_of::<f32>(), None)),
                PrimitiveType::Integer => Some((size_of::<i32>(), None)),
                PrimitiveType::Bool => Some((size_of::<bool>(), None)),
                PrimitiveType::String => {
                    let next_hir = self.typeck_rx.recv().unwrap().unwrap();
                    return if let HIRInstruction::String(s) = next_hir.ins{
                        let mir = MIR{
                            pos: next_hir.pos,
                            sig: next_hir.sig,
                            ins: MIRInstruction::String(s.clone())
                        };
                        Some((s.len(), Some(mir)))
                    }else{
                        self.emit_error(format!("Failed to determine size of String object: Expected a String object but instead got {:?}", hir.ins), next_hir.pos).unwrap();
                        None
                    }
                },
                _ => {
                    Some((size_of::<usize>(), None))
                }
            },
            TypeSignature::Untyped => {
                self.emit_error("Found an untyped local, which is an illegal operation. This should not be happening. Please report this to the author.".to_string(), hir.pos).unwrap();
                None
            },
            _ => {
                self.emit_error(format!("Found an unknown type signature: {:?}", hir.sig.clone()).to_string(), hir.pos).unwrap();
                None
            }
        }
    }

    fn convert_function_block(&mut self, hir: HIR) -> Result<(),()>{
        let mut header = Vec::<MIR>::new();
        let mut preallocs = Vec::<MIR>::new();
        let mut other = Vec::<MIR>::new();
        if let HIRInstruction::Fn(name) = hir.ins{
            header.push(MIR{
                pos: hir.pos,
                sig: hir.sig,
                ins: MIRInstruction::Fun(name)
            });
        }else{
            self.emit_error("Failed to convert HIR function instruction to MIR function instruction.".to_string(), hir.pos)?;
        }
        loop{
            let next_hir = self.typeck_rx.recv().unwrap().unwrap();
            match &next_hir.ins{
                HIRInstruction::FnParam(name) => {
                    header.push(
                        MIR{
                            pos: next_hir.pos,
                            sig: next_hir.sig,
                            ins: MIRInstruction::FunParam(name.clone())
                        }
                    )
                },
                HIRInstruction::LocalVar(name, mutable) => {
                    let size_clone = next_hir.clone();
                    let size = if let Some(size) = self.determine_alloc_size(size_clone){
                        size
                    }else{
                        let error_clone = next_hir.clone();
                        self.emit_error("Failed to determine size of local object.".to_string(), error_clone.clone().pos)?;
                        return Err(())
                    };
                    let prealloc_clone = next_hir.clone();
                    preallocs.push(
                        MIR{
                            pos: prealloc_clone.pos,
                            sig: prealloc_clone.sig,
                            ins: MIRInstruction::StackAlloc(name.clone(), size.0)
                        }
                    );
                    let clone = next_hir.clone();
                    other.push(
                        MIR{
                            pos: clone.pos,
                            sig: clone.sig,
                            ins: MIRInstruction::ObjInit(name.clone(), *mutable)
                        }
                    );
                    if let Some(mir) = size.1{
                        other.push(mir)
                    }
                },
                HIRInstruction::Property(name, mutable) => {
                    let size_clone = next_hir.clone();
                    let size = if let Some(size) = self.determine_alloc_size(size_clone){
                        size
                    }else{
                        let error_clone = next_hir.clone();
                        self.emit_error("Failed to determine size of local object.".to_string(), error_clone.pos)?;
                        return Err(())
                    };
                    let prealloc_clone = next_hir.clone();
                    preallocs.push(
                        MIR{
                            pos: prealloc_clone.pos,
                            sig: prealloc_clone.sig,
                            ins: MIRInstruction::HeapAlloc(name.clone(), size.0)
                        }
                    );
                    let clone = next_hir.clone();
                    other.push(
                        MIR{
                            pos: clone.pos,
                            sig: clone.sig,
                            ins: MIRInstruction::ObjInit(name.clone(), *mutable)
                        }
                    );
                    if let Some(mir) = size.1{
                        other.push(mir)
                    }
                },
                HIRInstruction::EndFn => {
                    other.push(MIR{
                        pos: next_hir.pos,
                        sig: next_hir.sig,
                        ins: MIRInstruction::EndFun
                    });
                    break;
                },
                _ => {
                    other.push(MIR{
                        pos: next_hir.pos,
                        sig: next_hir.sig,
                        ins: MIRInstruction::from_hir(next_hir.ins)
                    });
                }
            }
        }
        for mir in header{
            self.completed_mir.push(mir)
        }
        for mir in preallocs{
            self.completed_mir.push(mir)
        }
        for mir in other{
            self.completed_mir.push(mir)
        }
        Ok(())
    }

    fn check_and_sort(&mut self) -> Result<(),()>{
        loop{
            let hir = if let Ok(Some(ir)) = self.typeck_rx.recv(){
                ir
            }else{
                break
            };
            match &hir.clone().ins{
                HIRInstruction::Module(s) => self.completed_mir.push(MIR{
                    pos: hir.pos,
                    sig: hir.sig,
                    ins: MIRInstruction::Module(s.clone())
                }),
                HIRInstruction::EndModule => self.completed_mir.push(MIR{
                    pos: hir.pos,
                    sig: hir.sig,
                    ins: MIRInstruction::EndModule
                }),
                HIRInstruction::Fn(s) => if let Err(()) = self.convert_function_block(hir){
                    return Err(())
                }
                HIRInstruction::Integer(s) => {
                    self.completed_mir.push(MIR{
                        pos: hir.pos,
                        sig: hir.sig,
                        ins: MIRInstruction::Integer(*s)
                    })
                },
                HIRInstruction::Float(f) => {
                    self.completed_mir.push(MIR{
                        pos: hir.pos,
                        sig: hir.sig,
                        ins: MIRInstruction::Float(*f)
                    })
                },
                HIRInstruction::String(s) => {
                    self.completed_mir.push(MIR{
                        pos: hir.pos,
                        sig: hir.sig,
                        ins: MIRInstruction::String(s.clone())
                    })
                },
                HIRInstruction::Bool(b) => {
                    self.completed_mir.push(MIR{
                        pos: hir.pos,
                        sig: hir.sig,
                        ins: MIRInstruction::Bool(*b)
                    })
                },
                HIRInstruction::Property(name, mutable) => {
                    let hir_clone = hir.clone();
                    let size = if let Some(size) = self.determine_alloc_size(hir_clone){
                        size
                    }else{
                        let error_clone = hir.clone();
                        self.emit_error("Failed to determine size of property.".to_string(), error_clone.pos)?;
                        return Err(())
                    };
                    let alloc_hir_clone = hir.clone();
                    self.completed_mir.push(MIR{
                        pos: alloc_hir_clone.pos,
                        sig: alloc_hir_clone.sig,
                        ins: MIRInstruction::HeapAlloc(name.clone(), size.0)
                    });
                    let obj_clone = hir.clone();
                    self.completed_mir.push(MIR{
                        pos: obj_clone.pos,
                        sig: obj_clone.sig,
                        ins: MIRInstruction::ObjInit(name.clone(), *mutable)
                    });
                    if let Some(mir) = size.1{
                        self.completed_mir.push(mir)
                    }
                },
                HIRInstruction::Halt => {
                    self.completed_mir.push(MIR{
                        pos: hir.pos,
                        sig: hir.sig,
                        ins: MIRInstruction::Halt
                    });
                    break
                }
                _ => {
                    self.emit_error(format!("Unrecognized element: {:?}", hir), hir.pos)?;
                    return Err(())
                }
            };
        }
        for ir in self.completed_mir.to_vec(){
            self.mir_tx.send(Some(ir)).unwrap();
        }
        Ok(())
    }

    pub async fn start(module_name: String, mir_tx: Sender<Option<MIR>>, notice_tx: Sender<Option<Notice>>, typeck_rx: Receiver<Option<HIR>>) -> Result<(),()>{
        let mut memmy = Self{
            module_name,
            symbol_table: Vec::new(),
            completed_mir: Vec::new(),
            mir_tx,
            notice_tx,
            typeck_rx,
        };
        if memmy.check_and_sort().is_err(){
            return Err(())
        }
        Ok(())
    }
}
