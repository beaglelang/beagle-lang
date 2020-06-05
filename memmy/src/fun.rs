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
    DiagnosticLevel
};

use ty::Ty;

#[derive(Debug, Clone)]
pub struct Fun<'a>{
    ident: Identifier,
    params: Vec<FunParam>,
    pos: BiPos,
    return_type: Ty,
    body: Vec<Statement<'a>>
}

impl<'a> Load for Fun<'a>{
    type Output = Fun<'a>;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .message(msg)
                    .level(DiagnosticLevel::Error)
                    .build();
                memmy.emit_diagnostic(&[], &[diagnosis]);
                return Err(())
            }
        };
        let ident = match Identifier::load(chunk, memmy){
            Ok(ident) => ident,
            Err(()) => {
                return Err(())
            }
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
        let return_type = match Ty::load(chunk, memmy){
            Ok(ty) => ty,
            Err(()) => {
                return Err(())
            }
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
    typename: Ty,
    pos: BiPos,
}

impl Load for FunParam{
    type Output = FunParam;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .message(msg)
                    .build();
                memmy.emit_diagnostic(&[], &[diagnosis]);
                return Err(())
            }
        };
        let ident = match Identifier::load(chunk, memmy){
            Ok(ident) => ident,
            Err(()) => return Err(())
        };

        let typename = match Ty::load(chunk, memmy){
            Ok(ty) => ty,
            Err(()) => return Err(())
        };

        Ok(FunParam{
            ident,
            typename,
            pos
        })
    }
}