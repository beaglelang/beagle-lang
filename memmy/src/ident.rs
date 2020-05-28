use core::pos::BiPos;

use ir::{
    Chunk,
};


use super::{
    MemmyGenerator,
    Load,
};

#[derive(Debug, Clone)]
pub struct Identifier{
    pub ident: String,
    pub pos: BiPos
}

impl Load for Identifier{
    type Output = Identifier;
    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                memmy.emit_error(msg, BiPos::default())?;
                return Err(())
            }
        };
        let ident = chunk.read_string();
        return Ok(Identifier{
            ident: ident.to_owned(),
            pos
        })
    }
}