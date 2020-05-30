use super::{
    Parser,
    ParseRule,
    local_statements::LocalStatementParser,
    type_::TypeParser,
    ParseContext,
};

use lexer::tokens::{
    TokenData,
    TokenType,
};

use notices::{ NoticeLevel, Notice };

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::WriteInstruction;

pub struct FunctionParser;

impl ParseRule for FunctionParser{
    fn parse(parser: &mut Parser) -> Result<(),Notice>{
        let mut chunk = Chunk::new();
        let lpos = parser.current_token().pos;
        if !parser.check_consume(TokenType::KwFun) {
            let message = format!(
                "Expected a fun keyword token, but instead got {}",
                parser.current_token()
            );
            parser.emit_notice(lpos, NoticeLevel::Error, message);
            return Err(Notice::new(
                format!("Function Parser"), 
                format!(
                    "Expected a fun keyword token, but instead got {}",
                    parser.current_token()
                ), 
                Some(parser.name.clone()), 
                Some(lpos), 
                NoticeLevel::Error,
                vec![]
            ));
        }
        chunk.write_instruction(HIRInstruction::Fn);
        chunk.write_pos(lpos.clone());
        if !parser.check(TokenType::Identifier) {
            let message = format!(
                "Expected an identifier token, but instead got {}",
                parser.current_token()
            );
            parser.emit_notice(lpos, NoticeLevel::Error, message);
            return Err(Notice::new(
                format!("Function Parser"), 
                format!(
                    "Expected an identifier token, but instead got {}",
                    parser.current_token()
                ), 
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
                    format!("Function Parser"), 
                    format!(
                        "Failed to extract string data from identifier token.",
                    ), 
                    Some(parser.name.clone()), 
                    Some(parser.current_token().pos), 
                    NoticeLevel::Error,
                    vec![]
                ));
            }
        };
        chunk.write_string(name);
        chunk.write_pos(parser.current_token().pos);
        if parser.advance().is_err(){
            return Err(Notice::new(
                format!("Function Parser"), 
                format!(
                    "Failed to advance parser. This is a bug in the compiler, please report to the author.\n
                    It's possible that the tokenizer closed its channel prematurely, which shouldn't happen.",
                ), 
                Some(parser.name.clone()), 
                Some(parser.current_token().pos), 
                NoticeLevel::Error,
                vec![]
            ));
        }
        if parser.check(TokenType::LParen){
            loop{
                if parser.check(TokenType::RParen){
                    chunk.write_instruction(HIRInstruction::EndParams);
                    break;
                }
                let mut param_chunk = Chunk::new();
                param_chunk.write_instruction(HIRInstruction::FnParam);
                let loc = parser.next_token().pos;
                let param_name = match parser.consume(TokenType::Identifier) {
                    Ok(TokenData::String(s)) => (*s).to_string(),
                    Ok(_) => {
                        return Err(Notice::new(
                            format!("Function Parser"), 
                            format!(
                                "Failed to extract string data from identifier token.",
                            ), 
                            Some(parser.name.clone()), 
                            Some(parser.current_token().pos), 
                            NoticeLevel::Error,
                            vec![]
                        ));
                    },
                    Err(notice) => return Err(notice),
                };
                param_chunk.write_pos(loc);
                param_chunk.write_string(param_name);
                let _ = parser.consume(TokenType::Colon);
                let param_type = TypeParser::get_type(parser).unwrap();
                param_chunk.write_chunk(param_type);
                chunk.write_chunk(param_chunk);
                parser.advance().unwrap();
            }
        }

        if parser.check_consume_next(TokenType::Colon){
            let retype_chunk = TypeParser::get_type(parser)?;
            chunk.write_chunk(retype_chunk);
        }else{
            chunk.write_pos(parser.current_token().pos);
            chunk.write_instruction(HIRInstruction::Unit);
        }

        parser.emit_ir_whole(chunk);

        if let Err(notice) = parser.consume(TokenType::LCurly){
            return Err(notice);
        }

        parser.context = ParseContext::Local;
        
        let mut body_chunk = Chunk::new();
        body_chunk.write_instruction(HIRInstruction::Block);
        body_chunk.write_pos(parser.current_token().pos);
        parser.emit_ir_whole(body_chunk);
        while !parser.check_consume(TokenType::RCurly){
            parser.advance().unwrap();
            if let Err(notice) = LocalStatementParser::parse(parser){
                return Err(notice)
            }
        }
        let mut end_chunk = Chunk::new();
        end_chunk.write_instruction(HIRInstruction::EndBlock);
        end_chunk.write_pos(parser.prev_token().pos);
        end_chunk.write_instruction(HIRInstruction::EndFn);

        parser.emit_ir_whole(end_chunk);
        Ok(())
    }
}