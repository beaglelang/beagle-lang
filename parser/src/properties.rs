use super::{
    ParseRule,
    Parser,
    OwnedParse,
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

use notices::{
    NoticeLevel,
    Notice
};

pub struct PropertyParser;

impl ParseRule for PropertyParser{
    fn parse(parser: &mut Parser) -> Result<(),Notice>{
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
                return Err(Notice::new(
                    format!("Property Parser"),
                    message,
                    Some(parser.name.clone()),
                    Some(parser.current_token().pos),
                    NoticeLevel::Error,
                    vec![]
                ));
            }
            true
        }else{
            false
        };
        chunk.write_bool(mutable);
        chunk.write_pos(lpos);
        parser.advance().unwrap();
        if !parser.check(TokenType::Identifier) {
            let message = format!(
                "Expected an identifier token, but instead got {}",
                parser.current_token()
            );
            return Err(Notice::new(
                format!("Property Parser"),
                message,
                Some(parser.name.clone()),
                Some(parser.current_token().pos),
                NoticeLevel::Error,
                vec![]
            ));
        }
        let name = match &parser.current_token().data {
            TokenData::String(s) => (*s).to_string(),
            _ => {
                parser.emit_notice(
                    lpos,
                    NoticeLevel::Error,
                    "Failed to extract string data from identifier token.".to_string(),
                );
                return Err(Notice::new(
                    format!("Property Parser"),
                    format!("Failed to extract string data from identifier token."),
                    Some(parser.name.clone()),
                    Some(parser.current_token().pos),
                    NoticeLevel::Error,
                    vec![]
                ));
            }
        };

        chunk.write_pos(parser.current_token().pos);
        chunk.write_string(name.clone());
        if parser.check_consume_next(TokenType::Colon) {
            if let Ok(t) = TypeParser::get_type(parser){
                chunk.write_chunk(t);
            }else{
                return Err(Notice::new(
                    format!("Property Parser"),
                    format!("Could not create type signature for property."),
                    Some(parser.name.clone()),
                    Some(parser.current_token().pos),
                    NoticeLevel::Error,
                    vec![]
                ));
            }
            if let Err(notice) = parser.advance(){
                return Err(notice);
            };
        } else {
            if let Err(notice) = parser.advance(){
                return Err(notice);
            };
            chunk.write_instruction(HIRInstruction::Unknown);
        };

        if let Err(_) = parser.check_consume(TokenType::Equal) {
            let found_token = parser.current_token();
            let data = match &found_token.data {
                TokenData::Float(f) => f.to_string(),
                TokenData::Integer(i) => i.to_string(),
                TokenData::String(s) => s.clone(),
                _ => "Unknown".to_string(),
            };
            let cause = Notice::new(
                format!("Property Parser"),
                format!("Expected '=' but instead got {:?}", data),
                Some(parser.name.clone()),
                Some(parser.current_token().pos),
                NoticeLevel::Error,
                vec![]
            );
            return Err(Notice::new(
                format!("Property Parser"),
                format!("Property must be initialized."),
                Some(parser.name.clone()),
                Some(parser.current_token().pos),
                NoticeLevel::Error,
                vec![cause]
            ));
        }

        parser.emit_ir_whole(chunk);
        
        match ExpressionParser::owned_parse(parser){
            Ok(expr) => {
                parser.emit_ir_whole(expr);
            }
            Err(msg) => {
                return Err(msg);
            }
        }
        parser.advance().unwrap();
        Ok(())
    }
}