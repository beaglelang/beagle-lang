use super::{
    Typeck,
    Load,
    Unload,
    ty::{
        Inference,
        GetTy,
    }
};

use ty::{
    Ty,
};
use expr::{
    Expr,
};
use ident::Identifier;
use mutable::Mutability;

use core::pos::BiPos;

use std::cell::RefCell;

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::{
    WriteInstruction
};

use notices::{
    NoticeLevel,
    Notice
};

use stmt::{
    local::Local,
};

impl Inference for Local{
    fn infer_type(&self, typeck: &Typeck) -> Result<(),Notice> {
        let ty_inner = self.ty.clone().into_inner();
        let expr_ty = self.expr.get_ty();
        if ty_inner.ident == "Unknown"{
            self.ty.replace(Ty{
                ident: expr_ty.ident.clone(),
                pos: ty_inner.pos,
            });
            return Ok(());
        }
        if ty_inner != *expr_ty{
            return Err(Notice::new(
                format!("Local Checker"),
                format!(
                    "Expected an assignment of type {:?} but instead got {:?}", 
                    ty_inner.ident,
                    expr_ty.ident
                ),
                Some(typeck.module_name.clone()),
                Some(BiPos{
                    start: self.pos.start,
                    end: expr_ty.pos.end
                }),
                NoticeLevel::Error,
                vec![]
            ))
        }
        Ok(())
    }
}

impl<'a> super::Check<'a> for Local{
    fn check(&self, typeck: &Typeck) -> Result<(), Notice> {
        self.infer_type(typeck)?;
        Ok(())
    }
}

impl Load for Local{
    type Output = Local;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, Notice> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                return Err(Notice::new(
                    format!("Local Loader"),
                    msg,
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![]
                ))
            }
        };
        let mutable = match Mutability::load(chunk, typeck){
            Ok(mutable) => mutable,
            Err(msg) => return Err(msg)
        };
        let ident = match Identifier::load(chunk, typeck){
            Ok(ident) => ident,
            Err(msg) => return Err(msg)
        };

        let ty = match Ty::load(chunk, typeck){
            Ok(ty) => ty,
            Err(msg) => return Err(msg)
        };

        let expr_chunk = if let Ok(Some(expr_chunk)) = typeck.chunk_rx.recv(){
            expr_chunk
        }else{
            return Err(Notice::new(
                format!("Local Loader"),
                format!("Failed to get HIR chunk for expression while loading property"),
                Some(typeck.module_name.clone()),
                Some(pos),
                NoticeLevel::Error,
                vec![]
            ))
        };
        let expr = match Expr::load(&expr_chunk, typeck){
            Ok(expr) => expr,
            Err(msg) => return Err(msg)
        };
        return Ok(
            Local{
                ident,
                pos,
                ty: RefCell::new(ty),
                expr,
                mutable
            }
        )
    }
}

impl Unload for Local{
    fn unload(&self) -> Result<Chunk, Notice> {
        let mut chunk = Chunk::new();

        chunk.write_instruction(HIRInstruction::LocalVar);
        chunk.write_pos(self.pos);
        let ident_chunk = match self.ident.unload(){
            Ok(chunk) => chunk,
            Err(msg) => return Err(msg)
        };
        chunk.write_chunk(ident_chunk);
        let mut_chunk = match self.mutable.unload(){
            Ok(chunk) => chunk,
            Err(msg) => return Err(msg)
        };
        chunk.write_chunk(mut_chunk);
        let ty = self.ty.clone().into_inner();
        let ty_chunk = match ty.unload(){
            Ok(chunk) => chunk,
            Err(msg) => return Err(msg)
        };
        chunk.write_chunk(ty_chunk);
        let expr_chunk = match self.expr.unload(){
            Ok(chunk) => chunk,
            Err(msg) => return Err(msg)
        };
        chunk.write_chunk(expr_chunk);
        Ok(chunk)
    }
}