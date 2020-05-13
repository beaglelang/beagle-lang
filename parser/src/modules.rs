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

pub struct ModuleParser;

impl ParseRule for ModuleParser{
    fn parse(parser: &mut Parser) -> Result<(),()>{
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Module);
        if let Ok(TokenData::String(ident)) = parser.consume(TokenType::Identifier) {
            chunk.write_string(ident.clone());
        }
        parser.emit_ir_whole(chunk);
        while !parser.check(TokenType::RCurly) {
            if let Err(()) = StatementParser::parse(parser) {
                return Err(());
            }
        }
        let mut end_chunk = Chunk::new();
        end_chunk.write_instruction(HIRInstruction::EndModule);
        parser.emit_ir_whole(end_chunk);
        Ok(())
    }
}