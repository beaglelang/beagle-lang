use core::pos::BiPos;

use ty::{ Ty, TyValue };

#[derive(Debug, Clone)]
pub struct Expr{
    pub kind: Box<ExprElement>,
    pub ty: Ty,
    pub pos: BiPos,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum ExprElement{
    Grouped(Expr),
    Value(TyValue),
    UnaryOp(OpKind, Expr),
    Binary(OpKind, Expr, Expr),
}

#[derive(Debug, Clone)]
pub enum OpKind{
    Add,
    Min,
    Mul,
    Div,
}