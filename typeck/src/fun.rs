use super::{
    Check,
};

use ident::Identifier;
use ty::Ty;
use stmt::{
    Statement,
    fun::{
        Fun,
        FunParam
    },
};

use ir::{
    Chunk,
};

use super::{
    Typeck,
    Load,
    Unload,
};

use ir::hir::HIRInstruction;
use ir_traits::{ReadInstruction, WriteInstruction};
use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

impl Unload for FunParam{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::FnParam);
        chunk.write_pos(self.pos);
        match self.ident.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        match self.ty.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        Ok(chunk)
    }
}

impl<'a> Check<'a> for Fun{
    fn check(&self, typeck: &'a Typeck) -> Result<(), ()> {
        for statement in self.body.iter(){
            if let Err(notice) = statement.check(typeck){
                return Err(notice)
            }
        }
        Ok(())
    }
}

impl Load for Fun{
    type Output = Fun;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Option<Self::Output>, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .level(DiagnosticLevel::Error)
                    .message(msg)
                    .build();
                typeck.emit_diagnostic(&[], &[diag_source]);
                return Err(())
            }
        };
        let ident = match Identifier::load(chunk, typeck){
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
                        let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                            .message(msg)
                            .level(DiagnosticLevel::Error)
                            .build();
                        typeck.emit_diagnostic(&[], &[diag_source]);
                        return Err(())
                    }
                };
                let source = match typeck.request_source_snippet(pos){
                    Ok(source) => source,
                    Err(diag) => {
                        typeck.emit_diagnostic(&[], &[diag]);
                        return Err(())
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                            .message(format!("Expected an fn param instruction but instead got {:?}", ins))
                            .level(DiagnosticLevel::Error)
                            .source(source)
                            .build();
                typeck.emit_diagnostic(&[format!("This is a bug in the compiler.")], &[diag_source]);
                return Err(())
            }

            let param_ident = match Identifier::load(chunk, typeck){
                Ok(Some(ident)) => ident,
                Ok(None) => return Ok(None),
                Err(msg) => return Err(msg)
            };
            let param_type = match Ty::load(chunk, typeck){
                Ok(Some(ty)) => ty,
                Ok(None) => return Ok(None),
                Err(notice) => return Err(notice)
            };
            params.push(FunParam{
                ident: param_ident,
                ty: param_type,
                pos
            });
        }
        let return_type = match Ty::load(chunk, typeck){
            Ok(Some(ty)) => ty,
            Ok(None) => return Ok(None),
            Err(notice) => return Err(notice)
        };

        let block_chunk = match typeck.chunk_rx.recv(){
            Ok(Some(chunk)) => {
                chunk
            }
            Ok(None) => {
                let report = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .message(format!("Failed to get chunk from chunk channel."))
                    .level(DiagnosticLevel::Error)
                    .build();
                typeck.emit_diagnostic(&[
                    format!("The previous error should only have occurred during development. If you are a user then please notify the author.")
                    ], 
                    &[report]
                );
                return Err(())
            }
            Err(_) =>{
                let report = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .message(format!("Failed to get chunk from chunk channel."))
                    .level(DiagnosticLevel::Error)
                    .build();
                typeck.emit_diagnostic(&[
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
                    let diag_source = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                        .message(msg)
                        .level(DiagnosticLevel::Error)
                        .build();
                    typeck.emit_diagnostic(&[], &[diag_source]);
                    return Err(())
                }
            };
            let source = match typeck.request_source_snippet(pos){
                Ok(source) => source,
                Err(diag) => {
                    typeck.emit_diagnostic(&[], &[diag]);
                    return Err(())
                }
            };
            let report = DiagnosticSourceBuilder::new(typeck.module_name.clone(), 0)
                    .message(format!("Expected a function body but instead got {:?}.", block))
                    .level(DiagnosticLevel::Error)
                    .source(source)
                    .range(pos.col_range())
                    .build();
            typeck.emit_diagnostic(&[], &[report]);
            return Err(())
        };
        loop{
            let next_chunk = typeck.chunk_rx.recv().unwrap().unwrap();
            if let Some(HIRInstruction::EndBlock) = next_chunk.read_instruction(){
                break;
            }
            next_chunk.jump_to(0).unwrap();
            let statement = match Statement::load(&next_chunk, typeck){
                Ok(Some(statement)) => statement,
                Ok(None) => return Ok(None),
                Err(notice) => return Err(notice)
            };
            block.push(statement);
        }
        let fun = Fun{
            ident,
            ty: return_type,
            body: block,
            params,
            pos
        };
        Ok(Some(fun))
    }

}

impl Unload for Fun{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Fn);
        chunk.write_pos(self.pos);
        //Write the identifier
        match self.ident.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }

        //Write the params information
        for param in self.params.iter(){
            match param.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(notice) => return Err(notice)
            }
        }
        //Write the return type information
        match self.ty.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(notice) => return Err(notice)
        }
        //Write the body
        for statement in self.body.iter(){
            match statement.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(notice) => return Err(notice)
            }
        }

        chunk.write_instruction(HIRInstruction::EndFn);
        
        Ok(chunk)
    }
}