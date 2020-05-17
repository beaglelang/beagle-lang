use super::{
    properties::Property,
    fun::Fun,
    locals::Local,
};

use core::pos::BiPos;

use ir::{
    Chunk,
    hir::HIRInstruction
};

use super::Typeck;
use ir_traits::ReadInstruction;
use notices::NoticeLevel;

#[derive(Debug, Clone)]
pub struct Statement{
    pub kind: StatementKind,
    pub pos: BiPos,
}

#[derive(Debug, Clone)]
pub enum StatementKind{
    Property(Property),
    Fun(Fun),
    Local(Local),
}

impl super::Load for Statement{
    type Output = Statement;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        match chunk.read_instruction(){
            Some(HIRInstruction::Property) => match Property::load(chunk, typeck){
                Ok(property) => {
                    Ok(Statement{
                        kind: StatementKind::Property(property.clone()),
                        pos: property.pos.clone()
                    })
                },
                Err(()) => Err(())
            },
            Some(HIRInstruction::Fn) => match Fun::load(chunk, typeck){
                Ok(fun) => {
                    Ok(Statement{
                        kind: StatementKind::Fun(fun.clone()),
                        pos: fun.pos.clone()
                    })
                },
                Err(()) => Err(())
            },
            Some(HIRInstruction::LocalVar) => match Local::load(chunk, typeck){
                Ok(local) => {
                    Ok(Statement{
                        kind: StatementKind::Local(local.clone()),
                        pos: local.pos.clone()
                    })
                },
                Err(()) => Err(())
            }
            _ => {
                chunk.jump_to(0).unwrap();
                if chunk.code.is_empty(){
                    typeck.emit_notice(format!("Malformed bytecode chunk: chunk is empty, which is a bug in the compiler."), NoticeLevel::ErrorPrint, BiPos::default())?;
                }else{
                    typeck.emit_notice(format!("Malformed bytecode chunk: could not read instruction from chunk; no further information provided. This should only happening during development and should never be seen by the user. If this is the case contact the author with this information: \n\tTypeck#load_statement failed to read instruction from chunk.\n\tFurther information: {}", chunk), NoticeLevel::ErrorPrint, BiPos::default())?;
                }
                Err(())
            }
        }
    }
}

impl<'a> super::Check<'a> for Statement{
    fn check(&self, typeck: &'a Typeck) -> Result<(), ()> {
        match &self.kind{
            StatementKind::Local(local) => local.check(typeck),
            StatementKind::Fun(fun) => fun.check(typeck),
            StatementKind::Property(property) => property.check(typeck),
        }
    }
}

impl super::Unload for Statement{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        match &self.kind{
            StatementKind::Fun(fun) => match fun.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(()) => return Err(())
            },
            StatementKind::Local(local) => match local.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(()) => return Err(())
            },
            StatementKind::Property(prop) => match prop.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(()) => return Err(())
            },
        }
        chunk.write_pos(self.pos);
        Ok(chunk)
    }
}