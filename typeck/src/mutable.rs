use ir::{
    Chunk,
};

use super::{
    Typeck,
    Load,
    Unload,
};

use core::pos::BiPos;
use notices::NoticeLevel;

use mutable::Mutability;

impl Load for Mutability{
    type Output = Mutability;
    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let mutable = chunk.read_bool();
        let mut_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::Error, BiPos::default())?;
                return Err(())
            }
        };
        Ok(Mutability{
            mutable,
            pos: mut_pos
        })
    }
}

impl Unload for Mutability{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_bool(self.mutable);
        chunk.write_pos(self.pos);
        Ok(chunk)
    }
}