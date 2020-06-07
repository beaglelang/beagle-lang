use super::{
    SymbolResolver,
    Load,
};

use ir::Chunk;

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

use core::pos::BiPos;

#[derive(Debug, Clone)]
pub struct Identifier{
    pub ident: String,
    pub pos: BiPos
}

impl Load for Identifier{
    type Output = Identifier;
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
        let ident = chunk.read_string();
        Ok(Some(Self{
            ident: ident.to_string(),
            pos,
        }))
    }
}