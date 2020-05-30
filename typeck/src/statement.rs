use stmt::{
    property::Property,
    fun::Fun,
    local::Local,
    Statement,
    StatementKind
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use super::{
    Typeck,
    Load,
    Unload,
};
use ir_traits::ReadInstruction;
use notices::{ 
    NoticeLevel,
    Notice,
};

impl Load for Statement{
    type Output = Statement;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, Notice> {
        match chunk.read_instruction(){
            Some(HIRInstruction::Property) => match Property::load(chunk, typeck){
                Ok(property) => {
                    Ok(Statement{
                        kind: StatementKind::Property(property.clone()),
                        pos: property.pos.clone()
                    })
                },
                Err(msg) => return Err(msg)
            },
            Some(HIRInstruction::Fn) => match Fun::load(chunk, typeck){
                Ok(fun) => {
                    Ok(Statement{
                        kind: StatementKind::Fun(fun.clone()),
                        pos: fun.pos.clone()
                    })
                },
                Err(msg) => return Err(msg)
            },
            Some(HIRInstruction::LocalVar) => match Local::load(chunk, typeck){
                Ok(local) => {
                    Ok(Statement{
                        kind: StatementKind::Local(local.clone()),
                        pos: local.pos.clone()
                    })
                },
                Err(msg) => return Err(msg)
            }
            _ => {
                chunk.jump_to(0).unwrap();
                let message = if chunk.code.is_empty(){
                    format!("Malformed bytecode chunk: chunk is empty, which is a bug in the compiler.")
                }else{
                    format!("Malformed bytecode chunk: could not read instruction from chunk; no further information provided. This should only happening during development and should never be seen by the user. If this is the case contact the author with this information: \n\tTypeck#load_statement failed to read instruction from chunk.\n\tFurther information: {}", chunk)
                };
                return Err(Notice::new(
                    format!("Statement Loader"),
                    message,
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![]
                ))
            }
        }
    }
}

impl<'a> super::Check<'a> for Statement{
    fn check(&self, typeck: &'a Typeck) -> Result<(), Notice> {
        match &self.kind{
            StatementKind::Local(local) => local.check(typeck),
            StatementKind::Fun(fun) => fun.check(typeck),
            StatementKind::Property(property) => property.check(typeck),
        }
    }
}

impl Unload for Statement{
    fn unload(&self) -> Result<Chunk, Notice> {
        let mut chunk = Chunk::new();
        match &self.kind{
            StatementKind::Fun(fun) => match fun.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(msg) => return Err(msg)
            },
            StatementKind::Local(local) => match local.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(msg) => return Err(msg)
            },
            StatementKind::Property(prop) => match prop.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(msg) => return Err(msg)
            },
        }
        chunk.write_pos(self.pos);
        Ok(chunk)
    }
}