use crate::{
    Parser,
    OwnedParse,
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

use notices::{
    Notice,
    NoticeLevel,
};

pub struct AddParser;

impl OwnedParse for AddParser{
    fn owned_parse(parser: &mut Parser) -> Result<Chunk, Notice>{
        if parser.next_token().type_ != TokenType::Plus{
            return Err(Notice::new(
                format!("Plus Parser"),
                format!("Attempted to parse an add operator binary expression but failed to find '+' token."),
                Some(parser.name.clone()),
                Some(parser.current_token().pos),
                NoticeLevel::Error,
                vec![]
            ));
        }
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Add);
        chunk.write_pos(parser.next_token().pos);

        match LiteralParser::owned_parse(parser){
            Ok(left) => {
                chunk.write_chunk(left);
            }
            Err(msg) => {
                return Err(msg)
            }
        }
        parser.advance().unwrap();
        parser.advance().unwrap();
        match ExpressionParser::owned_parse(parser){
            Ok(right) => {
                chunk.write_chunk(right);
            }
            Err(msg) => {
                return Err(msg)
            }
        }
        Ok(chunk)
    }
}

pub struct SubParser;

impl OwnedParse for SubParser{
    fn owned_parse(parser: &mut Parser) -> Result<Chunk, Notice>{
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Sub);
        chunk.write_pos(parser.next_token().pos);

        match LiteralParser::owned_parse(parser){
            Ok(left) => {
                chunk.write_chunk(left);
            }
            Err(msg) => {
                return Err(msg)
            }
        }
        parser.advance().unwrap();
        parser.advance().unwrap();
        match ExpressionParser::owned_parse(parser){
            Ok(right) => {
                chunk.write_chunk(right);
            }
            Err(msg) => {
                return Err(msg)
            }
        }
        Ok(chunk)
    }
}

pub struct MulParser;

impl OwnedParse for MulParser{
    fn owned_parse(parser: &mut Parser) -> Result<Chunk, Notice>{
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Sub);

        match LiteralParser::owned_parse(parser){
            Ok(left) => {
                chunk.write_chunk(left);
            }
            Err(msg) => {
                return Err(msg)
            }
        }
        parser.advance().unwrap();
        parser.advance().unwrap();
        match ExpressionParser::owned_parse(parser){
            Ok(right) => {
                chunk.write_chunk(right);
            }
            Err(msg) => {
                return Err(msg)
            }
        }
        Ok(chunk)
    }
}

pub struct DivParser;

impl OwnedParse for DivParser{
    fn owned_parse(parser: &mut Parser) -> Result<Chunk, Notice>{
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Mult);

        match LiteralParser::owned_parse(parser){
            Ok(left) => {
                chunk.write_chunk(left);
            }
            Err(msg) => {
                return Err(msg)
            }
        }
        parser.advance().unwrap();
        parser.advance().unwrap();
        match ExpressionParser::owned_parse(parser){
            Ok(right) => {
                chunk.write_chunk(right);
            }
            Err(msg) => {
                return Err(msg)
            }
        }
        Ok(chunk)
    }
}