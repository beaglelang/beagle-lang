use ir::{
    Chunk,
};

use super::{
    SymbolResolver,
    Load,
    ident::Identifier,
    statement::Statement
};

use ir::hir::HIRInstruction;
use ir_traits::{ReadInstruction};
use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

use core::pos::BiPos;

#[derive(Debug, Clone)]
pub struct Fun{
    pub ident: Identifier,
    pub return_ty: Identifier,
    pub params: Vec<FunParam>,
    pub body: Vec<Statement>,
    pub pos: BiPos,
}

#[derive(Debug, Clone)]
pub struct FunParam{
    ident: Identifier,
    ty: Identifier,
}

impl Load for Fun{
    type Output = Fun;

    fn load(chunk: &Chunk, symbol_resolver: &SymbolResolver) -> Result<Option<Self::Output>, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                    symbol_resolver.emit_diagnostic(&[], &[diag_source]);
                return Err(())
            }
        };
        let ident = match Identifier::load(chunk, symbol_resolver){
            Ok(Some(ident)) => ident,
            Ok(None) => return Ok(None),
            Err(msg) => return Err(msg)
        };
        let mut params = vec![];
        while let Some(ins) = chunk.read_instruction() as Option<HIRInstruction>{
            if ins == HIRInstruction::EndParams{
                break;
            }

            if ins != HIRInstruction::FnParam{
                let pos = match chunk.read_pos(){
                    Ok(pos) => pos,
                    Err(msg) => {
                        let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                            .message(msg)
                            .level(DiagnosticLevel::Error)
                            .build();
                            symbol_resolver.emit_diagnostic(&[], &[diag_source]);
                        return Err(())
                    }
                };
                let source = match symbol_resolver.request_source_snippet(pos){
                    Ok(source) => source,
                    Err(diag) => {
                        symbol_resolver.emit_diagnostic(&[], &[diag]);
                        return Err(())
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                            .message(format!("Expected an fn param instruction but instead got {:?}", ins))
                            .level(DiagnosticLevel::Error)
                            .source(source)
                            .build();
                            symbol_resolver.emit_diagnostic(&[format!("This is a bug in the compiler.")], &[diag_source]);
                return Err(())
            }

            let ident = match Identifier::load(chunk, symbol_resolver){
                Ok(Some(ident)) => ident,
                Ok(None) => return Ok(None),
                Err(msg) => return Err(msg)
            };
            let ty = match Identifier::load(chunk, symbol_resolver){
                Ok(Some(ty)) => ty,
                Ok(None) => return Ok(None),
                Err(notice) => return Err(notice)
            };
            params.push(FunParam{
                ident,
                ty,
            });
        }
        let return_ty = match Identifier::load(chunk, symbol_resolver){
            Ok(Some(ty)) => ty,
            Ok(None) => return Ok(None),
            Err(notice) => return Err(notice)
        };

        let block_chunk = match symbol_resolver.ir_rx.recv(){
            Ok(Some(chunk)) => {
                chunk
            }
            Ok(None) => {
                let report = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .message(format!("Failed to get chunk from chunk channel."))
                    .level(DiagnosticLevel::Error)
                    .build();
                    symbol_resolver.emit_diagnostic(&[
                    format!("The previous error should only have occurred during development. If you are a user then please notify the author.")
                    ], 
                    &[report]
                );
                return Err(())
            }
            Err(_) =>{
                let report = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .message(format!("Failed to get chunk from chunk channel."))
                    .level(DiagnosticLevel::Error)
                    .build();
                    symbol_resolver.emit_diagnostic(&[
                    format!("The previous error should only have occurred during development. If you are a user then please notify the author.")
                    ], 
                    &[report]
                );
                return Err(())
            }
        };
        let block = block_chunk.read_instruction();
        let mut block: Vec<Statement> = if let Some(HIRInstruction::Block) = block{
            vec![]
        }else{
            let pos = match chunk.read_pos(){
                Ok(pos) => pos,
                Err(msg) => {
                    let diag_source = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                        .message(msg)
                        .level(DiagnosticLevel::Error)
                        .build();
                        symbol_resolver.emit_diagnostic(&[], &[diag_source]);
                    return Err(())
                }
            };
            let source = match symbol_resolver.request_source_snippet(pos){
                Ok(source) => source,
                Err(diag) => {
                    symbol_resolver.emit_diagnostic(&[], &[diag]);
                    return Err(())
                }
            };
            let report = DiagnosticSourceBuilder::new(symbol_resolver.module_name.clone(), 0)
                    .message(format!("Expected a function body but instead got {:?}.", block))
                    .level(DiagnosticLevel::Error)
                    .source(source)
                    .range(pos.col_range())
                    .build();
            symbol_resolver.emit_diagnostic(&[], &[report]);
            return Err(())
        };
        loop{
            let next_chunk = symbol_resolver.ir_rx.recv().unwrap().unwrap();
            if let Some(HIRInstruction::EndBlock) = next_chunk.read_instruction(){
                break;
            }
            next_chunk.jump_to(0).unwrap();
            let statement = match Statement::load(&next_chunk, symbol_resolver){
                Ok(Some(statement)) => statement,
                Ok(None) => return Ok(None),
                Err(notice) => return Err(notice)
            };
            block.push(statement);
        }
        let fun = Fun{
            ident,
            return_ty,
            body: block,
            params,
            pos
        };
        Ok(Some(fun))
    }

}