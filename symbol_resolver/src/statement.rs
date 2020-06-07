use crate::{
    SymbolResolver,
    Load,
    property::Property,
    fun::Fun,
    local::Local,
};

use ir::{
    Chunk,
    hir::HIRInstruction
};

use ir_traits::ReadInstruction;

use notices::{
    DiagnosticLevel,
    DiagnosticSourceBuilder
};

use core::pos::BiPos;

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

impl Load for Statement{
    type Output = Statement;
    fn load(chunk: &Chunk, symbol_resolver: &SymbolResolver) -> Result<Option<Self::Output>, ()>{
        match chunk.read_instruction(){
            Some(HIRInstruction::Property) => match Property::load(chunk, symbol_resolver){
                Ok(Some(property)) => {
                    Ok(Some(Statement{
                        kind: StatementKind::Property(property.clone()),
                        pos: property.pos.clone()
                    }))
                },
                Ok(None) => return Ok(None),
                Err(msg) => return Err(msg)
            },
            Some(HIRInstruction::Fn) => match Fun::load(chunk, symbol_resolver){
                Ok(Some(fun)) => {
                    Ok(Some(Statement{
                        kind: StatementKind::Fun(fun.clone()),
                        pos: fun.pos.clone()
                    }))
                },
                Ok(None) => return Ok(None),
                Err(msg) => return Err(msg)
            },
            Some(HIRInstruction::LocalVar) => match Local::load(chunk, symbol_resolver){
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
                let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(message)
                    .build();
                symbol_resolver.emit_diagnostic(&[
                    format!("This should only happening during development and should never be seen by the user. If this is the case contact the author with this information: \n\tTypeck#load_statement failed to read instruction from chunk.\n\tFurther information: {}", chunk),
                ], &[diag_source]);
                return Err(())
            }
        }
    }
}