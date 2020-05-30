use super::{
    Parser,
    OwnedParse,
};

use ir::{
    Chunk,
};

use lexer::tokens::{
    TokenType,
};

use notices::{
    Notice,
};

mod binary;
mod literal;

pub struct ExpressionParser;

impl OwnedParse for ExpressionParser{
    fn owned_parse(parser: &mut Parser) -> Result<Chunk,Notice>{
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
            _ => {
                match literal::LiteralParser::owned_parse(parser){
                    Ok(literal) => chunk.write_chunk(literal),
                    Err(msg) => {
                        return Err(msg)
                    }
                }
            }
            
        }
        Ok(chunk)
    }
}