use crate::{
    Parser,
    ParseRule,
    OwnedParse,
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

use notices::{
    NoticeLevel,
    Notice,
};

pub struct LocalVarParser;

impl ParseRule for LocalVarParser{
    fn parse(parser: &mut Parser) -> Result<(),Notice>{
        let mut chunk = Chunk::new();
        if parser.context != ParseContext::Local{
            return Err(Notice::new(
                format!("Local Parser"),
                format!("Found 'let' outside of local context. This is illegal."),
                Some(parser.name.clone()),
                Some(parser.current_token().pos),
                NoticeLevel::Error,
                vec![]
            ))
        }
        if let Err(_) = parser.check_consume(TokenType::KwLet) {
            return Err(Notice::new(
                format!("Local Parser"),
                format!("Expected keyword 'let' for defining an local variable."),
                Some(parser.name.clone()),
                Some(parser.current_token().pos),
                NoticeLevel::Error,
                vec![]
            ))
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
            return Err(Notice::new(
                format!("Local Parser"),
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
                return Err(Notice::new(
                    format!("Local Parser"),
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
        if parser.next_token().type_ == TokenType::Colon {
            if let Ok(t) = TypeParser::get_type(parser){
                chunk.write_chunk(t)
            }else{
                return Err(Notice::new(
                    format!("Local Parser"),
                    format!("Could not create type signature for local variable."),
                    Some(parser.name.clone()),
                    Some(parser.current_token().pos),
                    NoticeLevel::Error,
                    vec![]
                ));
            }
        } else {
            if let Err(notice) = parser.advance(){
                return Err(notice)
            };
            chunk.write_pos(parser.current_token().pos);
            chunk.write_instruction(HIRInstruction::Unknown);
        }
        parser.emit_ir_whole(chunk);

        if let Err(_) = parser.check_consume(TokenType::Equal) {
            let found_token = parser.current_token();
            let data = match &found_token.data {
                TokenData::Float(f) => f.to_string(),
                TokenData::Integer(i) => i.to_string(),
                TokenData::String(s) => s.clone(),
                _ => "Unknown".to_string(),
            };
            let cause = Notice::new(
                format!("Local Parser"),
                format!("Expected '=' but instead got {:?}", data),
                Some(parser.name.clone()),
                Some(parser.current_token().pos),
                NoticeLevel::Error,
                vec![]
            );
            return Err(Notice::new(
                format!("Local Parser"),
                format!("Local must be initialized."),
                Some(parser.name.clone()),
                Some(parser.current_token().pos),
                NoticeLevel::Error,
                vec![cause]
            ));
        }

        match ExpressionParser::owned_parse(parser) {
            Ok(expr) => parser.emit_ir_whole(expr),
            Err(cause) => {
                return Err(Notice::new(
                    format!("Local Parser"),
                    format!("Could not parse assigned expression for local"),
                    None,
                    None,
                    NoticeLevel::Error,
                    vec![cause]
                ))
            }
        }
        Ok(())
    }
}