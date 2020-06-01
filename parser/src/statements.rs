use crate::{
    ParseRule,
    Parser,
    properties::PropertyParser,
    functions::FunctionParser,
    modules::ModuleParser,
};

use lexer::tokens::{
    TokenType,
};

use notices::{
    DiagnosticSourceBuilder,
    DiagnosticLevel
};

pub struct StatementParser;

impl ParseRule for StatementParser{
    fn parse(parser: &mut Parser) -> Result<(), ()>{
        let token = parser.current_token();
        match token.type_ {
            TokenType::KwMod => ModuleParser::parse(parser)?,
            TokenType::KwVal => PropertyParser::parse(parser)?,
            TokenType::KwVar => PropertyParser::parse(parser)?,
            TokenType::KwFun => FunctionParser::parse(parser)?,
            _ => {
                let source = match parser.request_source_snippet(){
                    Ok(source) => source,
                    Err(diag) => {
                        parser.emit_parse_diagnostic(&[], &[diag]);
                        return Err(())
                    }
                };
                let diag_source = DiagnosticSourceBuilder::new(parser.name.clone(), token.pos.start.0)
                    .level(DiagnosticLevel::Error)
                    .message(format!("Unexpected token found: {:?}", token.type_))
                    .range(token.pos.col_range())
                    .source(source)
                    .build();
                parser.emit_parse_diagnostic(&[], &[diag_source]);
                return Err(());
            }
        }
        Ok(())
    }
}