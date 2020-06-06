use crate::{
    SymbolResolver,
    Load,
};

use ir::{
    Chunk,
    hir::HIRInstruction
};

use ir_traits::ReadInstruction;

use stmt::{
    Statement,
    StatementKind,
    property::Property,
};

use notices::{
    DiagnosticLevel,
    DiagnosticSource,
    DiagnosticSourceBuilder
};

use ty::{
    Ty,
};

use mutable::Mutability;

use expr::{
    Expr,
};

use ident::Identifier;

use std::cell::RefCell;

impl Load for Property{
    type Output = Property;

    fn load(chunk: &Chunk, symbol_resolver: &SymbolResolver) -> Result<Option<Self::Output>, ()>{
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                symbol_resolver.emit_diagnostic(&[], &[diag_source]);
                return Err(())
            }
        };
        let mutable = match Mutability::load(chunk, symbol_resolver){
            Ok(Some(mutable)) => mutable,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };
        let ident = match Identifier::load(chunk, symbol_resolver){
            Ok(Some(ident)) => ident,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };

        let ty = match Ty::load(chunk, symbol_resolver){
            Ok(Some(ty)) => ty,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };

        let expr_chunk = if let Ok(Some(expr_chunk)) = symbol_resolver.ir_rx.recv(){
            expr_chunk
        }else{
            let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                .level(DiagnosticLevel::Error)
                .message(format!("Failed to get HIR chunk for expression while loading property"))
                .build();
                symbol_resolver.emit_diagnostic(&[], &[diag_source]);
            return Err(())
        };
        let expr = match Expr::load(&expr_chunk, symbol_resolver){
            Ok(Some(expr)) => expr,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };
        return Ok(Some(
            Property{
                ident,
                pos,
                ty: RefCell::new(ty),
                expr,
                mutable
            }
        ))
    }
}