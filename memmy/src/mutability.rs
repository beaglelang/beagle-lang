use crate::{
    MemmyGenerator,
    Load,
};

use ir::{
    Chunk,
};

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

use mutable::Mutability;

impl Load for Mutability{
    type Output = Mutability;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let source = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .message(msg)
                    .level(DiagnosticLevel::Error)
                    .build();
                memmy.emit_diagnostic(&[], &[source]);
                return Err(())
            }
        };

        let mutable = chunk.read_bool();
        Ok(
            Mutability{
                mutable,
                pos
            }
        )
    }
}