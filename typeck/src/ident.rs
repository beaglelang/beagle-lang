use super::{
    Typeck,
    Load,
    Unload,
};

use ir::Chunk;

use ident::Identifier;
use notices::NoticeLevel;
use core::pos::BiPos;

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
    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::Error, BiPos::default())?;
                return Err(())
            }
        };
        let ident = chunk.read_string();
        Ok(Self{
            ident: ident.to_string(),
            pos,
        })
    }
}