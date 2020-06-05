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

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

#[derive(Debug, Clone)]
pub struct Statement<'a>{
    pos: BiPos,
    kind: StatementKind<'a>,
}

#[derive(Debug, Clone)]
pub enum StatementKind<'a>{
    Property(Property<'a>),
    Fun(Fun<'a>),
    Local(Local)
}

impl<'a> Load for Statement<'a>{
    type Output = Statement<'a>;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        let ins = chunk.read_instruction();
        match &ins{
            Some(HIRInstruction::Fn) => {
                let fun = match Fun::load(chunk, memmy){
                    Ok(fun) => fun,
                    Err(diag) => return Err(diag)
                };
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                            .level(DiagnosticLevel::Error)
                            .message(msg)
                            .build();
                        memmy.emit_diagnostic(&[], &[diagnosis]);
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
                    Err(diag) => return Err(diag)
                };
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                            .level(DiagnosticLevel::Error)
                            .message(msg)
                            .build();
                        memmy.emit_diagnostic(&[], &[diagnosis]);
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
                    Err(diag) => return Err(diag)
                };
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                            .level(DiagnosticLevel::Error)
                            .message(msg)
                            .build();
                        memmy.emit_diagnostic(&[], &[diagnosis]);
                        return Err(())
                    }
                };
                Ok(Statement{
                    kind: StatementKind::Local(local),
                    pos
                })
            }
            _ => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                        .message(format!("This feature is not yet implemented: {:?}", ins.clone().unwrap()))
                        .level(DiagnosticLevel::Error)
                        .build();
                    memmy.emit_diagnostic(&[], &[diagnosis]);
                    return Err(())
            }
        }
        
    }
}