use ir::{
    Chunk,
};

use super::{
    SymbolResolver,
    Load,
};

use notices::{
    DiagnosticLevel,
    DiagnosticSourceBuilder
};

use mutable::Mutability;

impl Load for Mutability{
    type Output = Mutability;
    fn load(chunk: &Chunk, symbol_resolver: &SymbolResolver) -> Result<Option<Self::Output>, ()>{
        let mutable = chunk.read_bool();
        let mut_pos = match chunk.read_pos(){
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
        Ok(Some(Mutability{
            mutable,
            pos: mut_pos
        }))
    }
}