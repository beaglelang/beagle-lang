use super::{
    Parser,
    TryParse,
    ParseError,
};

use ir::{
    Chunk,
};

use lexer::tokens::{
    TokenType,
};

mod binary;
mod literal;

pub struct ExpressionParser;

impl TryParse for ExpressionParser{
    fn try_parse(parser: &mut Parser) -> Result<Chunk,ParseError>{
        let mut chunk = Chunk::new();
        let next = parser.next_token();
        match &next.type_{
            TokenType::Plus => {
                match binary::AddParser::try_parse(parser){
                    Ok(expr) => {
                        chunk.write_chunk(expr);
                    }
                    Err(msg) => {
                        let pos = parser.current_token().pos;
                        return Err(ParseError{
                            cause: Some(Box::new(msg)),
                            msg: format!("An error occurred while trying to parse add expression"),
                            pos,
                        })
                    }
                }
            }
            TokenType::Minus => {
                match binary::SubParser::try_parse(parser){
                    Ok(expr) => {
                        chunk.write_chunk(expr);
                    }
                    Err(msg) => {
                        return Err(ParseError{
                            cause: Some(Box::new(msg)),
                            msg: format!("An error occurred while trying to parse sub expression"),
                            pos: parser.current_token().pos,
                        })
                    }
                }
            }
            TokenType::Star => {
                match binary::MulParser::try_parse(parser){
                    Ok(expr) => {
                        chunk.write_chunk(expr);
                    }
                    Err(msg) => {
                        return Err(ParseError{
                            cause: Some(Box::new(msg)),
                            msg: format!("An error occurred while trying to parse multiply expression"),
                            pos: parser.current_token().pos,
                        })
                    }
                }
            }
            TokenType::Slash => {
                match binary::DivParser::try_parse(parser){
                    Ok(expr) => {
                        chunk.write_chunk(expr);
                    }
                    Err(msg) => {
                        return Err(ParseError{
                            cause: Some(Box::new(msg)),
                            msg: format!("An error occurred while trying to parse division expression"),
                            pos: parser.current_token().pos,
                        })
                    }
                }
            }
            _ => {
                match literal::LiteralParser::try_parse(parser){
                    Ok(literal) => chunk.write_chunk(literal),
                    Err(msg) => {
                        let pos = parser.next_token().pos;
                        return Err(ParseError{
                            cause: Some(Box::new(msg)),
                            msg: format!("An error occurred while trying to parse literal"),
                            pos,
                        })
                    }
                }
            }
            
        }
        Ok(chunk)
    }
}