use super::{
    ident::Identifier,
    statements::Statement,
    Load,
    MemmyGenerator
};

use ir::{ Chunk, hir::HIRInstruction };

use ir_traits::{ ReadInstruction };

use core::{
    pos::BiPos
};

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticSource,
    DiagnosticLevel
};

#[derive(Debug, Clone)]
pub struct Fun{
    ident: Identifier,
    params: Vec<FunParam>,
    pos: BiPos,
    return_type: Identifier,
    body: Vec<Statement>
}

impl Load for Fun{
    type Output = Fun;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, DiagnosticSource> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .message(msg)
                    .level(DiagnosticLevel::Error)
                    .build();
                return Err(diagnosis)
            }
        };
        let ident = match Identifier::load(chunk, memmy){
            Ok(ident) => ident,
            Err(diag) => return Err(diag)
        };
        let mut params = vec![];
        loop{
            match chunk.read_instruction(){
                Some(HIRInstruction::FnParam) => params.push(match FunParam::load(chunk, memmy){
                    Ok(param) => param,
                    Err(diag) => return Err(diag)
                }),
                Some(HIRInstruction::EndParams) => break,
                _ => break
            }
        }
        let return_type = match Identifier::load(chunk, memmy){
            Ok(ident) => ident,
            Err(diag) => return Err(diag)
        };
        let mut body = vec![];
        loop{
            if let Some(HIRInstruction::EndFn) = chunk.read_instruction(){
                break;
            }
            chunk.dec_ins_ptr(1);
            match Statement::load(chunk, memmy){
                Ok(statement) => body.push(statement),
                Err(diag) => return Err(diag)
            }

        }
        Ok(Fun{
            ident,
            params,
            pos,
            return_type,
            body
        })
    }
}

#[derive(Debug, Clone)]
pub struct FunParam{
    ident: Identifier,
    typename: Identifier,
    pos: BiPos,
}

impl Load for FunParam{
    type Output = FunParam;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, DiagnosticSource> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .message(msg)
                    .build();
                return Err(diagnosis)
            }
        };
        let ident = match Identifier::load(chunk, memmy){
            Ok(ident) => ident,
            Err(diag) => return Err(diag)
        };
        match chunk.read_instruction(){
            Some(HIRInstruction::Integer) => {},
            Some(_) => {},
            None => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .message(format!("Expected to read instruction for type information. This is a compiler bug and should only be seen during development."))
                    .build();
                return Err(diagnosis)
            }
        }
        let typename = match Identifier::load(chunk, memmy){
            Ok(ident) => ident,
            Err(diag) => return Err(diag)
        };
        Ok(FunParam{
            ident,
            typename,
            pos
        })
    }
}