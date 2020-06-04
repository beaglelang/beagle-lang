use crate::{
    Parser,
    OwnedParse,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use lexer::tokens::{
    TokenData,
    TokenType
};

use notices::{
    DiagnosticSource,
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

use ir_traits::WriteInstruction;

pub struct LiteralParser;

impl OwnedParse for LiteralParser{
    fn owned_parse(parser: &mut Parser) -> Result<Chunk, DiagnosticSource>{
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
            _ => {
                let source = match parser.request_source_snippet(parser.prev_token().pos){
                    Ok(source) => source,
                    Err(diag_source) => {
                        return Err(diag_source)
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), token.pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Attempted to parse literal expression but instead found {:?}", token.type_))
                    .range(token.pos.col_range())
                    .source(source)
                    .build();
                return Err(diag_source)
            }
        }
        Ok(chunk)
    }
}