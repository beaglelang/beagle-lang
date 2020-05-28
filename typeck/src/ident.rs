use super::{
    Unload
};

use ir::Chunk;

use core::pos::BiPos;
///A part of an IR that contains an identifier.
#[derive(Debug, Clone)]
pub struct Identifier{
    ///The identifier
    pub ident: String,
    ///The in source location of the identifier.
    pub pos: BiPos,
}

impl Unload for Identifier{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_pos(self.pos);
        chunk.write_string(self.ident.clone());
        Ok(chunk)
    }
}