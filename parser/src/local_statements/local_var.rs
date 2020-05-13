use crate::{
    Parser,
    ParseRule,
    ParseContext,
    expressions::ExpressionParser,
    type_::TypeParser,
};

use core::pos::BiPos;

use ir::{
    Chunk,
    hir::HIRInstruction
};

use ir_traits::WriteInstruction;

use lexer::tokens::{
    TokenType,
    TokenData,
};

use notices::NoticeLevel;

pub struct LocalVarParser;

impl ParseRule for LocalVarParser{
    fn parse(parser: &mut Parser) -> Result<(),()>{
        let mut chunk = Chunk::new();
        if parser.context != ParseContext::Local{
            parser.emit_notice(parser.current_token().pos, NoticeLevel::Error, "Found 'let' outside of local context.".to_string());
            return Err(())
        }
        if !parser.check_consume(TokenType::KwLet) {
            parser.emit_notice(
                parser.current_token().pos,
                NoticeLevel::Error,
                "Expected keyword 'let' for defining an local variable.".to_string(),
            );
        }
        chunk.write_instruction(HIRInstruction::LocalVar);
        let pos = parser.current_token().pos;
        chunk.write_pos(pos);
        if parser.check_consume_next(TokenType::KwMut){
            chunk.write_bool(true);
            chunk.write_pos(parser.current_token().pos);
        }else{
            chunk.write_bool(false);
            chunk.write_pos(BiPos::default());
        }
        if !parser.check(TokenType::Identifier) {
            let message = format!(
                "Expected an identifier token, but instead got {}",
                parser.current_token()
            );
            parser.emit_notice(pos, NoticeLevel::Error, message);
            return Err(());
        }
        let name = match &parser.current_token().data {
            TokenData::String(s) => (*s).to_string(),
            _ => {
                parser.emit_notice(
                    pos,
                    NoticeLevel::Error,
                    "Failed to extract string data from identifier token.".to_string(),
                );
                return Err(());
            }
        };
        chunk.write_string(name.clone());
        chunk.write_pos(parser.current_token().pos);
        if parser.next_token().type_ == TokenType::Colon {
            if let Ok(t) = TypeParser::get_type(parser){
                chunk.write_chunk(t)
            }else{
                parser.emit_notice(parser.current_token().pos, NoticeLevel::Error, "Could not create type signature for local variable.".to_string());
                return Err(())
            }
        } else {
            parser.advance()
                .expect("Failed to advance parser to next token.");
            chunk.write_pos(parser.current_token().pos);
            chunk.write_instruction(HIRInstruction::Unknown);
        }
        parser.emit_ir_whole(chunk);

        if !parser.check_consume(TokenType::Equal) {
            parser.emit_notice(
                pos,
                NoticeLevel::Error,
                "Local property must be initialized.".to_string(),
            );
            let found_token = parser.current_token();
            let data = match &found_token.data {
                TokenData::Float(f) => f.to_string(),
                TokenData::Integer(i) => i.to_string(),
                TokenData::String(s) => s.clone(),
                _ => "Unknown".to_string(),
            };
            parser.emit_notice(
                found_token.pos,
                NoticeLevel::Error,
                format!("Expected '=' but instead got {:?}", data),
            );
            return Err(());
        }

        if ExpressionParser::parse(parser).is_err() {
            parser.emit_notice(
                pos,
                NoticeLevel::Error,
                format!("Local variable {} cannot go uninitialized.", name.clone()),
            );
        }
        Ok(())
    }
}