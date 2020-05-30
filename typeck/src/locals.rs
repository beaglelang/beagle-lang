use super::{
    Typeck,
    Load,
    Unload,
};

use ty::Ty;
use expr::{
    Expr,
    ExprElement    
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

impl<'a> super::Check<'a> for Local{
    fn check(&self, typeck: &Typeck) -> Result<(), Notice> {
        let ty = self.ty.clone();
        let expr = &self.expr;
        match expr.kind.as_ref(){
            ExprElement::Value(value) => {
                let ty_inner = ty.clone().into_inner();
                if ty_inner.ident == "Unknown"{
                    self.ty.replace(Ty{
                        ident: value.ty.ident.clone(),
                        pos: ty_inner.pos,
                    });
                    return Ok(());
                }
                if ty.clone().into_inner() != value.ty{
                    return Err(Notice::new(
                        format!("Local Checker"),
                        format!(
                            "Expected an assignment of type {:?} but instead got {:?}", 
                            ty,
                            value.ty
                        ),
                        Some(typeck.module_name.clone()),
                        Some(BiPos{
                            start: self.pos.start,
                            end: expr.pos.end
                        }),
                        NoticeLevel::Error,
                        vec![]
                    ))
                }
            }
            ExprElement::Binary(_, left, right) => {
                if left.ty.ident != right.ty.ident{
                    let error_pos = BiPos{
                        start: left.pos.start,
                        end: right.pos.end
                    };
                    return Err(Notice::new(
                        format!("Local Checker"),
                        format!("Left hand expression of binary operation is of type {} while right hand is of type {}. This is incorrect.\n\tEither change the left to match the right or change the right to match the left.", left.ty.ident, right.ty.ident),
                        Some(typeck.module_name.clone()),
                        Some(error_pos),
                        NoticeLevel::Error,
                        vec![]
                    ))
                }
            }
            _ => return Err(Notice::new(
                format!("Local Checker"),
                format!("Compound expressions not implemented yet."),
                Some(typeck.module_name.clone()),
                Some(expr.pos),
                NoticeLevel::Error,
                vec![]
            )),
        }
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