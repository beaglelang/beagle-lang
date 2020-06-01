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
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

pub struct LocalVarParser;

impl ParseRule for LocalVarParser{
    fn parse(parser: &mut Parser) -> Result<(),()>{
        let mut chunk = Chunk::new();
        if parser.context != ParseContext::Local{
            let source = match parser.request_source_snippet(){
                Ok(source) => source,
                Err(diag) => {
                    parser.emit_parse_diagnostic(&[], &[diag]);
                    return Err(())
                }
            };
            let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                .level(DiagnosticLevel::Error)
                .message(format!("Found 'let' outside of local context. This is illegal."))
                .range(parser.current_token().pos.col_range())
                .source(source)
                .build();
            parser.emit_parse_diagnostic(&[], &[diag_source]);
            return Err(())
        }
        match parser.check_consume(TokenType::KwLet){
            Ok(true) => {},
            Ok(false) => {
                let source = match parser.request_source_snippet(){
                    Ok(source) => source,
                    Err(diag) => {
                        parser.emit_parse_diagnostic(&[], &[diag]);
                        return Err(())
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Expected keyword 'let' for defining an local variable."))
                    .source(source)
                    .build();
                parser.emit_parse_diagnostic(&[], &[diag_source]);
                return Err(())
            }
            Err(diag) => {
                parser.emit_parse_diagnostic(&[], &[diag]);
                return Err(())
            }
        }
        chunk.write_instruction(HIRInstruction::LocalVar);
        let pos = parser.current_token().pos;
        chunk.write_pos(pos);
        match parser.check_consume_next(TokenType::KwMut){
            Ok(true) => {
                chunk.write_bool(true);
                chunk.write_pos(parser.current_token().pos);
            }
            Ok(false) => {
                chunk.write_bool(false);
                chunk.write_pos(BiPos::default());
            }
            Err(diag) => {
                parser.emit_parse_diagnostic(&[], &[diag]);
                return Err(())
            }
        }
        if !parser.check(TokenType::Identifier) {
            let message = format!(
                "Expected an identifier token, but instead got {}",
                parser.current_token()
            );
            let source = match parser.request_source_snippet(){
                Ok(source) => source,
                Err(diag) => {
                    parser.emit_parse_diagnostic(&[], &[diag]);
                    return Err(())
                }
            };
            let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(message)
                    .source(source)
                    .build();
            parser.emit_parse_diagnostic(&[], &[diag_source]);
            return Err(());
        }
        let name = match &parser.current_token().data {
            TokenData::String(s) => (*s).to_string(),
            _ => {
               let message = format!(
                    "Failed to extract string data from identifier token.",
                );
                let source = match parser.request_source_snippet(){
                    Ok(source) => source,
                    Err(diag) => {
                        parser.emit_parse_diagnostic(&[], &[diag]);
                        return Err(())
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(message)
                    .range(parser.current_token().pos.col_range())
                    .source(source)
                    .build();
                parser.emit_parse_diagnostic(&[], &[diag_source]);
                return Err(());
            }
        };
        chunk.write_pos(parser.current_token().pos);
        chunk.write_string(name.clone());
        if parser.next_token().type_ == TokenType::Colon {
            match TypeParser::get_type(parser){
                Ok(t) =>chunk.write_chunk(t),
                Err(diag) => {
                    let message = format!(
                        "Could not parse type signature for property.",
                    );
                    let source = match parser.request_source_snippet(){
                        Ok(source) => source,
                        Err(diag) => {
                            parser.emit_parse_diagnostic(&[], &[diag]);
                            return Err(())
                        }
                    };
                    let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                        .level(DiagnosticLevel::Error)
                        .message(message)
                        .range(parser.current_token().pos.col_range())
                        .source(source)
                        .build();
                    parser.emit_parse_diagnostic(&[], &[diag, diag_source]);
                    return Err(());
                }
            }
        } else {
            if let Err(source) = parser.advance(){
                parser.emit_parse_diagnostic(&[], &[source]);
                return Err(())
            };
            chunk.write_pos(parser.current_token().pos);
            chunk.write_instruction(HIRInstruction::Unknown);
        }
        parser.emit_ir_whole(chunk);

        if let Ok(false) = parser.check_consume(TokenType::Equal) {
            let found_token = parser.current_token();
            let data = &found_token.data;
            let cause_message = format!("Expected '=' but instead got {:?}", data);
            let cause_source = match parser.request_source_snippet(){
                Ok(source) => source,
                Err(diag) => {
                    parser.emit_parse_diagnostic(&[], &[diag]);
                    return Err(())
                }
            };
            let cause_diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                .level(DiagnosticLevel::Error)
                .message(cause_message)
                .range(parser.current_token().pos.col_range())
                .source(cause_source)
                .build();

            let message = format!("Local must be initialized");
            let source = match parser.request_source_snippet(){
                Ok(source) => source,
                Err(diag) => {
                    parser.emit_parse_diagnostic(&[], &[diag]);
                    return Err(())
                }
            };
            let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), parser.current_token().pos.start.0)
                .level(DiagnosticLevel::Error)
                .message(message)
                .range(parser.current_token().pos.col_range())
                .source(source)
                .build();
            parser.emit_parse_diagnostic(&[], &[cause_diag_source, diag_source]);
            return Err(());
        }

        match ExpressionParser::owned_parse(parser) {
            Ok(expr) => parser.emit_ir_whole(expr),
            Err(cause) => {
                parser.emit_parse_diagnostic(&[], &[cause]);
                return Err(())
            }
        }
        Ok(())
    }
}