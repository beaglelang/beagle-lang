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
    DiagnosticSourceBuilder,
    DiagnosticLevel,
};

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
                let source = match parser.request_source_snippet(lpos){
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
            let source = match parser.request_source_snippet(parser.current_token().pos){
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
        let name = match &parser.current_token().data {
            TokenData::String(s) => (*s).to_string(),
            _ => {
               let message = format!(
                    "Failed to extract string data from identifier token.",
                );
                let source = match parser.request_source_snippet(parser.current_token().pos){
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
        match parser.check_consume_next(TokenType::Colon) {
            Ok(true) => {
                match TypeParser::get_type(parser){
                    Ok(t) =>chunk.write_chunk(t),
                    Err(diag) => {
                        let message = format!(
                            "Could not parse type signature for property.",
                        );
                        let source = match parser.request_source_snippet(parser.current_token().pos){
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
                if let Err(diag_source) = parser.advance(){
                    parser.emit_parse_diagnostic(&[], &[diag_source]);
                    return Err(());
                };
            }
            Ok(false) => {
                if let Err(diag_source) = parser.advance(){
                    parser.emit_parse_diagnostic(&[], &[diag_source]);
                    return Err(());
                };
                chunk.write_instruction(HIRInstruction::Unknown);
            }
            Err(diag) => {
                parser.emit_parse_diagnostic(&[], &[diag]);
                return Err(())
            }
        }

        if let Ok(false) = parser.check_consume(TokenType::Equal) {
            let found_token = parser.current_token();
            let data = &found_token.data;
            let cause_message = format!("Expected '=' but instead got {:?}", data);
            let cause_source = match parser.request_source_snippet(found_token.pos){
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

            let message = format!("Property must be initialized");
            let source = match parser.request_source_snippet(found_token.pos){
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

        parser.emit_ir_whole(chunk);
        
        match ExpressionParser::owned_parse(parser){
            Ok(expr) => {
                parser.emit_ir_whole(expr);
            }
            Err(msg) => {
                parser.emit_parse_diagnostic(&[], &[msg]);
                return Err(());
            }
        }
        parser.advance().unwrap();
        Ok(())
    }
}