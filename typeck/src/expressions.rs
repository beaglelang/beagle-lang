use super::{
    GetTy,
    Ty,
    TyValue,
    Typeck,
    TyValueElement,
    Unload,
};

use core::pos::BiPos;

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::{
    ReadInstruction,
    WriteInstruction
};

use notices::NoticeLevel;

#[derive(Debug, Clone)]
pub struct Expr{
    pub kind: Box<ExprElement>,
    pub ty: Ty,
    pub pos: BiPos,
}

impl GetTy for Expr{
    fn get_ty(&self) -> &Ty {
        self.kind.get_ty()
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ExprElement{
    Grouped(Expr),
    Value(TyValue),
    UnaryOp(OpKind, Expr),
    Binary(OpKind, Expr, Expr)
}

impl Unload for ExprElement{
    fn unload(&self) -> Result<Chunk, ()> {
        match &self{
            ExprElement::Grouped(expr) => expr.unload(),
            ExprElement::Value(ty_val) => ty_val.unload(),
            ExprElement::UnaryOp(kind, expr) => {
                let kind_chunk = match kind.unload(){
                    Ok(chunk) => chunk,
                    Err(()) => return Err(())
                };
                let expr_chunk = match expr.unload(){
                    Ok(chunk) => chunk,
                    Err(()) => return Err(())
                };
                let mut chunk = Chunk::new();
                chunk.write_chunk(kind_chunk);
                chunk.write_chunk(expr_chunk);
                Ok(chunk)
            },
            ExprElement::Binary(kind, left, right) => {
                let mut chunk = Chunk::new();
                let kind_chunk = match kind.unload(){
                    Ok(chunk) => chunk,
                    Err(()) => return Err(())
                };
                chunk.write_chunk(kind_chunk);
                let left_chunk = match left.unload(){
                    Ok(chunk) => chunk,
                    Err(()) => return Err(())
                };
                chunk.write_chunk(left_chunk);
                let right_chunk = match right.unload(){
                    Ok(chunk) => chunk,
                    Err(()) => return Err(())
                };
                chunk.write_chunk(right_chunk);
                Ok(chunk)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum OpKind{
    Add,
    Min,
    Mul,
    Div,
}

impl Unload for OpKind{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        match self{
            OpKind::Add => chunk.write_instruction(HIRInstruction::Add),
            OpKind::Min => chunk.write_instruction(HIRInstruction::Sub),
            OpKind::Div => chunk.write_instruction(HIRInstruction::Div),
            OpKind::Mul => chunk.write_instruction(HIRInstruction::Mult)
        }
        Ok(chunk)
    }
}

impl GetTy for ExprElement{
    fn get_ty(&self) -> &Ty{
        match self{
            Self::Grouped(expr) => {
                expr.get_ty()
            }
            Self::Value(t) => &t.ty,
            Self::UnaryOp(_, left) => &left.ty,
            Self::Binary(_, left, _) => &left.ty
        }
    }
}

impl super::Load for Expr{
    type Output = Expr;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let ins: Option<HIRInstruction> = chunk.read_instruction();
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        match &ins {
            Some(HIRInstruction::Bool) => {
                let value = chunk.read_bool();
                let ty = Ty{
                    ident: "Bool".to_owned(),
                    pos
                };
                let kind = ExprElement::Value(TyValue{
                    ty: ty.clone(),
                    elem: TyValueElement::Bool(value),
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    ty,
                    pos
                });
            }
            Some(HIRInstruction::Integer) => {
                let value = chunk.read_int();
                let ty = Ty{
                    ident: "Integer".to_owned(),
                    pos
                };
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::Integer(value),
                    ty: ty.clone(),
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    ty,
                    pos
                });
            }
            Some(HIRInstruction::Float) => {
                let value = chunk.read_float();
                let ty = Ty{
                    ident: "Float".to_owned(),
                    pos
                };
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::Float(value),
                    ty: ty.clone(),
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    ty,
                    pos
                });
            }
            Some(HIRInstruction::String) => {
                let value = chunk.read_string().to_owned();
                let ty = Ty{
                    ident: "String".to_owned(),
                    pos
                };
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::String(value),
                    ty: ty.clone(),
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    ty,
                    pos
                });
            }
            Some(HIRInstruction::Add) => {
                let left = match Expr::load(&chunk, typeck){
                    Ok(expr) => {
                        expr
                    }
                    Err(()) => return Err(())
                };
                let right = match Expr::load(&chunk, typeck){
                    Ok(expr) => {
                        expr
                    }
                    Err(()) => return Err(())
                };
                return Ok(Expr{
                    kind: Box::new(ExprElement::Binary(
                        OpKind::Add,
                        left.clone(),
                        right,
                    )),
                    ty: left.ty,
                    pos
                })
            },
            Some(HIRInstruction::Sub) => {
                let left = match Expr::load(&chunk, typeck){
                    Ok(expr) => {
                        expr
                    }
                    Err(()) => return Err(())
                };
                let right = match Expr::load(&chunk, typeck){
                    Ok(expr) => {
                        expr
                    }
                    Err(()) => return Err(())
                };
                return Ok(Expr{
                    kind: Box::new(ExprElement::Binary(
                        OpKind::Min,
                        left.clone(),
                        right,
                    )),
                    ty: left.ty,
                    pos
                })
            }
            Some(HIRInstruction::Mult) => {
                let left = match Expr::load(&chunk, typeck){
                    Ok(expr) => {
                        expr
                    }
                    Err(()) => return Err(())
                };
                let right = match Expr::load(&chunk, typeck){
                    Ok(expr) => {
                        expr
                    }
                    Err(()) => return Err(())
                };
                return Ok(Expr{
                    kind: Box::new(ExprElement::Binary(
                        OpKind::Mul,
                        left.clone(),
                        right,
                    )),
                    ty: left.ty,
                    pos
                })
            }
            Some(HIRInstruction::Div) => {
                let left = match Expr::load(&chunk, typeck){
                    Ok(expr) => {
                        expr
                    }
                    Err(()) => return Err(())
                };
                let right = match Expr::load(&chunk, typeck){
                    Ok(expr) => {
                        expr
                    }
                    Err(()) => return Err(())
                };
                return Ok(Expr{
                    kind: Box::new(ExprElement::Binary(
                        OpKind::Div,
                        left.clone(),
                        right,
                    )),
                    ty: left.ty,
                    pos
                })
            }
            
            _ => {
                typeck.emit_notice(format!("Expected an expression but instead got instruction {:?}", ins.unwrap()), NoticeLevel::Error, pos).unwrap();
                return Err(());
            }
        }
    }
}

impl Unload for Expr{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        match self.kind.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(()) => return Err(())
        }
        match self.ty.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(()) => return Err(())
        }
        chunk.write_pos(self.pos);
        Ok(chunk)
    }
}