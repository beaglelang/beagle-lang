use super::{
    Load,
    ident::Identifier,
    MemmyGenerator,
    expr::Expression,
    Mutability
};

use core::pos::BiPos;

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::ReadInstruction;

#[derive(Debug, Clone)]
pub struct Property{
    ident: Identifier,
    typename: Identifier,
    pos: BiPos,
    mutable: Mutability,
    expression: Expression,
}

impl Load for Property{
    type Output = Property;
    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                memmy.emit_error(msg, BiPos::default())?;
                return Err(())
            }
        };
        let ident = match Identifier::load(chunk, memmy){
            Ok(typename) => typename,
            Err(()) => return Err(())
        };
        let mutable = chunk.read_bool();
        let mutable_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                memmy.emit_error(msg, BiPos::default())?;
                return Err(())
            }
        };

        let typename_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                memmy.emit_error(msg, BiPos::default())?;
                return Err(())
            }
        };
        match chunk.read_instruction(){
            Some(HIRInstruction::Integer) | Some(HIRInstruction::Float) | Some(HIRInstruction::String) | Some(HIRInstruction::Unit) | Some(HIRInstruction::Custom) => {},
            Some(_) => {}
            None =>{
                memmy.emit_error(format!("Attempted to read type information from typeck while loading property into memmy, found None."), BiPos::default())?;
                return Err(())
            }
        };
        let typename = chunk.read_string();

        let expression = match Expression::load(chunk, memmy){
            Ok(expr) => expr,
            Err(()) => return Err(())
        };

        return Ok(
            Property{
                ident,
                typename: Identifier{
                    ident: typename.to_owned(),
                    pos: typename_pos
                },
                pos,
                mutable: Mutability{
                    mutable,
                    pos: mutable_pos
                },
                expression,
            }
        )
    }
}