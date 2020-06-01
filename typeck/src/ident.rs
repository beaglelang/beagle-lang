use super::{
    Typeck,
    Load,
    Unload,
};

use ir::Chunk;

use ident::Identifier;
use notices::{
    NoticeLevel,
    Notice,
};

impl Unload for Identifier{
    fn unload(&self) -> Result<Chunk, Notice> {
        let mut chunk = Chunk::new();
        chunk.write_pos(self.pos);
        chunk.write_string(self.ident.clone());
        Ok(chunk)
    }
}

impl Load for Identifier{
    type Output = Identifier;
    fn load(chunk: &Chunk, _typeck: &Typeck) -> Result<Option<Self::Output>, Notice> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                return Err(Notice::new(
                    format!("Identifier Loader"),
                    msg,
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![]
                ))
            }
        };
        let ident = chunk.read_string();
        Ok(Some(Self{
            ident: ident.to_string(),
            pos,
        }))
    }
}