use ir::{
    Chunk,
};

use super::{
    Typeck,
    Load,
    Unload,
};

use notices::{
    DiagnosticLevel,
    DiagnosticSourceBuilder
};

use mutable::Mutability;

impl Load for Mutability{
    type Output = Mutability;
    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, ()> {
        let mutable = chunk.read_bool();
        let mut_pos = match chunk.read_pos(){
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
        Ok(Some(Mutability{
            mutable,
            pos: mut_pos
        }))
    }
}

impl Unload for Mutability{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_pos(self.pos);
        chunk.write_bool(self.mutable);
        Ok(chunk)
    }
}