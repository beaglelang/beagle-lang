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

use notices::NoticeLevel;

use stmt::{
    local::Local,
};

impl<'a> super::Check<'a> for Local{
    fn check(&self, typeck: &Typeck) -> Result<(), ()> {
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
                    typeck.emit_notice(format!(
                        "Expected an assignment of type {:?} but instead got {:?}", 
                        ty,
                        value.ty
                    ), NoticeLevel::Error, BiPos{
                        start: self.pos.start,
                        end: expr.pos.end
                    })?;
                }
            }
            ExprElement::Binary(_, left, right) => {
                if left.ty.ident != right.ty.ident{
                    let error_pos = BiPos{
                        start: left.pos.start,
                        end: right.pos.end
                    };
                    typeck.emit_notice(format!("Left hand expression of binary operation is of type {} while right hand is of type {}. This is incorrect.\n\tEither change the left to match the right or change the right to match the left.", left.ty.ident, right.ty.ident), NoticeLevel::Error, error_pos)?;
                    return Err(())
                }
            }
            _ => typeck.emit_notice(format!(
                "Compound expressions not yet implemented"
            ), NoticeLevel::Error, expr.pos)?,
        }
        Ok(())
    }
}

impl Load for Local{
    type Output = Local;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let mutable = match Mutability::load(chunk, typeck){
            Ok(mutable) => mutable,
            Err(()) => return Err(())
        };
        let ident = match Identifier::load(chunk, typeck){
            Ok(ident) => ident,
            Err(()) => return Err(())
        };

        let ty = match Ty::load(chunk, typeck){
            Ok(ty) => ty,
            Err(()) => return Err(())
        };

        let expr_chunk = if let Ok(Some(expr_chunk)) = typeck.chunk_rx.recv(){
            expr_chunk
        }else{
            typeck.emit_notice(format!("Failed to get HIR chunk for expression while loading property"), NoticeLevel::ErrorPrint, pos)?;
            return Err(())
        };
        let expr = match Expr::load(&expr_chunk, typeck){
            Ok(expr) => expr,
            Err(()) => return Err(())
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
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();

        chunk.write_instruction(HIRInstruction::LocalVar);
        chunk.write_pos(self.pos);
        let ident_chunk = match self.ident.unload(){
            Ok(chunk) => chunk,
            Err(_) => return Err(())
        };
        chunk.write_chunk(ident_chunk);
        let mut_chunk = match self.mutable.unload(){
            Ok(chunk) => chunk,
            Err(_) => return Err(())
        };
        chunk.write_chunk(mut_chunk);
        let ty = self.ty.clone().into_inner();
        let ty_chunk = match ty.unload(){
            Ok(chunk) => chunk,
            Err(_) => return Err(())
        };
        chunk.write_chunk(ty_chunk);
        let expr_chunk = match self.expr.unload(){
            Ok(chunk) => chunk,
            Err(()) => return Err(())
        };
        chunk.write_chunk(expr_chunk);
        Ok(chunk)
    }
}