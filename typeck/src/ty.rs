use super::{
    Typeck,
    Load,
    Unload,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::{ WriteInstruction, ReadInstruction };

use core::pos::BiPos;

use ty::{
    Ty,
    TyValueElement,
    TyValue
};

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

pub trait Inference{
    fn infer_type(&self, typeck: &Typeck) -> Result<(),()>;
}

impl Unload for Ty{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        if self.pos != BiPos::default(){
            chunk.write_pos(self.pos);
        }
        match self.ident.clone().as_str(){
            "Int" => chunk.write_instruction(HIRInstruction::Integer),
            "Float" => chunk.write_instruction(HIRInstruction::Float),
            "Bool" => chunk.write_instruction(HIRInstruction::Bool),
            "String" => chunk.write_instruction(HIRInstruction::String),
            _ => chunk.write_instruction(HIRInstruction::Custom),
        }
        chunk.write_string(self.ident.clone());
        Ok(chunk)
    }
}

impl Load for Ty{
    type Output = Ty;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                typeck.emit_diagnostic(&[], &[diag_source]);
                return Err(())
            }
        };
        let ins = chunk.read_instruction() as Option<HIRInstruction>;
        let ident = match ins{
            Some(type_) => {
                if type_ == HIRInstruction::Custom{
                    chunk.read_string().to_string()
                }else{
                    format!("{:?}", type_)
                }
            }
            None => {
                let source = match typeck.request_source_snippet(pos){
                    Ok(source) => source,
                    Err(msg) => {
                        typeck.emit_diagnostic(&[], &[msg]);
                        return Err(())
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Expected a param type annotation but instead got none. This is a bug in the compiler."))
                    .source(source)
                    .build();
                typeck.emit_diagnostic(&[], &[diag_source]);
                return Err(())
            }
        };
        Ok(Some(Ty{
            ident,
            pos
        }))
    }
}

///A trait that provides a method called `get_ty` which is a convenience method for quickly getting an IR element's type info.
pub trait GetTy{
    fn get_ty(&self) -> &Ty;
}

impl Unload for TyValueElement{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        match self{
            TyValueElement::Bool(b) => {
                chunk.write_instruction(HIRInstruction::Bool);
                chunk.write_bool(*b);
            }
            TyValueElement::Integer(i) => {
                chunk.write_instruction(HIRInstruction::Integer);
                chunk.write_int(*i);
            }
            TyValueElement::Float(f) => {
                chunk.write_instruction(HIRInstruction::Float);
                chunk.write_float(*f);
            }
            TyValueElement::String(s) => {
                chunk.write_instruction(HIRInstruction::String);
                chunk.write_string(s.clone());
            }
            TyValueElement::Custom(name) => {
                chunk.write_instruction(HIRInstruction::Custom);
                chunk.write_string(name.clone());
            }
            TyValueElement::Unit => {
                chunk.write_instruction(HIRInstruction::Unit);
            }
        }
        Ok(chunk)
    }
}



impl Unload for TyValue{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        let tyval_chunk = match self.elem.unload(){
            Ok(chunk) => chunk,
            Err(notice) => return Err(notice)
        };
        chunk.write_chunk(tyval_chunk);
        Ok(chunk)
    }
}