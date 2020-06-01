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
    DiagnosticLevel,
    DiagnosticSourceBuilder,
};

impl Load for Statement{
    type Output = Statement;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, ()> {
        match chunk.read_instruction(){
            Some(HIRInstruction::Property) => match Property::load(chunk, typeck){
                Ok(Some(property)) => {
                    Ok(Some(Statement{
                        kind: StatementKind::Property(property.clone()),
                        pos: property.pos.clone()
                    }))
                },
                Ok(None) => return Ok(None),
                Err(msg) => return Err(msg)
            },
            Some(HIRInstruction::Fn) => match Fun::load(chunk, typeck){
                Ok(Some(fun)) => {
                    Ok(Some(Statement{
                        kind: StatementKind::Fun(fun.clone()),
                        pos: fun.pos.clone()
                    }))
                },
                Ok(None) => return Ok(None),
                Err(msg) => return Err(msg)
            },
            Some(HIRInstruction::LocalVar) => match Local::load(chunk, typeck){
                Ok(Some(local)) => {
                    Ok(Some(Statement{
                        kind: StatementKind::Local(local.clone()),
                        pos: local.pos.clone()
                    }))
                },
                Ok(None) => return Ok(None),
                Err(msg) => return Err(msg)
            }
            _ => {
                chunk.jump_to(0).unwrap();
                let message = if chunk.code.is_empty(){
                    format!("Malformed bytecode chunk: chunk is empty.")
                }else{
                    format!("Malformed bytecode chunk: could not read instruction from chunk; no further information provided. ")
                };
                let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(message)
                    .build();
                typeck.emit_diagnostic(&[
                    format!("This should only happening during development and should never be seen by the user. If this is the case contact the author with this information: \n\tTypeck#load_statement failed to read instruction from chunk.\n\tFurther information: {}", chunk),
                ], &[diag_source]);
                return Err(())
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

impl Unload for Statement{
    fn unload(&self) -> Result<Chunk, ()> {
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