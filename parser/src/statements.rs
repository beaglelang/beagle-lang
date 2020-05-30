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
    NoticeLevel,
    Notice
};

pub struct StatementParser;

impl ParseRule for StatementParser{
    fn parse(parser: &mut Parser) -> Result<(), Notice>{
        let token = parser.current_token();
        match token.type_ {
            TokenType::KwMod => ModuleParser::parse(parser)?,
            TokenType::KwVal => PropertyParser::parse(parser)?,
            TokenType::KwVar => PropertyParser::parse(parser)?,
            TokenType::KwFun => FunctionParser::parse(parser)?,
            _ => {
                return Err(Notice::new(
                    format!("Statement Parser"),
                    format!("Unexpected token found: {}", token),
                    Some(parser.name.clone()),
                    Some(parser.current_token().pos),
                    NoticeLevel::Error,
                    vec![]
                ));
            }
        }
        Ok(())
    }
}