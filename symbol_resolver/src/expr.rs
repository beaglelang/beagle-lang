use super::{
    SymbolResolver,
    Load,
    ResolveSymbols,
};


use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::{
    ReadInstruction,
};

use notices::{
    DiagnosticLevel,
    DiagnosticSourceBuilder,
};

use core::pos::BiPos;

#[derive(Debug, Clone)]
pub struct Expr{
    pos: BiPos,
    kind: Box<ExprKind>,
}

#[derive(Debug, Clone)]
pub enum ExprKind{
    Binary(OpKind, Expr, Expr),
    Value(Value),
}

#[derive(Debug, Clone)]
pub enum OpKind{
    Add,
    Sub,
    Mul,
    Div
}

#[derive(Debug, Clone)]
pub enum Value{
    Integer(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Ref(String),
    FunCall(String)
}

impl ResolveSymbols for Value{
    fn resolve(&self, symbol_resolver: &SymbolResolver) -> Result<(),()>{
        
    }
}

impl ResolveSymbols for Expr{
    fn resolve(&self, symbol_resolver: &SymbolResolver) -> Result<(),()>{
        match self.kind.into(){
            ExprKind::Value(value) => {
                match value{
                    Value::Integer(_) | Value::Float(_) | Value::String(_) | Value::Bool(_) => Ok(()),
                    Value::Ref(ident) => {
                        match symbol_resolver.find_symbol(ident.clone()){
                            Some(symbol) => {
                                match symbol{
                                    Symbol::Fun(_) => {
                                        
                                    }
                                }
                                return Ok(())
                            }
                            None => {
                                return Err(())
                            }
                        }
                    }
                    Self::
                }
            }
        }
    }
}

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
                let kind = ExprKind::Value(Value::Bool(value));
                return Ok(Some(Expr{
                    kind: Box::new(kind),
                    pos
                }));
            }
            Some(HIRInstruction::Integer) => {
                let value = chunk.read_int();
                let kind = ExprKind::Value(Value::Integer(value));
                return Ok(Some(Expr{
                    kind: Box::new(kind),
                    pos
                }));
            }
            Some(HIRInstruction::Float) => {
                let value = chunk.read_float();
                let kind = ExprKind::Value(Value::Float(value));
                return Ok(Some(Expr{
                    kind: Box::new(kind),
                    pos
                }));
            }
            Some(HIRInstruction::String) => {
                let value = chunk.read_string().to_owned();
                let kind = ExprKind::Value(Value::String(value));
                return Ok(Some(Expr{
                    kind: Box::new(kind),
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
                    kind: Box::new(ExprKind::Binary(
                        OpKind::Add,
                        left.clone(),
                        right,
                    )),
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
                    kind: Box::new(ExprKind::Binary(
                        OpKind::Sub,
                        left.clone(),
                        right,
                    )),
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
                    kind: Box::new(ExprKind::Binary(
                        OpKind::Mul,
                        left.clone(),
                        right,
                    )),
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
                    kind: Box::new(ExprKind::Binary(
                        OpKind::Div,
                        left.clone(),
                        right,
                    )),
                    pos
                }))
            }
            Some(HIRInstruction::Reference) => {
                let ident = chunk.read_string();
                return Ok(Some(Expr{
                    kind: Box::new(ExprKind::Value(Value::Ref(ident.to_owned()))),
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