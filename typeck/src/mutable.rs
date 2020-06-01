use ir::{
    Chunk,
};

use super::{
    Typeck,
    Load,
    Unload,
};

use notices::{
    NoticeLevel,
    Notice,
};

use mutable::Mutability;

impl Load for Mutability{
    type Output = Mutability;
    fn load(chunk: &Chunk, _typeck: &Typeck) -> Result<Option<Self::Output>, Notice> {
        let mutable = chunk.read_bool();
        let mut_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                return Err(Notice::new(
                    format!("Mutability Loader"),
                    msg,
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![]
                ))
            }
        };
        Ok(Some(Mutability{
            mutable,
            pos: mut_pos
        }))
    }
}

impl Unload for Mutability{
    fn unload(&self) -> Result<Chunk, Notice> {
        let mut chunk = Chunk::new();
        chunk.write_bool(self.mutable);
        chunk.write_pos(self.pos);
        Ok(chunk)
    }
}