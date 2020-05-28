use super::{
    Load,
    property::Property,
    fun::Fun,
    local::Local,
    MemmyGenerator
};

use ir::{ Chunk, hir::HIRInstruction };

use core::pos::BiPos;

use ir_traits::{ ReadInstruction };

#[derive(Debug, Clone)]
pub struct Statement{
    pos: BiPos,
    kind: StatementKind,
}

#[derive(Debug, Clone)]
pub enum StatementKind{
    Property(Property),
    Fun(Fun),
    Local(Local)
}

impl Load for Statement{
    type Output = Statement;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        match chunk.read_instruction(){
            Some(HIRInstruction::Fn) => {
                let fun = match Fun::load(chunk, memmy){
                    Ok(fun) => fun,
                    Err(()) => return Err(())
                };
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        memmy.emit_error(msg, BiPos::default())?;
                        return Err(())
                    }
                };
                Ok(Statement{
                    kind: StatementKind::Fun(fun),
                    pos
                })
            },
            Some(HIRInstruction::Property) => {
                let property = match Property::load(chunk, memmy){
                    Ok(fun) => fun,
                    Err(()) => return Err(())
                };
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        memmy.emit_error(msg, BiPos::default())?;
                        return Err(())
                    }
                };
                Ok(Statement{
                    kind: StatementKind::Property(property),
                    pos
                })
            },
            Some(HIRInstruction::LocalVar) => {
                let local = match Local::load(chunk, memmy){
                    Ok(local) => local,
                    Err(()) => return Err(())
                };
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        memmy.emit_error(msg, BiPos::default())?;
                        return Err(())
                    }
                };
                Ok(Statement{
                    kind: StatementKind::Local(local),
                    pos
                })
            }
            _ => {
                memmy.emit_error(format!("Unimplemented. This should only be seen in developer mode."), BiPos::default())?;
                return Err(())
            }
        }
        
    }
}