use super::{
    ParseRule,
    Parser,
    expressions::ExpressionParser,
    type_::TypeParser,
};

use lexer::tokens::{
    TokenData,
    TokenType,
};

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::WriteInstruction;

use notices::NoticeLevel;

pub struct PropertyParser;

impl ParseRule for PropertyParser{
    fn parse(parser: &mut Parser) -> Result<(),()>{
        let mut chunk = Chunk::new();
        chunk.write_instruction(HIRInstruction::Property);
        let lpos = parser.current_token().pos;
        chunk.write_pos(lpos);
        let mutable = if !parser.check(TokenType::KwVal) {
            if !parser.check(TokenType::KwVar){
                let message = format!(
                    "Expected a val or var keyword token, but instead got {}",
                    parser.current_token()
                );
                parser.emit_notice(lpos, NoticeLevel::Error, message);
                return Err(());
            }
            true
        }else{
            false
        };
        chunk.write_bool(mutable);
        parser.advance().unwrap();
        if !parser.check(TokenType::Identifier) {
            let message = format!(
                "Expected an identifier token, but instead got {}",
                parser.current_token()
            );
            parser.emit_notice(lpos, NoticeLevel::Error, message);
            return Err(());
        }
        let name = match &parser.current_token().data {
            TokenData::String(s) => (*s).to_string(),
            _ => {
                parser.emit_notice(
                    lpos,
                    NoticeLevel::Error,
                    "Failed to extract string data from identifier token.".to_string(),
                );
                return Err(());
            }
        };

        chunk.write_string(name.clone());
        if parser.check_consume_next(TokenType::Colon) {
            if let Ok(t) = TypeParser::get_type(parser){
                chunk.write_chunk(t);
            }else{
                parser.emit_notice(parser.prev_token().pos, NoticeLevel::Error, "Could not create type signature for property.".to_string());
                return Err(())
            }
            parser.advance()
                .expect("Failed to advance parser to next token.");
        } else {
            parser.advance()
                .expect("Failed to advance parser to next token.");
            chunk.write_instruction(HIRInstruction::Unknown);
        };

        if !parser.check_consume(TokenType::Equal) {
            parser.emit_notice(
                lpos,
                NoticeLevel::Error,
                "Value property must be initialized.".to_string(),
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

        parser.emit_ir_whole(chunk);
        
        ExpressionParser::parse(parser).expect("Could not parse expression.");
        Ok(())
    }
}