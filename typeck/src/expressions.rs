use super::{
    GetTy,
    Ty,
    TyValue,
    Typeck,
    TyValueElement
};

use core::pos::BiPos;

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::ReadInstruction;

use notices::NoticeLevel;

#[derive(Debug, Clone)]
pub struct Expr{
    pub kind: Box<ExprElement>,
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
    UnaryOp(TyValue, TyValue)
}

impl GetTy for ExprElement{
    fn get_ty(&self) -> &Ty{
        match self{
            Self::Grouped(expr) => {
                expr.get_ty()
            }
            Self::Value(t) => &t.ty,
            Self::UnaryOp(left, _) => &left.ty
        }
    }
}

impl super::Load for Expr{
    type Output = Expr;

    fn load(chunk: Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let ins: Option<HIRInstruction> = chunk.read_instruction();
        let pos = chunk.read_pos();
        match &ins {
            Some(HIRInstruction::Bool) => {
                let value = chunk.read_bool();
                let kind = ExprElement::Value(TyValue{
                    ty: Ty{
                        ident: "Bool".to_owned(),
                        pos
                    },
                    elem: TyValueElement::Bool(value),
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    pos
                });
            }
            Some(HIRInstruction::Integer) => {
                let value = chunk.read_int();
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::Integer(value),
                    ty: Ty{
                        ident: "Integer".to_owned(),
                        pos
                    },
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    pos
                });
            }
            Some(HIRInstruction::Float) => {
                let value = chunk.read_float();
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::Float(value),
                    ty: Ty{
                        ident: "Float".to_owned(),
                        pos
                    },
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    pos
                });
            }
            Some(HIRInstruction::String) => {
                let value = chunk.read_string().to_owned();
                let kind = ExprElement::Value(TyValue{
                    elem: TyValueElement::String(value),
                    ty: Ty{
                        ident: "String".to_owned(),
                        pos
                    },
                });
                return Ok(Expr{
                    kind: Box::new(kind),
                    pos
                });
            }
            _ => {
                typeck.emit_notice(format!("Expected an expression but instead got instruction {:?}", ins), NoticeLevel::Error, pos).unwrap();
                return Err(());
            }
        }
    }
}