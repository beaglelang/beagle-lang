use super::{
    SymbolResolver,
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

impl Load for Expr{
    type Output = Expr;

    fn load(chunk: &Chunk, symbol_resolver: &SymbolResolver) -> Result<Option<Self::Output>, ()> {
        let ins: Option<HIRInstruction> = chunk.read_instruction();
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .message(msg)
                    .level(DiagnosticLevel::Error)
                    .build();
                symbol_resolver.emit_diagnostic(&[], &[diag_source]);
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
                let left = match Expr::load(&chunk, symbol_resolver){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                let right = match Expr::load(&chunk, symbol_resolver){
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
                let left = match Expr::load(&chunk, symbol_resolver){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                let right = match Expr::load(&chunk, symbol_resolver){
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
                let left = match Expr::load(&chunk, symbol_resolver){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                let right = match Expr::load(&chunk, symbol_resolver){
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
                let left = match Expr::load(&chunk, symbol_resolver){
                    Ok(Some(expr)) => {
                        expr
                    }
                    Ok(None) => return Ok(None),
                    Err(notice) => return Err(notice)
                };
                let right = match Expr::load(&chunk, symbol_resolver){
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
                let source = match symbol_resolver.request_source_snippet(pos){
                    Ok(source) => source,
                    Err(diag) => {
                        symbol_resolver.emit_diagnostic(&[], &[diag]);
                        return Err(())
                    }
                };
                let report = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                        .message(format!("Expected an expression but instead got instruction {:?}", ins.unwrap()))
                        .level(DiagnosticLevel::Error)
                        .source(source)
                        .range(pos.col_range())
                        .build();
                symbol_resolver.emit_diagnostic(&[], &[report]);
                return Err(())
            }
        }
    }
}