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
    Notice,
    NoticeLevel
};

use core::pos::BiPos;

struct MemmyElement{
    bc_index: usize,
    count: usize,
    //
    refs: Vec<(String, usize)>
}

pub struct MemmyGenerator{
    module_name: String,
    symbol_table: Vec<MemmyElement>,
    final_chunk: Chunk,
    mir_tx: Sender<Option<Chunk>>,
    typeck_rx: Receiver<Option<Chunk>>,
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

    fn determine_alloc_size(&mut self, chunk: &mut Chunk) -> Option<(usize, Option<Chunk>)>{
        let mut ret_chunk = Chunk::new();
        let pos = chunk.read_pos();
        match FromPrimitive::from_u8(chunk.get_current()){
            Some(HIRInstruction::Float) => {
                ret_chunk.write_instruction(MIRInstructions::Float);
                Some((size_of::<f32>(), Some(ret_chunk)))
            },
            Some(HIRInstruction::Integer) => {
                ret_chunk.write_instruction(MIRInstructions::Integer);
                Some((size_of::<i32>(), Some(ret_chunk)))
            },
            Some(HIRInstruction::Bool) => 
            {
                ret_chunk.write_instruction(MIRInstructions::Bool);
                Some((size_of::<bool>(), Some(ret_chunk)))
            },
            Some(HIRInstruction::String) => {
                chunk.advance();
                let name = chunk.read_string();
                let mut new_chunk = Chunk::new();
                new_chunk.write_instruction(MIRInstructions::String);
                new_chunk.write_string(name.clone());
                Some((name.len(), Some(new_chunk)))
            },
            Some(HIRInstruction::None) => {
                self.emit_error("Found an untyped object, which is an illegal operation. This should not be happening. Please report this to the author.".to_string(), pos).unwrap();
                None
            },
            _ => {
                self.emit_error(format!("Found an unknown bytecode instruction: {:?}", chunk.get_current()).to_string(), pos).unwrap();
                None
            }
        }
    }

    fn hir_2_mir(&mut self, chunk: &mut Chunk) -> Result<Chunk, ()>{
        let pos = chunk.read_pos();
        let mut new_chunk = Chunk::new();
        chunk.advance();
        match FromPrimitive::from_u8(chunk.get_current()){
            Some(HIRInstruction::Bool) => {
                new_chunk.write_instruction(MIRInstructions::Bool);
                chunk.advance();
                let pos = chunk.read_pos();
                new_chunk.write_pos(pos);
                let value = chunk.read_bool();
                new_chunk.write_bool(value);
                chunk.advance();
                let vpos = chunk.read_pos();
                new_chunk.write_pos(vpos);
            }
            Some(HIRInstruction::Integer) => {
                new_chunk.write_instruction(MIRInstructions::Integer);
                chunk.advance();
                let pos = chunk.read_pos();
                new_chunk.write_pos(pos);
                let value = chunk.read_int();
                new_chunk.write_int(value);
                chunk.advance();
                let vpos = chunk.read_pos();
                new_chunk.write_pos(vpos);
            }
            Some(HIRInstruction::Float) => {
                new_chunk.write_instruction(MIRInstructions::Float);
                chunk.advance();
                let pos = chunk.read_pos();
                new_chunk.write_pos(pos);
                let value = chunk.read_float();
                new_chunk.write_float(value);
                chunk.advance();
                let vpos = chunk.read_pos();
                new_chunk.write_pos(vpos);
            }
            Some(HIRInstruction::String) => {
                new_chunk.write_instruction(MIRInstructions::String);
                chunk.advance();
                let pos = chunk.read_pos();
                new_chunk.write_pos(pos);
                let value = chunk.read_string();
                new_chunk.write_string(value);
                chunk.advance();
                let vpos = chunk.read_pos();
                new_chunk.write_pos(vpos);
            }
            _ => {
                self.emit_error(format!("Unexpected instruction: {}", chunk.get_current()), pos)?;
            }
        }
        Ok(new_chunk)
    }

    fn convert_function_block(&mut self, mut chunk: &mut Chunk) -> Result<(),()>{
        let mut header = Chunk::new();
        let mut preallocs = Chunk::new();
        let mut other = Chunk::new();
        if let Some(HIRInstruction::Fn) = FromPrimitive::from_u8(chunk.get_current()){
            header.write_instruction(MIRInstructions::Fun);
            chunk.advance();
            let name = chunk.read_string();
            header.write_string(name);
        }else{
            let pos = chunk.read_pos();
            self.emit_error(format!("Expected an Fn HIR instruction, instead got {}", chunk.get_current()), pos)?;
            return Err(())
        }
        let pos = chunk.read_pos();
        header.write_pos(pos);
        chunk.advance();
        let name = chunk.read_string();
        header.write_string(name);
        chunk.advance();
        loop{
            let next = &chunk.get_current();
            match FromPrimitive::from_u8(*next){
                Some(HIRInstruction::FnParam) => {
                    header.write_instruction(MIRInstructions::FunParam);
                    chunk.advance();
                    let name = chunk.read_string();
                    header.write_string(name);
                },
                Some(HIRInstruction::LocalVar) => {
                    chunk.advance();
                    //Get the position of the let keyword
                    let var_pos = chunk.read_pos();
                    chunk.advance();
                    //Get the anem
                    let name = chunk.read_string();
                    chunk.advance();
                    //Get the position of the name
                    let name_pos = chunk.read_pos();
                    //Mutable flag
                    let mutable = chunk.read_bool();
                    chunk.advance();
                    let mut_pos = chunk.read_pos();
                    //Determine the size of the object being allocated
                    let size = if let Some(size) = self.determine_alloc_size(chunk){
                        size
                    }else{
                        self.emit_error("Failed to determine size of local object.".to_string(), var_pos)?;
                        return Err(())
                    };
                    
                    preallocs.write_string(name.clone());
                    preallocs.write_pos(name_pos);
                    preallocs.write_instruction(MIRInstructions::StackAlloc);
                    preallocs.write_usize(size.0);
                    if let Some(chunk) = size.1{
                        other.write_instruction(MIRInstructions::ObjInit);
                        other.write_string(name);
                        other.write_pos(name_pos);
                        other.write_bool(mutable);
                        other.write_pos(mut_pos);
                        other.write_chunk(chunk);
                    }
                },
                Some(HIRInstruction::Property) => {
                    chunk.advance();
                    //Get the position of the let keyword
                    let var_pos = chunk.read_pos();
                    chunk.advance();
                    //Get the anem
                    let name = chunk.read_string();
                    chunk.advance();
                    //Get the position of the name
                    let name_pos = chunk.read_pos();
                    //Mutable flag
                    let mutable = chunk.read_bool();
                    chunk.advance();
                    let mut_pos = chunk.read_pos();
                    //Determine the size of the object being allocated
                    let size = if let Some(size) = self.determine_alloc_size(chunk){
                        size
                    }else{
                        self.emit_error("Failed to determine size of property object.".to_string(), var_pos)?;
                        return Err(())
                    };
                    
                    preallocs.write_string(name.clone());
                    preallocs.write_pos(name_pos);
                    preallocs.write_instruction(MIRInstructions::HeapAlloc);
                    preallocs.write_usize(size.0);
                    if let Some(chunk) = size.1{
                        other.write_instruction(MIRInstructions::ObjInit);
                        other.write_string(name);
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

    fn check_and_sort(&mut self) -> Result<(),()>{
        loop{
            let mut chunk = if let Ok(Some(chunk)) = self.typeck_rx.recv(){
                chunk
            }else{
                break
            };
            let mir = self.hir_2_mir(&mut chunk).unwrap();
            self.final_chunk.write_chunk(mir);
        }
        self.mir_tx.send(Some(self.final_chunk.clone())).unwrap();
        Ok(())
    }

    pub async fn start(module_name: String, mir_tx: Sender<Option<Chunk>>, notice_tx: Sender<Option<Notice>>, typeck_rx: Receiver<Option<Chunk>>) -> Result<(),()>{
        let mut memmy = Self{
            module_name,
            symbol_table: Vec::new(),
            mir_tx,
            notice_tx,
            typeck_rx,
            final_chunk: Chunk::new(),
        };
        if memmy.check_and_sort().is_err(){
            return Err(())
        }
        Ok(())
    }
}
