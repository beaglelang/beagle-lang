use ir::{
    hir::{
        HIRInstruction
    },
    mir::{
        MIRInstructions,
    },
    Chunk,
};

use num_traits::FromPrimitive;

#[allow(unused_imports)]
use ir_traits::{
    WriteInstruction,
    ReadInstruction,
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
    Diagnostic,
    DiagnosticBuilder,
    DiagnosticLevel,
    DiagnosticSource,
    DiagnosticSourceBuilder
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
    ///A global copy of a notice sender channel that all typeck's are given clones of.
    notice_tx: Sender<Option<Diagnostic>>,
}

impl MemmyManager{
    ///Create a new memmy manager with the given notice sender channel.
    pub fn new(notice_tx: Sender<Option<Diagnostic>>) -> Self{
        MemmyManager{
            thread_pool: ThreadPool::new().unwrap(),
            notice_tx,
        }
    }

    ///Enqueue a module for being type checked in parallel to other stages. See [Driver] for more info.
    ///This will spawn a new task/thread in thread_pool which executes [Typeck::start_checking].
    pub fn enqueue_module(&self, module_name: String, typeck_rx: Receiver<Option<Chunk>>, mir_tx: Sender<Option<Chunk>>, master_tx: Sender<ModuleMessage>, master_rx: Receiver<ModuleMessage>){
        let notice_tx_clone = self.notice_tx.clone();
        let module_name_clone = module_name.clone();
        self.thread_pool.spawn_ok(async move{
            let typeck = MemmyGenerator::start(module_name_clone.clone(), mir_tx, notice_tx_clone.clone(), typeck_rx, master_tx, master_rx);
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
    master_rx: Receiver<ModuleMessage>,
}

impl MemmyGenerator{

    #[allow(dead_code)]
    fn determine_alloc_size(&mut self, chunk: &mut Chunk) -> Result<(usize, Option<Chunk>), DiagnosticSource>{
        let mut ret_chunk = Chunk::new();
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                    .message(msg)
                    .build();
                return Err(diagnosis)
            }
        };
        match FromPrimitive::from_u8(chunk.get_current()){
            Some(HIRInstruction::Float) => {
                ret_chunk.write_instruction(MIRInstructions::Float);
                Ok((size_of::<f32>(), Some(ret_chunk)))
            },
            Some(HIRInstruction::Integer) => {
                ret_chunk.write_instruction(MIRInstructions::Integer);
                Ok((size_of::<i32>(), Some(ret_chunk)))
            },
            Some(HIRInstruction::Bool) => 
            {
                ret_chunk.write_instruction(MIRInstructions::Bool);
                Ok((size_of::<bool>(), Some(ret_chunk)))
            },
            Some(HIRInstruction::String) => {
                chunk.advance();
                let name = chunk.read_string();
                let mut new_chunk = Chunk::new();
                new_chunk.write_instruction(MIRInstructions::String);
                new_chunk.write_str(name.clone());
                Ok((name.len(), Some(new_chunk)))
            },
            Some(HIRInstruction::None) => {
                if let Err(_) = self.master_tx.send(ModuleMessage::SourceRequest(pos)){
                    return Err(DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                        .level(DiagnosticLevel::Error)
                        .message(format!("The master channel was closed??"))
                        .build());
                }
                let source_snip = match self.master_rx.recv(){
                    Ok(ModuleMessage::SourceResponse(source_snip)) => source_snip,
                    Ok(thing) => return Err(DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Not sure what we got but we shouldn't have: {:?}", thing))
                    .build()),
                    Err(_) => {
                        return Err(DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                            .level(DiagnosticLevel::Error)
                            .message(format!("The master channel was closed??"))
                            .build());
                    }
                };
                let diagnostic_source = DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                    .range(pos.col_range())
                    .message("Found an untyped object, which is an illegal operation. This should not be happening. Please report this to the author.".to_string())
                    .source(source_snip)
                    .level(DiagnosticLevel::Error)
                    .build();
                Err(diagnostic_source)
            },
            _ => {
                if let Err(_) = self.master_tx.send(ModuleMessage::SourceRequest(pos)){
                    return Err(DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                        .level(DiagnosticLevel::Error)
                        .message(format!("The master channel was closed??"))
                        .build());
                }
                let source_snip = match self.master_rx.recv(){
                    Ok(ModuleMessage::SourceResponse(source_snip)) => source_snip,
                    Ok(thing) => return Err(DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Not sure what we got but we shouldn't have: {:?}", thing))
                    .build()),
                    Err(_) => {
                        return Err(DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                            .level(DiagnosticLevel::Error)
                            .message(format!("The master channel was closed??"))
                            .build());
                    }
                };
                let diagnosticSource = DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                    .range(pos.col_range())
                    .message("Found an untyped object, which is an illegal operation. This should not be happening. Please report this to the author.".to_string())
                    .source(source_snip)
                    .level(DiagnosticLevel::Error)
                    .build();
                Err(diagnosticSource)
            }
        }
    }

    fn hir_2_mir(&mut self, chunk: &mut Chunk) -> Result<Chunk, DiagnosticSource>{
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                    .message(msg)
                    .build();
                return Err(diagnosis)
            }
        };
        let mut new_chunk = Chunk::new();
        chunk.advance();
        match FromPrimitive::from_u8(chunk.get_current()){
            Some(HIRInstruction::Bool) => {
                new_chunk.write_instruction(MIRInstructions::Bool);
                chunk.advance();
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                            .message(msg)
                            .build();
                        return Err(diagnosis)
                    }
                };
                new_chunk.write_pos(pos);
                let value = chunk.read_bool();
                new_chunk.write_bool(value);
                chunk.advance();
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                            .message(msg)
                            .build();
                        return Err(diagnosis)
                    }
                };
                new_chunk.write_pos(pos);
            }
            Some(HIRInstruction::Integer) => {
                new_chunk.write_instruction(MIRInstructions::Integer);
                chunk.advance();
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                            .message(msg)
                            .build();
                        return Err(diagnosis)
                    }
                };
                new_chunk.write_pos(pos);
                let value = chunk.read_int();
                new_chunk.write_int(value);
                chunk.advance();
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                            .message(msg)
                            .build();
                        return Err(diagnosis)
                    }
                };
                new_chunk.write_pos(pos);
            }
            Some(HIRInstruction::Float) => {
                new_chunk.write_instruction(MIRInstructions::Float);
                chunk.advance();
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                            .message(msg)
                            .build();
                        return Err(diagnosis)
                    }
                };
                new_chunk.write_pos(pos);
                let value = chunk.read_float();
                new_chunk.write_float(value);
                chunk.advance();
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                            .message(msg)
                            .build();
                        return Err(diagnosis)
                    }
                };
                new_chunk.write_pos(pos);
            }
            Some(HIRInstruction::String) => {
                new_chunk.write_instruction(MIRInstructions::String);
                chunk.advance();
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                            .message(msg)
                            .build();
                        return Err(diagnosis)
                    }
                };
                new_chunk.write_pos(pos);
                let value = chunk.read_string();
                new_chunk.write_str(value);
                chunk.advance();
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                            .message(msg)
                            .build();
                        return Err(diagnosis)
                    }
                };
                new_chunk.write_pos(pos);
            }
            _ => {
                let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                            .message(format!("Unexpected instruction: {}", chunk.get_current()))
                            .level(DiagnosticLevel::Error)
                            .range(pos.col_range())
                            .build();
                return Err(diagnosis)
            }
        }
        Ok(new_chunk)
    }

    #[allow(dead_code)]
    fn convert_function_block(&mut self, mut chunk: &mut Chunk) -> Result<(),DiagnosticSource>{
        let mut header = Chunk::new();
        let mut preallocs = Chunk::new();
        let mut other = Chunk::new();
        if let Some(HIRInstruction::Fn) = FromPrimitive::from_u8(chunk.get_current()){
            header.write_instruction(MIRInstructions::Fun);
            chunk.advance();
            let name = chunk.read_string();
            header.write_str(name);
        }else{
            let pos = match chunk.read_pos(){
                Ok(pos) => pos,
                Err(msg) => {
                    let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                        .message(msg)
                        .build();
                    return Err(diagnosis)
                }
            };
            let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), pos.start.0)
                            .message(format!("Expected an Fn HIR instruction, instead got {}", chunk.get_current()))
                            .level(DiagnosticLevel::Error)
                            .range(pos.col_range())
                            .build();
            return Err(diagnosis)
        }
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                    .message(msg)
                    .build();
                return Err(diagnosis)
            }
        };
        header.write_pos(pos);
        chunk.advance();
        let name = chunk.read_string();
        header.write_str(name);
        chunk.advance();
        loop{
            let next = &chunk.get_current();
            match FromPrimitive::from_u8(*next){
                Some(HIRInstruction::FnParam) => {
                    header.write_instruction(MIRInstructions::FunParam);
                    chunk.advance();
                    let name = chunk.read_string();
                    header.write_str(name);
                },
                Some(HIRInstruction::LocalVar) => {
                    chunk.advance();
                    //Get the position of the let keyword
                    let var_pos = match chunk.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                                .message(msg)
                                .build();
                            return Err(diagnosis)
                        }
                    };
                    chunk.advance();
                    //Get the anem
                    let name = chunk.read_string().to_owned();
                    chunk.advance();
                    //Get the position of the name
                    let name_pos = match chunk.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                                .message(msg)
                                .build();
                            return Err(diagnosis)
                        }
                    };
                    //Mutable flag
                    let mutable = chunk.read_bool();
                    chunk.advance();
                    let mut_pos = match chunk.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                                .message(msg)
                                .build();
                            return Err(diagnosis)
                        }
                    };
                    //Determine the size of the object being allocated
                    let size = match self.determine_alloc_size(chunk){
                        Ok(size) => size,
                        Err(diag) => {
                            return Err(diag)
                        }
                    };
                    
                    preallocs.write_str(name.clone().as_str());
                    preallocs.write_pos(name_pos);
                    preallocs.write_instruction(MIRInstructions::StackAlloc);
                    preallocs.write_usize(size.0);
                    if let Some(chunk) = size.1{
                        other.write_instruction(MIRInstructions::ObjInit);
                        other.write_str(name.as_str());
                        other.write_pos(name_pos);
                        other.write_bool(mutable);
                        other.write_pos(mut_pos);
                        other.write_chunk(chunk);
                    }
                },
                Some(HIRInstruction::Property) => {
                    chunk.advance();
                    //Get the position of the let keyword
                    let var_pos = match chunk.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                                .message(msg)
                                .build();
                            return Err(diagnosis)
                        }
                    };
                    chunk.advance();
                    //Get the anem
                    let name = chunk.read_string().to_owned();
                    chunk.advance();
                    //Get the position of the name
                    let name_pos = match chunk.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                                .message(msg)
                                .build();
                            return Err(diagnosis)
                        }
                    };
                    //Mutable flag
                    let mutable = chunk.read_bool();
                    chunk.advance();
                    let mut_pos = match chunk.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            let diagnosis = DiagnosticSourceBuilder::new(self.module_name.clone(), 0)
                                .message(msg)
                                .build();
                            return Err(diagnosis)
                        }
                    };
                    //Determine the size of the object being allocated
                    let size = match self.determine_alloc_size(chunk){
                        Ok(size) => size,
                        Err(diag) => {
                            return Err(diag)
                        }
                    };
                    
                    preallocs.write_str(name.clone().as_str());
                    preallocs.write_pos(name_pos);
                    preallocs.write_instruction(MIRInstructions::HeapAlloc);
                    preallocs.write_usize(size.0);
                    if let Some(chunk) = size.1{
                        other.write_instruction(MIRInstructions::ObjInit);
                        other.write_str(name.as_str());
                        preallocs.write_pos(name_pos);
                        other.write_bool(mutable);
                        preallocs.write_pos(mut_pos);
                        other.write_chunk(chunk);
                    }
                },
                Some(HIRInstruction::EndFn) => {
                    other.write_instruction(MIRInstructions::EndFun);
                    break;
                },
                _ => {
                    self.hir_2_mir(&mut chunk)?;
                }
            }
        }
        self.final_chunk.write_chunk(header);
        self.final_chunk.write_chunk(preallocs);
        self.final_chunk.write_chunk(other);
        Ok(())
    }

    #[allow(dead_code)]
    fn check_and_sort(&mut self) -> Result<(),DiagnosticSource>{
        loop{
            let mut chunk = if let Ok(Some(chunk)) = self.typeck_rx.recv(){
                chunk
            }else{
                break
            };
            let mir = match self.hir_2_mir(&mut chunk){
                Ok(mir) => mir,
                Err(diag) => {
                    return Err(diag)
                }
            };
            self.final_chunk.write_chunk(mir);
        }
        self.mir_tx.send(Some(self.final_chunk.clone())).unwrap();
        Ok(())
    }

    pub fn start(module_name: String, mir_tx: Sender<Option<Chunk>>, notice_tx: Sender<Option<Diagnostic>>, typeck_rx: Receiver<Option<Chunk>>, master_tx: Sender<ModuleMessage>, master_rx: Receiver<ModuleMessage>) -> Result<(),()>{
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
