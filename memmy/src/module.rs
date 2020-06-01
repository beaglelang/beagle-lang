use super::{
    ident::Identifier,
    statements::Statement,
    Load,
    MemmyGenerator,
};

use core::pos::BiPos;

use ir::{ Chunk };

use notices::{
    DiagnosticSource,
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

#[allow(dead_code)]
pub struct Module{
    ident: Identifier,
    statements: Vec<Statement>,
    pos: BiPos,
}

impl Load for Module{
    type Output = Module;

    #[allow(unused_variables)]
    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, DiagnosticSource>{
        unimplemented!()
    }
}