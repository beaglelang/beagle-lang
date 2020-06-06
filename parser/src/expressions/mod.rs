use super::{
    Parser,
    OwnedParse,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use lexer::tokens::{
    TokenType,
    TokenData
};

use notices::{
    DiagnosticLevel,
    DiagnosticSource,
    DiagnosticSourceBuilder
};

use ir_traits::WriteInstruction;

mod binary;
mod literal;

pub struct ExpressionParser;

impl OwnedParse for ExpressionParser{
    fn owned_parse(parser: &mut Parser) -> Result<Chunk,DiagnosticSource>{
        let mut chunk = Chunk::new();
        let next = parser.next_token();
        match &next.type_{
            TokenType::Plus => {
                match binary::AddParser::owned_parse(parser){
                    Ok(expr) => {
                        chunk.write_chunk(expr);
                    }
                    Err(msg) => {
                        return Err(msg)
                    }
                }
            }
            TokenType::Minus => {
                match binary::SubParser::owned_parse(parser){
                    Ok(expr) => {
                        chunk.write_chunk(expr);
                    }
                    Err(msg) => {
                        return Err(msg)
                    }
                }
            }
            TokenType::Star => {
                match binary::MulParser::owned_parse(parser){
                    Ok(expr) => {
                        chunk.write_chunk(expr);
                    }
                    Err(msg) => {
                        return Err(msg)
                    }
                }
            }
            TokenType::Slash => {
                match binary::DivParser::owned_parse(parser){
                    Ok(expr) => {
                        chunk.write_chunk(expr);
                    }
                    Err(msg) => {
                        return Err(msg)
                    }
                }
            }
            TokenType::Number | TokenType::Decimal | TokenType::String => match literal::LiteralParser::owned_parse(parser){
                Ok(literal) => chunk.write_chunk(literal),
                Err(msg) => {
                    return Err(msg)
                }
            },
            TokenType::Identifier => {
                let ident = match &next.data{
                    TokenData::String(ident) => ident,
                    _ => {
                        return Err(
                            DiagnosticSourceBuilder::new(parser.name.clone(), next.pos.start.0)
                                .message(format!("Expected to find identifier data but instead found: {:?}", next.data))
                                .level(DiagnosticLevel::Error)
                                .build()
                        )
                    }
                };
                let mut chunk = Chunk::new();
                chunk.write_instruction(HIRInstruction::Reference);
                chunk.write_pos(next.pos);
                chunk.write_string(ident.clone());
            }
            _ => unimplemented!("Working on it! :)")
            
        }
        Ok(chunk)
    }
}