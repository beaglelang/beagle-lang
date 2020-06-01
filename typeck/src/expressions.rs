use super::{
    ty::GetTy,
    Typeck,
    Load,
    Unload,
};

use ty::{ Ty, TyValue, TyValueElement };

use expr::{ Expr, ExprElement, OpKind };

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::{
    ReadInstruction,
    WriteInstruction
};

use notices::{
    DiagnosticLevel,
    DiagnosticSourceBuilder,
};


impl GetTy for Expr{
    fn get_ty(&self) -> &Ty {
        self.kind.get_ty()
    }
}



impl Unload for Expr{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_pos(self.pos);
        match self.kind.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        
        Ok(chunk)
    }
}

impl Unload for ExprElement{
    fn unload(&self) -> Result<Chunk, ()> {
        match &self{
            ExprElement::Grouped(expr) => expr.unload(),
            ExprElement::Value(ty_val) => ty_val.unload(),
            ExprElement::UnaryOp(kind, expr) => {
                let kind_chunk = match kind.unload(){
                    Ok(chunk) => chunk,
                    Err(notice) => return Err(notice)
                };
                let expr_chunk = match expr.unload(){
                    Ok(chunk) => chunk,
                    Err(notice) => return Err(notice)
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
                    Err(notice) => return Err(notice)
                };
                chunk.write_chunk(kind_chunk);
                let left_chunk = match left.unload(){
                    Ok(chunk) => chunk,
                    Err(notice) => return Err(notice)
                };
                chunk.write_chunk(left_chunk);
                let right_chunk = match right.unload(){
                    Ok(chunk) => chunk,
                    Err(notice) => return Err(notice)
                };
                chunk.write_chunk(right_chunk);
                Ok(chunk)
            }
        }
    }
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

impl Load for Expr{
    type Output = Expr;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, ()> {
        let ins: Option<HIRInstruction> = chunk.read_instruction();
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .message(msg)
                    .level(DiagnosticLevel::Error)
                    .build();
                typeck.emit_diagnostic(&[], &[diag_source]);
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
                return Ok(Some(Expr{
                    kind: Box::new(kind),
                    ty,
                    pos
                }));
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
                return Ok(Some(Expr{
                    kind: Box::new(kind),
                    ty,
                    pos
                }));
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
                return Ok(Some(Expr{
                    kind: Box::new(kind),
                    ty,
                    pos
                }));
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
                return Ok(Some(Expr{
                    kind: Box::new(kind),
                    ty,
                    pos
                }));
            }
            Some(HIRInstruction::Add) => {
                let left = match Expr::load(&chunk, typeck){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                let right = match Expr::load(&chunk, typeck){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                return Ok(Some(Expr{
                    kind: Box::new(ExprElement::Binary(
                        OpKind::Add,
                        left.clone(),
                        right,
                    )),
                    ty: left.ty,
                    pos
                }))
            },
            Some(HIRInstruction::Sub) => {
                let left = match Expr::load(&chunk, typeck){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                let right = match Expr::load(&chunk, typeck){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                return Ok(Some(Expr{
                    kind: Box::new(ExprElement::Binary(
                        OpKind::Add,
                        left.clone(),
                        right,
                    )),
                    ty: left.ty,
                    pos
                }))
            }
            Some(HIRInstruction::Mult) => {
                let left = match Expr::load(&chunk, typeck){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                let right = match Expr::load(&chunk, typeck){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                return Ok(Some(Expr{
                    kind: Box::new(ExprElement::Binary(
                        OpKind::Add,
                        left.clone(),
                        right,
                    )),
                    ty: left.ty,
                    pos
                }))
            }
            Some(HIRInstruction::Div) => {
                let left = match Expr::load(&chunk, typeck){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                let right = match Expr::load(&chunk, typeck){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                return Ok(Some(Expr{
                    kind: Box::new(ExprElement::Binary(
                        OpKind::Add,
                        left.clone(),
                        right,
                    )),
                    ty: left.ty,
                    pos
                }))
            }
            
            _ => {
                let source = match typeck.request_source_snippet(pos){
                    Ok(source) => source,
                    Err(diag) => {
                        typeck.emit_diagnostic(&[], &[diag]);
                        return Err(())
                    }
                };
                let report = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                        .message(format!("Expected an expression but instead got instruction {:?}", ins.unwrap()))
                        .level(DiagnosticLevel::Error)
                        .source(source)
                        .range(pos.col_range())
                        .build();
                typeck.emit_diagnostic(&[], &[report]);
                return Err(())
            }
        }
    }
}

