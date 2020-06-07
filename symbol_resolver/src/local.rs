use super::{
    SymbolResolver,
    Load,
    ident::Identifier,
    expr::Expr
};

use mutable::Mutability;

use ir::{
    Chunk,
};

use notices::{
    DiagnosticLevel,
    DiagnosticSourceBuilder,
};

use core::pos::BiPos;

#[derive(Debug, Clone)]
pub struct Local{
    pub ident: Identifier,
    pub ty: Identifier,
    pub expr: Expr,
    pub mutable: Mutability,
    pub pos: BiPos
}

impl Load for Local{
    type Output = Local;

    fn load(chunk: &Chunk, symbol_resolver: &SymbolResolver) -> Result<Option<Self::Output>, ()> {
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

        let ty = match Identifier::load(chunk, symbol_resolver){
            Ok(Some(ty)) => ty,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };

        let expr_chunk = if let Ok(Some(expr_chunk)) = symbol_resolver.ir_rx.recv(){
            expr_chunk
        }else{
            return Ok(None)
        };
        let expr = match Expr::load(&expr_chunk, symbol_resolver){
            Ok(Some(expr)) => expr,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };
        return Ok(Some(
            Local{
                ident,
                pos,
                ty,
                expr,
                mutable
            }
        ))
    }
}