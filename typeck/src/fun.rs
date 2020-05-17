use super::{
    statement::{
        Statement,
    },
    Ty,
    ident::Identifier,
    Check,
    Unload,
};

use core::pos::BiPos;

use ir::Chunk;

use super::Typeck;

use ir::hir::HIRInstruction;
use ir_traits::{ReadInstruction, WriteInstruction};
use notices::NoticeLevel;


#[derive(Debug, Clone)]
pub struct Fun{
    pub ident: Identifier,
    pub ty: Ty,
    pub params: Vec<FunParam>,
    pub body: Vec<Statement>,
    pub pos: BiPos,
}

#[derive(Debug, Clone)]
pub struct FunParam{
    pub ident: Identifier,
    pub ty: Ty,
    pub pos: BiPos
}

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

impl super::Load for Fun{
    type Output = Fun;

    fn load(chunk: &Chunk, typeck: &Typeck) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let name = chunk.read_string();
        let name_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::Error, BiPos::default())?;
                return Err(())
            } 
        };
        let ident = Identifier{
            ident: name.to_owned(),
            pos: name_pos,
        };
        let mut params = vec![];
        while let Some(ins) = chunk.read_instruction() as Option<HIRInstruction>{
            if ins == HIRInstruction::EndParams{
                break;
            }

            let pos = match chunk.read_pos(){
                Ok(pos) => pos,
                Err(msg) => {
                    typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                    return Err(())
                }
            };
            if ins != HIRInstruction::FnParam{
                typeck.emit_notice(format!("Expected an fn param instruction but instead got {:?}; this is a bug in the compiler.", ins), NoticeLevel::Error, pos)?;
                return Err(())
            }

            let param_name = chunk.read_string();
            let param_type_pos = match chunk.read_pos(){
                Ok(pos) => pos,
                Err(msg) => {
                    typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                    return Err(())
                }
            };
            let param_type = chunk.read_instruction() as Option<HIRInstruction>;
            let param_typename = match param_type{
                Some(type_) => {
                    if type_ == HIRInstruction::Custom{
                        Some(chunk.read_string().to_string())
                    }else{
                        Some(format!("{:?}", type_))
                    }
                }
                None => {
                    typeck.emit_notice(format!("Expected a param type annotation but instead got none. This is a bug in the compiler."), NoticeLevel::Error, pos)?;
                    return Err(())
                }
            };
            params.push(FunParam{
                ident: Identifier{
                    ident: param_name.to_owned(),
                    pos
                },
                ty: Ty{
                    ident: param_typename.unwrap(),
                    pos: param_type_pos
                },
                pos
            });
        }
        let fun_type_pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                typeck.emit_notice(msg, NoticeLevel::ErrorPrint, BiPos::default())?;
                return Err(())
            }
        };
        let return_type = chunk.read_instruction() as Option<HIRInstruction>;
        let typename = match return_type{
            Some(name_ins) => {
                if name_ins == HIRInstruction::Custom{
                    chunk.read_string().to_owned()
                }else{
                    format!("{:?}", name_ins)
                }
            }
            None => {
                typeck.emit_notice(format!("Expected a return type instruction but instead got {:?}; this is compiler bug.", return_type.unwrap()), NoticeLevel::Error, fun_type_pos)?;
                return Err(())
            }
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
            ty: Ty{
                ident: typename,
                pos: fun_type_pos,
            },
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
        match self.ident.unload(){
            Ok(ch) => chunk.write_chunk(ch),
            Err(()) => return Err(())
        }
        for param in self.params.iter(){
            match param.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(()) => return Err(())
            }
        }
        for statement in self.body.iter(){
            match statement.unload(){
                Ok(ch) => chunk.write_chunk(ch),
                Err(()) => return Err(())
            }
        }
        
        Ok(chunk)
    }
}