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

use core::pos::BiPos;

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
use notices::NoticeLevel;

impl Unload for FunParam{
    fn unload(&self) -> Result<Chunk, ()> {
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::FnParam);
        chunk.write_pos(self.pos);
        match self.ident.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(()) => return Err(())
        }
        match self.ty.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(()) => return Err(())
        }
        Ok(chunk)
    }
}

impl<'a> Check<'a> for Fun{
    fn check(&self, typeck: &'a Typeck) -> Result<(), ()> {
        for statement in self.body.iter(){
            if statement.check(typeck).is_err(){
                return Err(())
            }
        }
        Ok(())
    }
}

impl Load for Fun{
    type Output = Fun;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let ident = match Identifier::load(chunk, typeck){
            Ok(ident) => ident,
            Err(()) => return Err(())
        };
        let mut params = vec![];
        while let Some(ins) = chunk.read_instruction() as Option<HIRInstruction>{
            if ins == HIRInstruction::EndParams{
                break;
            }

            if ins != HIRInstruction::FnParam{
                typeck.emit_notice(format!("Expected an fn param instruction but instead got {:?}; this is a bug in the compiler.", ins), NoticeLevel::Error, pos)?;
                return Err(())
            }

            let param_ident = match Identifier::load(chunk, typeck){
                Ok(ident) => ident,
                Err(()) => return Err(())
            };
            let param_type = match Ty::load(chunk, typeck){
                Ok(ty) => ty,
                Err(()) => return Err(())
            };
            params.push(FunParam{
                ident: param_ident,
                ty: param_type,
                pos
            });
        }
        let return_type = match Ty::load(chunk, typeck){
            Ok(ty) => ty,
            Err(()) => return Err(())
        };

        let block_chunk = match typeck.chunk_rx.recv(){
            Ok(Some(chunk)) => {
                chunk
            }
            Ok(None) => {
                typeck.emit_notice(format!("Incomplete bytecode. Expected a chunk for function body. This is a bug in the compiler."), NoticeLevel::Error, BiPos::default())?;
                typeck.emit_notice(format!("The previous error should only have occurred during development. If you are a user then please notify the author."), NoticeLevel::Notice, BiPos::default())?;
                return Err(())
            }
            Err(_) =>{
                typeck.emit_notice(format!("Failed to get chunk from chunk channel."), NoticeLevel::Error, BiPos::default())?;
                typeck.emit_notice(format!("The previous error should only have occurred during development. If you are a user then please notify the author."), NoticeLevel::Notice, BiPos::default())?;
                return Err(())
            }
        };
        let mut block: Vec<Statement> = if let Some(HIRInstruction::Block) = block_chunk.read_instruction(){
            vec![]
        }else{
            let pos = match chunk.read_pos(){
                Ok(pos) => pos,
                Err(msg) => {
                    typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                    return Err(())
                }
            };
            typeck.emit_notice(format!("Expected a block chunk denotig the start of a function body."), NoticeLevel::Error, pos)?;
            return Err(())
        };
        loop{
            let next_chunk = typeck.chunk_rx.recv().unwrap().unwrap();
            if let Some(HIRInstruction::EndBlock) = next_chunk.read_instruction(){
                break;
            }
            next_chunk.jump_to(0).unwrap();
            let statement = match Statement::load(&next_chunk, typeck){
                Ok(statement) => statement,
                Err(()) => return Err(())
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
        Ok(fun)
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
            Err(()) => return Err(())
        }

        //Write the params information
        for param in self.params.iter(){
            match param.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(()) => return Err(())
            }
        }
        //Write the return type information
        match self.ty.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(()) => return Err(())
        }
        //Write the body
        for statement in self.body.iter(){
            match statement.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(()) => return Err(())
            }
        }

        chunk.write_instruction(HIRInstruction::EndFn);
        
        Ok(chunk)
    }
}