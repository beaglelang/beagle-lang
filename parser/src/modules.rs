use super::{
    ParseRule,
    Parser,
    statements::StatementParser,
};

use lexer::tokens::{
    TokenType,
    TokenData,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::WriteInstruction;

use notices::{
    Notice,
    NoticeLevel,
};

pub struct ModuleParser;

impl ParseRule for ModuleParser{
    fn parse(parser: &mut Parser) -> Result<(),Notice>{
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Module);
        let ident = if let Ok(TokenData::String(ident)) = parser.consume(TokenType::Identifier) {
            chunk.write_string(ident.clone());
            ident.clone()
        }else{
            return Err(
                Notice::new(
                    format!("Module Parser"),
                    format!("Expected an identifier token."),
                    Some(parser.name.clone()),
                    Some(parser.current_token().pos),
                    NoticeLevel::Error,
                    vec![]
                )
            )
        };
        parser.emit_ir_whole(chunk);
        while !parser.check(TokenType::RCurly) {
            if let Err(notice) = StatementParser::parse(parser) {
                return Err(Notice::new(
                    format!("Module Parser"),
                    format!("An error occurred while parsing statement."),
                    Some(ident.clone()),
                    Some(parser.current_token().pos),
                    NoticeLevel::Error,
                    vec![notice]
                ));
            }
        }
        let mut end_chunk = Chunk::new();
        end_chunk.write_instruction(HIRInstruction::EndModule);
        parser.emit_ir_whole(end_chunk);
        Ok(())
    }
}