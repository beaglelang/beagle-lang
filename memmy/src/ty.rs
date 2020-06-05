use crate::{
    Load,
    MemmyGenerator
};

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

use ty::Ty;

use ir::{
    Chunk,
    hir::HIRInstruction
};

use ir_traits::ReadInstruction;

impl Load for Ty{
    type Output = Ty;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                memmy.emit_diagnostic(&[], &[diagnosis]);
                return Err(())
            }
        };
        let _ins = chunk.read_instruction() as Option<HIRInstruction>;
        let ident = chunk.read_string().to_owned();
        Ok(
            Ty{
                pos,
                ident
            }
        )
    }
}