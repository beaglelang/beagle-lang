use super::{
    Parser,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::WriteInstruction;

use notices::{ DiagnosticSourceBuilder, DiagnosticSource, DiagnosticLevel };

use lexer::tokens::{
    TokenType,
    TokenData,
};

pub struct TypeParser;

impl TypeParser{
    pub fn get_type(parser: &mut Parser) -> Result<Chunk, DiagnosticSource>{
        let mut chunk = Chunk::new();
        if let Err(notice) = parser.advance(){
            return Err(notice)
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
                chunk
            }
            _ => {
                let source = match parser.request_source_snippet(){
                    Ok(source) => source,
                    Err(diag) => {
                        return Err(diag)
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), current_token.pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Expected a type identifier but instead got {:?}", current_token.type_))
                    .source(source)
                    .build();
                return Err(diag_source)
            },
        };
        
        Ok(ret)
    }
}