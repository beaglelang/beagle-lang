use super::{
    Parser,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::WriteInstruction;

use notices::NoticeLevel;

use lexer::tokens::{
    TokenType,
    TokenData,
};

pub struct TypeParser;

impl TypeParser{
    pub fn get_type(parser: &mut Parser) -> Result<Chunk, ()>{
        let mut chunk = Chunk::new();
        if parser.advance().is_err(){
            parser.emit_notice(parser.current_token().pos, NoticeLevel::Error, "Could not advance parser.".to_string());
            return Err(())
        }
        let current_token = parser.current_token();
        let ret = match (&current_token.type_, &current_token.data) {
            (TokenType::Identifier, TokenData::String(s)) => {
                chunk.write_pos(current_token.pos);
                let str = s.as_str();
                match str{
                    "Int" => chunk.write_instruction(HIRInstruction::Integer),
                    "Float" => chunk.write_instruction(HIRInstruction::Float),
                    "String" => chunk.write_instruction(HIRInstruction::String),
                    "Bool" => chunk.write_instruction(HIRInstruction::Bool),
                    _ => {
                        chunk.write_instruction(HIRInstruction::Custom);
                        chunk.write_string(s.clone());
                    }
                }
                Ok(chunk)
            }
            _ => Err(()),
        };
        if ret.is_ok() {
            ret
        } else {
            Err(())
        }
    }
}