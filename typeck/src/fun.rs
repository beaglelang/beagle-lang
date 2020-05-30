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
    NoticeLevel,
    Notice,
};

impl Unload for FunParam{
    fn unload(&self) -> Result<Chunk, Notice> {
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
    fn check(&self, typeck: &'a Typeck) -> Result<(), Notice> {
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

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, Notice> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                return Err(Notice::new(
                    format!("Function Loader"),
                    msg,
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![]
                ))
            }
        };
        let ident = match Identifier::load(chunk, typeck){
            Ok(ident) => ident,
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
                        return Err(Notice::new(
                            format!("Function Loader"),
                            msg,
                            None,
                            None,
                            NoticeLevel::Error,
                            vec![]
                        ))
                    }
                };
                return Err(Notice::new(
                    format!("Function Loader"),
                    format!("Expected an fn param instruction but instead got {:?}; this is a bug in the compiler.", ins),
                    Some(typeck.module_name.clone()),
                    Some(pos),
                    NoticeLevel::Error,
                    vec![]
                ))
            }

            let param_ident = match Identifier::load(chunk, typeck){
                Ok(ident) => ident,
                Err(notice) => return Err(notice)
            };
            let param_type = match Ty::load(chunk, typeck){
                Ok(ty) => ty,
                Err(notice) => return Err(notice)
            };
            params.push(FunParam{
                ident: param_ident,
                ty: param_type,
                pos
            });
        }
        let return_type = match Ty::load(chunk, typeck){
            Ok(ty) => ty,
            Err(notice) => return Err(notice)
        };

        let block_chunk = match typeck.chunk_rx.recv(){
            Ok(Some(chunk)) => {
                chunk
            }
            Ok(None) => {
                let notice = Notice::new(
                    format!("Function Loader"),
                    format!("The previous error should only have occurred during development. If you are a user then please notify the author."),
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![]
                );
                return Err(Notice::new(
                    format!("Function Loader"),
                    format!("Failed to get chunk from chunk channel."),
                    Some(typeck.module_name.clone()),
                    Some(pos),
                    NoticeLevel::Error,
                    vec![notice]
                ))
            }
            Err(_) =>{
                let notice = Notice::new(
                    format!("Function Loader"),
                    format!("The previous error should only have occurred during development. If you are a user then please notify the author."),
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![]
                );
                return Err(Notice::new(
                    format!("Function Loader"),
                    format!("Failed to get chunk from chunk channel."),
                    Some(typeck.module_name.clone()),
                    Some(pos),
                    NoticeLevel::Error,
                    vec![notice]
                ))
            }
        };
        let mut block: Vec<Statement> = if let Some(HIRInstruction::Block) = block_chunk.read_instruction(){
            vec![]
        }else{
            let pos = match chunk.read_pos(){
                Ok(pos) => pos,
                Err(msg) => {
                    return Err(Notice::new(
                        format!("Function Loader"),
                        msg,
                        None,
                        None,
                        NoticeLevel::Error,
                        vec![]
                    ))
                }
            };
            return Err(Notice::new(
                format!("Function Loader"),
                format!("Expected a block chunk denotig the start of a function body."),
                Some(typeck.module_name.clone()),
                Some(pos),
                NoticeLevel::Error,
                vec![]
            ))
        };
        loop{
            let next_chunk = typeck.chunk_rx.recv().unwrap().unwrap();
            if let Some(HIRInstruction::EndBlock) = next_chunk.read_instruction(){
                break;
            }
            next_chunk.jump_to(0).unwrap();
            let statement = match Statement::load(&next_chunk, typeck){
                Ok(statement) => statement,
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
        Ok(fun)
    }

}

impl Unload for Fun{
    fn unload(&self) -> Result<Chunk, Notice> {
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