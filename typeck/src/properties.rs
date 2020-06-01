use super::{
    Typeck,
    Load,
    Unload,
    ty::{
        GetTy,
        Inference
    }
};

use ty::{
    Ty,
};

use mutable::Mutability;

use expr::{
    Expr,
};

use ident::Identifier;

use core::pos::{
    BiPos,
};

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
    Notice,
};

use stmt::{
    property::Property
};

impl Inference for Property{
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

impl Load for Property{
    type Output = Property;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, Notice> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                return Err(Notice::new(
                    format!("Property Loader"),
                    msg,
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![]
                ))
            }
        };
        let mutable = match Mutability::load(chunk, typeck){
            Ok(Some(mutable)) => mutable,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };
        let ident = match Identifier::load(chunk, typeck){
            Ok(Some(ident)) => ident,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };

        let ty = match Ty::load(chunk, typeck){
            Ok(Some(ty)) => ty,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };

        let expr_chunk = if let Ok(Some(expr_chunk)) = typeck.chunk_rx.recv(){
            expr_chunk
        }else{
            return Err(Notice::new(
                format!("Property Loader"),
                format!("Failed to get HIR chunk for expression while loading property"),
                Some(typeck.module_name.clone()),
                Some(pos),
                NoticeLevel::Error,
                vec![]
            ))
        };
        let expr = match Expr::load(&expr_chunk, typeck){
            Ok(Some(expr)) => expr,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };
        return Ok(Some(
            Property{
                ident,
                pos,
                ty: RefCell::new(ty),
                expr,
                mutable
            }
        ))
    }
}

impl<'a> super::Check<'a> for Property{
    fn check(&self, typeck: &'a Typeck) -> Result<(),Notice>{
        self.infer_type(typeck)?;
        Ok(())
    }
}

impl Unload for Property{
    fn unload(&self) -> Result<Chunk, Notice> {
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Property);
        chunk.write_pos(self.pos);
        match self.ident.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        match self.mutable.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        match self.ty.clone().into_inner().unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        match self.expr.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        Ok(chunk)
    }
}