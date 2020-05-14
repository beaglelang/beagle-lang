use crate::{
    Parser,
    TryParse,
    ParseError,
    expressions::{
        ExpressionParser,
        literal::LiteralParser,
    },
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use lexer::tokens::{
    TokenType,
};

use ir_traits::WriteInstruction;

pub struct AddParser;

impl TryParse for AddParser{
    fn try_parse(parser: &mut Parser) -> Result<Chunk, ParseError>{
        if parser.next_token().type_ != TokenType::Plus{
            return Err(ParseError{
                cause: None,
                msg: format!("Attempted to parse an add operator binary expression but failed to find '+' token."),
                pos: parser.next_token().pos,
            })
        }
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Add);
        chunk.write_pos(parser.next_token().pos);

        match LiteralParser::try_parse(parser){
            Ok(left) => {
                chunk.write_chunk(left);
            }
            Err(msg) => {
                return Err(ParseError{
                    cause: Some(Box::new(msg)),
                    msg: format!("An error occurred while trying to parse left hand expression of add operation"),
                    pos: parser.current_token().pos,
                })
            }
        }
        parser.advance().unwrap();
        parser.advance().unwrap();
        match ExpressionParser::try_parse(parser){
            Ok(right) => {
                chunk.write_chunk(right);
            }
            Err(msg) => {
                return Err(ParseError{
                    cause: Some(Box::new(msg)),
                    msg: format!("An error occurred while trying to parse right hand expression of add operation"),
                    pos: parser.current_token().pos,
                })
            }
        }
        Ok(chunk)
    }
}

pub struct SubParser;

impl TryParse for SubParser{
    fn try_parse(parser: &mut Parser) -> Result<Chunk, ParseError>{
        if parser.next_token().type_ != TokenType::Minus{
            return Err(ParseError{
                cause: None,
                msg: format!("Attempted to parse a sub operator binary expression but failed to find '-' token."),
                pos: parser.next_token().pos,
            })
        }
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Sub);
        chunk.write_pos(parser.next_token().pos);

        match LiteralParser::try_parse(parser){
            Ok(left) => {
                chunk.write_chunk(left);
            }
            Err(msg) => {
                return Err(ParseError{
                    cause: Some(Box::new(msg)),
                    msg: format!("An error occurred while trying to parse left hand expression of add operation"),
                    pos: parser.current_token().pos,
                })
            }
        }
        parser.advance().unwrap();
        parser.advance().unwrap();
        match ExpressionParser::try_parse(parser){
            Ok(right) => {
                chunk.write_chunk(right);
            }
            Err(msg) => {
                return Err(ParseError{
                    cause: Some(Box::new(msg)),
                    msg: format!("An error occurred while trying to parse right hand expression of add operation"),
                    pos: parser.current_token().pos,
                })
            }
        }
        Ok(chunk)
    }
}

pub struct MulParser;

impl TryParse for MulParser{
    fn try_parse(parser: &mut Parser) -> Result<Chunk, ParseError>{
        if parser.next_token().type_ != TokenType::Star{
            return Err(ParseError{
                cause: None,
                msg: format!("Attempted to parse an multiply operator binary expression but failed to find '*' token."),
                pos: parser.next_token().pos,
            })
        }
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Sub);

        match LiteralParser::try_parse(parser){
            Ok(left) => {
                chunk.write_chunk(left);
            }
            Err(msg) => {
                return Err(ParseError{
                    cause: Some(Box::new(msg)),
                    msg: format!("An error occurred while trying to parse left hand expression of add operation"),
                    pos: parser.current_token().pos,
                })
            }
        }
        parser.advance().unwrap();
        parser.advance().unwrap();
        match ExpressionParser::try_parse(parser){
            Ok(right) => {
                chunk.write_chunk(right);
            }
            Err(msg) => {
                return Err(ParseError{
                    cause: Some(Box::new(msg)),
                    msg: format!("An error occurred while trying to parse right hand expression of add operation"),
                    pos: parser.current_token().pos,
                })
            }
        }
        Ok(chunk)
    }
}

pub struct DivParser;

impl TryParse for DivParser{
    fn try_parse(parser: &mut Parser) -> Result<Chunk, ParseError>{
        if parser.next_token().type_ != TokenType::Slash{
            return Err(ParseError{
                cause: None,
                msg: format!("Attempted to parse a div operator binary expression but failed to find '/' token."),
                pos: parser.next_token().pos,
            })
        }
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Mult);

        match LiteralParser::try_parse(parser){
            Ok(left) => {
                chunk.write_chunk(left);
            }
            Err(msg) => {
                return Err(ParseError{
                    cause: Some(Box::new(msg)),
                    msg: format!("An error occurred while trying to parse left hand expression of add operation"),
                    pos: parser.current_token().pos,
                })
            }
        }
        parser.advance().unwrap();
        parser.advance().unwrap();
        match ExpressionParser::try_parse(parser){
            Ok(right) => {
                chunk.write_chunk(right);
            }
            Err(msg) => {
                return Err(ParseError{
                    cause: Some(Box::new(msg)),
                    msg: format!("An error occurred while trying to parse right hand expression of add operation"),
                    pos: parser.current_token().pos,
                })
            }
        }
        Ok(chunk)
    }
}