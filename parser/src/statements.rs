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

use notices::NoticeLevel;

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
                parser.emit_notice(
                    token.pos,
                    NoticeLevel::Error,
                    format!("Unexpected token found: {}", token).to_string(),
                );
                return Err(());
            }
        }
        Ok(())
    }
}