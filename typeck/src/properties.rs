use super::{
    Typeck,
    Load,
    Unload,
};

use ty::{
    Ty,
};

use mutable::Mutability;

use expr::{
    Expr,
    ExprElement
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

impl Load for Property{
    type Output = Property;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, Notice> {
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
                format!("Property Loader"),
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
            Property{
                ident,
                pos,
                ty: RefCell::new(ty),
                expr,
                mutable
            }
        )
    }
}

impl<'a> super::Check<'a> for Property{
    fn check(&self, typeck: &'a Typeck) -> Result<(),Notice>{
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
                if ty_inner != value.ty{
                    return Err(Notice::new(
                        format!("Local Checker"),
                        format!(
                            "Expected an assignment of type {:?} but instead got {:?}", 
                            ty_inner,
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