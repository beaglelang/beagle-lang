use crate::{
    Parser,
    TryParse,
    ParseError,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use lexer::tokens::{
    TokenData,
    TokenType
};

use ir_traits::WriteInstruction;

pub struct LiteralParser;

impl TryParse for LiteralParser{
    fn try_parse(parser: &mut Parser) -> Result<Chunk, ParseError>{
        let token = parser.current_token();
        let mut chunk = Chunk::new();
        match &token.type_{
            TokenType::Decimal => {
                if let TokenData::Float(f) = token.data{
                    chunk.write_instruction(HIRInstruction::Float);
                    chunk.write_pos(token.pos);
                    chunk.write_float(f);
                }
            }
            TokenType::Number => {
                if let TokenData::Integer(i) = token.data{
                    chunk.write_instruction(HIRInstruction::Integer);
                    chunk.write_pos(token.pos);
                    chunk.write_int(i);
                }
            }
            TokenType::String => {
                if let TokenData::String(s) = &token.data{
                    chunk.write_instruction(HIRInstruction::String);
                    chunk.write_pos(token.pos);
                    chunk.write_str(s);
                }
            }
            TokenType::KwNone => {
                chunk.write_instruction(HIRInstruction::None);
                chunk.write_pos(token.pos);
            }
            TokenType::KwTrue => {
                chunk.write_instruction(HIRInstruction::Bool);
                chunk.write_pos(token.pos);
                chunk.write_bool(true);
            }
            TokenType::KwFalse => {
                chunk.write_instruction(HIRInstruction::Bool);
                chunk.write_pos(token.pos);
                chunk.write_bool(false);
            }
            _ => return Err(ParseError{
                cause: None,
                msg: format!("Unrecognized expression literal: {:?}", token),
                pos: token.pos,
            })
        }
        Ok(chunk)
    }
}