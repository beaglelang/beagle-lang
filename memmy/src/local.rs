use super::{
    ident::Identifier,
    expr::Expression,
    Load,
    MemmyGenerator,
};

use core::pos::BiPos;

use ir::{ Chunk, hir::HIRInstruction };

use ir_traits::ReadInstruction;

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

#[derive(Debug, Clone)]
pub struct LocalMut{
    mutable: bool,
    pos: BiPos,
}

#[derive(Debug, Clone)]
pub struct Local{
    ident: Identifier,
    mutable: LocalMut,
    typename: Identifier,
    pos: BiPos,
    expr: Expression,
}

impl Load for Local{
    type Output = Local;
    
    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
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
        let ident = match Identifier::load(chunk, memmy){
            Ok(pos) => pos,
            Err(diag) => {
                return Err(diag)
            }
        };
        let mutable = chunk.read_bool();
        let mut_pos = match chunk.read_pos(){
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

        let typename_pos = match chunk.read_pos(){
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

        match chunk.read_instruction(){
            Some(HIRInstruction::Integer) | Some(HIRInstruction::Float) | Some(HIRInstruction::String) | Some(HIRInstruction::Unit) | Some(HIRInstruction::Custom) => {},
            Some(_) => {}
            None =>{
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Attempted to read type information from typeck while loading property into memmy, found None."))
                    .build();
                memmy.emit_diagnostic(&[], &[diagnosis]);
                return Err(())
            }
        };
        
        let typename = chunk.read_string();
        let expr = match Expression::load(chunk, memmy){
            Ok(expr) => expr,
            Err(diag) => return Err(diag)
        };
        Ok(Local{
            ident,
            typename: Identifier{
                ident: typename.to_owned(),
                pos: typename_pos
            },
            mutable: LocalMut{
                mutable,
                pos: mut_pos
            },
            pos,
            expr,
        })
    }
}