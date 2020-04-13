use crate::{
    parser::{functions, rules},
    tokens::{LexerToken, TokenData, TokenType},
    Parser,
};

use ir::{
    hir::Instruction,
    type_signature::{PrimitiveType, TypeSignature},
};

use core::pos::BiPos as Position;
use notices::{
    NoticeLevel,
};

type IRError = Result<(), ()>;

pub(crate) fn nil_func<'a>(p: &mut Parser<'a>) -> IRError {
    panic!("This is a placehold func for parse rules. This represents a null pointer function. This should not have been called.")
}

pub fn module<'a>(p: &mut Parser<'a>) -> IRError {
    p.emit_ir(
        Position::default(),
        TypeSignature::None,
        Instruction::Module(p.name.clone()),
    );
    while !p.check(TokenType::Eof) {
        if let Err(()) = declaration_or_statement(p) {
            return Err(())
        }
        p.advance().unwrap();
    }
    p.emit_ir(Position::default(), TypeSignature::None, Instruction::Halt);
    Ok(())
}

pub(crate) fn declaration_or_statement<'a>(p: &mut Parser<'a>) -> IRError {
    match p.current_token().type_ {
        TokenType::KwMod => declaration(p),
        _ => statement(p),
    }
}

pub(crate) fn declaration<'a>(p: &mut Parser<'a>) -> IRError {
    Ok(())
}

pub(crate) fn statement<'a>(p: &mut Parser<'a>) -> IRError {
    let token = p.current_token();
    match token.type_ {
        TokenType::KwVal => property_val(p)?,
        TokenType::KwVar => property_var(p)?,
        TokenType::KwLet => local_var(p)?,
        _ => {
            p.emit_notice(token.pos, NoticeLevel::Error, format!("Unexpected token found: {:?}", token).to_string());
            return Err(())
        }
    }
    Ok(())
}

pub(crate) fn property_val<'a>(p: &mut Parser<'a>) -> IRError {
    let lpos = p.current_token().pos;
    if !p.check_consume(
        TokenType::KwVal
    )
    {
        let message = format!(
            "Expected a val keyword token, but instead got {}",
            p.current_token()
        );
        p.emit_notice(lpos, NoticeLevel::Error, message);
        return Err(());
    }
    if !p.check(TokenType::Identifier) {
        let message = format!(
            "Expected an identifier token, but instead got {}",
            p.current_token()
        );
        p.emit_notice(lpos, NoticeLevel::Error, message);
        return Err(());
    }
    let name = match p.current_token().data {
        TokenData::Str(s) => (*s).to_string(),
        _ => {
            p.emit_notice(lpos, NoticeLevel::Error, "Failed to extract string data from identifier token.".to_string());
            return Err(())
        },
    };
    let (signature, is_untyped) = if p.check_consume(TokenType::Colon) {
        (
            type_(p).expect("Could not create type signature for value property."),
            false,
        )
    } else {
        p.advance()
            .expect("Failed to advance parser to next token.");
        (TypeSignature::Untyped, false)
    };

    if !p.check_consume(TokenType::Equal) {
        p.emit_notice(lpos, NoticeLevel::Error, "Value property must be initialized.".to_string());
        let found_token = match &p.current_token().data{
            TokenData::Float(f) => f.to_string(),
            TokenData::Integer(i) => i.to_string(),
            TokenData::Str(s) => (*s).to_string(),
            TokenData::String(s) => s.clone(),
            &_ => "Unknown".to_string()
        };
        p.emit_notice(lpos, NoticeLevel::Error, format!("Expected '=' but instead got {:?}", found_token));
        return Err(());
    }
    p.emit_ir(lpos, signature, Instruction::Property(name, false));
    expression(p).expect("Could not parse expression.");
    Ok(())
}

pub(crate) fn property_var<'a>(p: &mut Parser<'a>) -> IRError {
    let was_error = p
        .consume(
            TokenType::KwVar,
            "Expected keyword 'var' for defining an mutable property.",
        )
        .is_err();
    Ok(())
}

pub(crate) fn local_var<'a>(p: &mut Parser<'a>) -> IRError {
    if !p.check_consume(TokenType::KwLet){
        p.emit_notice(
            p.current_token().pos, 
            NoticeLevel::Error, 
            "Expected keyword 'let' for defining an immutable local variable.".to_string());
    }
    let token = p.prev_token();
    let pos = token.pos;
    let name = if p.current_token().type_ != TokenType::Identifier{
        p.emit_notice(pos, NoticeLevel::Error, "Expected an identifier for value property".to_string());
        return Err(())
    }else {
        match &token.data{
            TokenData::Str(s) => (*s).to_string(),
            _ => {
                p.emit_notice(pos, NoticeLevel::Error, "Failed to extract string data from identifier token.".to_string());
                return Err(()) 
            },
        }
    };
    let (signature, is_untyped) = if p.check_consume(TokenType::Colon) {
        (
            type_(p).expect("Could not create type signature for local variable."),
            false,
        )
    } else {
        (TypeSignature::Untyped, false)
    };
    p.emit_ir(pos, signature, Instruction::LocalVar(name.clone(), false));

    match p.consume(TokenType::Equal, "Expected an EQUAL token.") {
        Ok(t) => {
            p.advance().unwrap();
            if expression(p).is_err() {
                p.emit_notice(pos, NoticeLevel::Error, format!("Local variable {} cannot go uninitialized.", name));
            }
        },
        Err(()) => return Err(())
    }
    Ok(())
}

fn expression<'a>(p: &mut Parser<'a>) -> IRError {
    let token = p.current_token();
    let rule = rules::PARSER_RULE_TABLE
        .get(&token.type_)
        .expect(format!("Could not find parse rule for current token: {:?}", token).as_str());
    let infix = rule.infix;
    if infix as usize == nil_func as usize {
        p.emit_notice(token.pos, NoticeLevel::Error, "Expected an infix function, but instead got nil_func".to_string());
        return Err(());
    }
    infix(p).unwrap();
    Ok(())
}

pub(crate) fn literal<'a>(p: &mut Parser<'a>) -> IRError {
    // p.advance();
    let current_token = p.current_token();
    let pos = current_token.pos;
    let token_type = current_token.type_;
    let token_data = current_token.data.clone();
    match token_type {
        TokenType::Number => match current_token.data {
            TokenData::Integer(int) => {
                p.emit_ir(
                    pos,
                    TypeSignature::Primitive(PrimitiveType::Integer),
                    Instruction::Integer(int as i32),
                );
            }
            _ => {
                p.emit_notice(pos, NoticeLevel::Error, format!(
                    "Expected integer token data, but instead found {:?}",
                    current_token.data
                ));
                return Err(())
            }
        },
        TokenType::String => match token_data {
            TokenData::Str(s) => p.emit_ir(
                pos,
                TypeSignature::Primitive(PrimitiveType::String),
                Instruction::String(s.to_string().clone()),
            ),
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
    Ok(())
}

fn type_<'a>(p: &mut Parser<'a>) -> Result<TypeSignature, ()> {
    let current_token = &p.current_token();
    let ret = match (&current_token.type_, &current_token.data) {
        (TokenType::Identifier, TokenData::Str(s)) => {
            Ok(TypeSignature::Primitive(PrimitiveType::new(s)))
        }
        _ => Err(()),
    };
    if ret.is_ok() {
        p.advance().unwrap();
        ret
    } else {
        Err(())
    }
}
