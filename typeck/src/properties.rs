use super::{
    Identifier,
    expressions::{
        Expr,
        ExprElement
    },
    Mutability,
    Ty,
    Typeck,
};

use core::pos::{
    BiPos,
};

use std::cell::RefCell;

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::ReadInstruction;

use notices::NoticeLevel;

#[derive(Debug, Clone)]
pub struct Property{
    pub ident: Identifier,
    pub ty: RefCell<Ty>,
    pub expr: Expr,
    pub pos: BiPos,
    pub mutable: Mutability,
}

impl super::Load for Property{
    type Output = Property;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let mutable = chunk.read_bool();
        let name = chunk.read_string().to_string();
        let name_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let current_type: HIRInstruction = chunk.read_instruction().unwrap();
        let typename = if current_type == HIRInstruction::Custom{
            let typename = chunk.read_string().to_owned();
            Some(typename)
        }else{
            Some(format!("{:?}", current_type).to_string())
        };
        let expr_chunk = if let Ok(Some(expr_chunk)) = typeck.chunk_rx.recv(){
            expr_chunk
        }else{
            typeck.emit_notice(format!("Failed to get HIR chunk for expression while loading property"), NoticeLevel::Error, pos)?;
            return Err(())
        };
        
        let expr = match Expr::load(&expr_chunk, typeck){
            Ok(expr) => expr,
            Err(()) => return Err(())
        };
        let property = Property{
            ident: Identifier{
                ident: name.clone(),
                pos: name_pos,
            },
            ty: RefCell::new(Ty{
                ident: if typename.is_some(){
                    typename.unwrap()
                }else{
                    match &current_type{
                        HIRInstruction::Integer => "Int".to_owned(),
                        HIRInstruction::String => "String".to_owned(),
                        HIRInstruction::Float => "Float".to_owned(),
                        HIRInstruction::Bool => "Bool".to_owned(),
                        HIRInstruction::Unknown => "Unknown".to_owned(),
                        _ => {
                            typeck.emit_notice(format!("Unrecognized type element; this is a bug in the compiler: {:?}", current_type), NoticeLevel::Error, pos).unwrap();
                            return Err(())
                        },
                    }
                },
                pos: pos.clone(),
            }),
            expr,
            pos,
            mutable: Mutability{
                mutable,
                pos,
            }
        };
        Ok(property)
    }
}

impl<'a> super::Check<'a> for Property{
    fn check(&self, typeck: &'a Typeck) -> Result<(),()>{
        let ty = &self.ty;
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
                if ty.clone().into_inner().ident != value.ty.ident{
                    typeck.emit_notice(format!(
                        "Expect an assignment of type {} but instead got {}", 
                        ty.clone().into_inner().ident,
                        value.ty.ident
                    ), NoticeLevel::Error, BiPos{
                        start: self.pos.start,
                        end: expr.pos.end
                    })?;
                    return Err(())
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