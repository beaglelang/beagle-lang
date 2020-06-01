use super::{
    Typeck,
    Load,
    Unload,
};

use ir::Chunk;

use ident::Identifier;
use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

impl Unload for Identifier{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_pos(self.pos);
        chunk.write_string(self.ident.clone());
        Ok(chunk)
    }
}

impl Load for Identifier{
    type Output = Identifier;
    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                typeck.emit_diagnostic(&[], &[diag_source]);
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