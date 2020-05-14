use super::{
    expressions::{
        Expr,
        ExprElement
    },
    Mutability,
    Identifier,
    Ty,
    Typeck,
};

use core::pos::BiPos;

use std::cell::RefCell;

use ir::{
    Chunk,
    hir::HIRInstruction
};

use ir_traits::ReadInstruction;

use notices::NoticeLevel;

#[derive(Debug, Clone)]
pub struct Local{
    pub ident: Identifier,
    pub ty: RefCell<Ty>,
    pub expr: Expr,
    pub pos: BiPos,
    pub mutable: Mutability
}

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

impl super::Load for Local{
    type Output = Local;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let mutable = chunk.read_bool();
        let mut_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let name = chunk.read_string();
        let name_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };

        let type_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let type_ins: Option<HIRInstruction> = chunk.read_instruction();
        let typename = match type_ins{
            Some(type_ins) => {
                if type_ins == HIRInstruction::Custom{
                    chunk.read_string().to_owned()
                }else{
                    format!("{:?}", type_ins)
                }
            }
            None => {
                typeck.emit_notice(format!("Expected a return type instruction but instead got {:?}; this is compiler bug.", type_ins.unwrap()), NoticeLevel::Error, type_pos)?;
                return Err(())
            }
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
                ident: Identifier{
                    ident: name.to_string(),
                    pos: name_pos,
                },
                pos,
                ty: RefCell::new(Ty{
                    ident: typename,
                    pos: type_pos
                }),
                expr,
                mutable: Mutability{
                    mutable,
                    pos: mut_pos
                }
            }
        )
    }
}