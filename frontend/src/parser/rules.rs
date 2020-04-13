use crate::{
    lexer::tokens::TokenType,
    parser::{
        functions::{literal, nil_func},
        Parser,
    },
};

use lazy_static::lazy_static;

use std::collections::HashMap;

pub(crate) type ParseFn = fn(&mut Parser) -> Result<(), ()>;

#[derive(Clone)]
pub struct ParseRule {
    pub(crate) prefix: ParseFn,
    pub(crate) infix: ParseFn,
}

lazy_static! {
    pub static ref PARSER_RULE_TABLE: HashMap<&'static TokenType, ParseRule> = {
        let mut m = HashMap::new();
        m.insert(
            &TokenType::String,
            ParseRule {
                prefix: nil_func,
                infix: literal,
            },
        );
        m.insert(
            &TokenType::Number,
            ParseRule {
                prefix: nil_func,
                infix: literal,
            },
        );
        m
    };
}
