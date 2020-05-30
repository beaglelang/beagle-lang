use super::{
    Parser,
    ParseRule,
    statements::StatementParser,
};

use lexer::tokens::{
    TokenType,
};

mod local_var;
use local_var::LocalVarParser;

use notices::{
    Notice,
};

pub struct LocalStatementParser;

impl ParseRule for LocalStatementParser{
    fn parse(parser: &mut Parser) -> Result<(),Notice>{
        match parser.current_token().type_{
            TokenType::RCurly => return Ok(()),
            TokenType::KwLet => LocalVarParser::parse(parser)?,
            TokenType::Identifier => {
                match parser.next_token().type_{
                    TokenType::Equal => unimplemented!(),
                    _ => unimplemented!()
                };
            },
            _ => StatementParser::parse(parser)?
        };
        // p.advance().unwrap();
        Ok(())
    }
}