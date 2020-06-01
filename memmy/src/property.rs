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

use notices::{
    DiagnosticSource,
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

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
    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, DiagnosticSource> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                return Err(diagnosis)
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
                return Err(diagnosis)
            }
        };

        let typename_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diagnosis = DiagnosticSourceBuilder::new(memmy.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                return Err(diagnosis)
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
                return Err(diagnosis)
            }
        };
        
        let typename = chunk.read_string();
        let expr = match Expression::load(chunk, memmy){
            Ok(expr) => expr,
            Err(diag) => return Err(diag)
        };
        Ok(Property{
            ident,
            typename: Identifier{
                ident: typename.to_owned(),
                pos: typename_pos
            },
            mutable: Mutability{
                mutable,
                pos: mut_pos
            },
            pos,
            expression: expr,
        })
    }
}