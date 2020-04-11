use crate::{
    parser::Parser,
    lexer::tokens::TokenType
};

pub(crate) type ParseFn = fn(&mut Parser) -> Result<(), ()>;

#[derive(Clone)]
pub(crate) struct ParseRule{
    pub(crate) prefix: ParseFn,
    pub(crate) infix: ParseFn,
}

const PARSER_RULE_TABLE: [ParseRule; TokenType::Eof as usize + 1] = [
    // ParseRule{
    //     prefix: grouping_or_fn
    // }
];