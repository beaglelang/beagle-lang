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

use notices::{ DiagnosticLevel, DiagnosticSourceBuilder, DiagnosticBuilder };

use ir::{
    Chunk,
    hir::HIRInstruction,
};

use ir_traits::WriteInstruction;

pub struct FunctionParser;

impl ParseRule for FunctionParser{
    fn parse(parser: &mut Parser) -> Result<(),()>{
        let mut chunk = Chunk::new();
        let lpos = parser.current_token().pos;
        if let Err(_) = parser.check_consume(TokenType::KwFun) {
            let message = format!(
                "Expected a fun keyword token, but instead got {}",
                parser.current_token()
            );
            let source = match parser.request_source_snippet(){
                Ok(source) => {
                    DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                        .message(message)
                        .range(parser.current_token().pos.col_range())
                        .level(DiagnosticLevel::Error)
                        .source(source)
                        .build()
                },
                Err(diag) => 
                {
                    diag
                }
            };
            let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                .message(format!("An error ocurred while parsing a function."))
                .add_source(source)
                .build();
            parser.notice_tx.send(Some(diag)).unwrap();
            return Err(())
        }
        chunk.write_instruction(HIRInstruction::Fn);
        chunk.write_pos(lpos.clone());
        if !parser.check(TokenType::Identifier) {
            let message = format!(
                "Expected an identifier token, but instead got {}",
                parser.current_token()
            );
            let source = match parser.request_source_snippet(){
                Ok(source) => {
                    DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                        .message(message)
                        .range(parser.current_token().pos.col_range())
                        .level(DiagnosticLevel::Error)
                        .source(source)
                        .build()
                },
                Err(diag) => 
                {
                    diag
                }
            };
            let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                    .message(format!("An error ocurred while parsing a function."))
                    .add_source(source)
                    .build();
            parser.notice_tx.send(Some(diag)).unwrap();
            return Err(())
        }
        let name = match &parser.current_token().data {
            TokenData::String(s) => (*s).to_string(),
            _ => {
                let source = match parser.request_source_snippet(){
                    Ok(source) => {
                        DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                            .message(format!("Failed to extract string data from identifier token."))
                            .range(parser.current_token().pos.col_range())
                            .level(DiagnosticLevel::Error)
                            .source(source)
                            .build()
                    },
                    Err(diag) => 
                    {
                        diag
                    }
                };
                let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                    .message(format!("An error ocurred while parsing a function."))
                    .add_source(source)
                    .build();
                parser.notice_tx.send(Some(diag)).unwrap();
                return Err(())
            }
        };
        chunk.write_pos(parser.current_token().pos);
        chunk.write_string(name);
        match parser.advance(){
            Ok(()) => {},
            Err(source) => {
                let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                    .add_source(source)
                    .message(format!("An error occurred while parsing a function."))
                    .build();
                parser.notice_tx.send(Some(diag)).unwrap();
                return Err(())
            }
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
                        let source = match parser.request_source_snippet(){
                            Ok(source) => {
                                DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                                    .message(format!("Failed to extract string data from identifier token."))
                                    .range(parser.current_token().pos.col_range())
                                    .level(DiagnosticLevel::Error)
                                    .source(source)
                                    .build()
                            },
                            Err(diag) => 
                            {
                                diag
                            }
                        };
                        let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                            .message(format!("An error ocurred while parsing a function."))
                            .add_source(source)
                            .build();
                        parser.notice_tx.send(Some(diag)).unwrap();
                        return Err(())
                    },
                    Err(source) => {
                        let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                            .add_source(source)
                            .message(format!("An error occurred while parsing a function."))
                            .build();
                        parser.notice_tx.send(Some(diag)).unwrap();
                        return Err(())
                    },
                };
                param_chunk.write_pos(loc);
                param_chunk.write_string(param_name);
                let _ = parser.consume(TokenType::Colon);
                let param_type = TypeParser::get_type(parser).unwrap();
                param_chunk.write_chunk(param_type);
                chunk.write_chunk(param_chunk);
                match parser.advance(){
                    Ok(()) => {},
                    Err(source) => {
                        let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                            .add_source(source)
                            .message(format!("An error occurred while parsing a function."))
                            .build();
                        parser.notice_tx.send(Some(diag)).unwrap();
                        return Err(())
                    }
                }
            }
        }

        match parser.check_consume_next(TokenType::Colon){
            Ok(true) => {
                let retype_chunk = match TypeParser::get_type(parser){
                    Ok(chunk) => chunk,
                    Err(diag) => {
                        parser.emit_parse_diagnostic(&[], &[diag]);
                        return Err(())
                    }
                };
                chunk.write_chunk(retype_chunk);
            }
            Ok(false) => {
                chunk.write_pos(parser.current_token().pos);
                chunk.write_instruction(HIRInstruction::Unit);
            }
            Err(diag) => {
                let diagnostic = DiagnosticBuilder::new(DiagnosticLevel::Error)
                                    .message(format!("An error occurred while parsing a function."))
                                    .add_source(diag)
                                    .build();
                parser.notice_tx.send(Some(diagnostic)).unwrap();
                return Err(())
            }
        }

        parser.emit_ir_whole(chunk);

        if let Err(source) = parser.consume(TokenType::LCurly){
            parser.emit_parse_diagnostic(&[], &[source]);
            return Err(());
        }

        parser.context = ParseContext::Local;
        
        let mut body_chunk = Chunk::new();
        body_chunk.write_instruction(HIRInstruction::Block);
        body_chunk.write_pos(parser.current_token().pos);
        parser.emit_ir_whole(body_chunk);
        while let Ok(false) = parser.check_consume(TokenType::RCurly){
            match parser.advance(){
                Ok(()) => {},
                Err(source) => {
                    let diag = DiagnosticBuilder::new(DiagnosticLevel::Error)
                        .add_source(source)
                        .message(format!("An error occurred while parsing a function."))
                        .build();
                    parser.notice_tx.send(Some(diag)).unwrap();
                    return Err(())
                }
            }
            if let Err(()) = LocalStatementParser::parse(parser){
                return Err(())
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