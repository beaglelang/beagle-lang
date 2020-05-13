use super::{
    Parser,
    ParseRule,
};

use ir::{
    Chunk,
    hir::HIRInstruction
};

use ir_traits::WriteInstruction;

use lexer::tokens::TokenData;

pub struct ExpressionParser;

impl ParseRule for ExpressionParser{
    fn parse(parser: &mut Parser) -> Result<(),()>{
        let token = parser.current_token();
        let mut chunk = Chunk::new();
        match &token.data{
            TokenData::Float(f) => {
                chunk.write_instruction(HIRInstruction::Float);
                chunk.write_pos(token.pos);
                chunk.write_float(*f);
            }
            TokenData::Integer(i) => {
                chunk.write_instruction(HIRInstruction::Integer);
                chunk.write_pos(token.pos);
                chunk.write_int(*i);
            }
            TokenData::String(s) => {
                chunk.write_instruction(HIRInstruction::String);
                chunk.write_pos(token.pos);
                chunk.write_string(s.clone());
            }
            TokenData::None => {
                chunk.write_instruction(HIRInstruction::None);
                chunk.write_pos(token.pos);
            }
        }
        parser.emit_ir_whole(chunk);
        parser.advance().unwrap();
        Ok(())
    }
}