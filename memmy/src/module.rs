use super::{
    ident::Identifier,
    statements::Statement,
    Load,
    MemmyGenerator,
};

use core::pos::BiPos;

use ir::{ Chunk };

#[allow(dead_code)]
pub struct Module<'a>{
    ident: Identifier,
    statements: Vec<Statement<'a>>,
    pos: BiPos,
}

impl<'a> Load for Module<'a>{
    type Output = Module<'a>;

    #[allow(unused_variables)]
    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()>{
        unimplemented!()
    }
}