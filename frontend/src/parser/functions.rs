use crate::{
    parser::{functions, rules, ParseContext},
    tokens::{LexerToken, TokenData, TokenType},
    Parser,
};

use ir::{
    hir::Instruction,
    type_signature::{PrimitiveType, TypeSignature},
};

use core::pos::BiPos as Position;
use notices::NoticeLevel;

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
            return Err(());
        }
        p.advance().unwrap();
    }
    p.emit_ir(Position::default(), TypeSignature::None, Instruction::EndModule);
    Ok(())
}

pub(crate) fn declaration_or_statement<'a>(p: &mut Parser<'a>) -> IRError {
    match p.current_token().type_ {
        TokenType::KwMod => mod_declaration(p),
        _ => statement(p),
    }
}

pub(crate) fn mod_declaration<'a>(p: &mut Parser<'a>) -> IRError {
    Ok(())
}

pub(crate) fn statement<'a>(p: &mut Parser<'a>) -> IRError {
    let token = p.current_token();
    match token.type_ {
        TokenType::KwVal => property(p)?,
        TokenType::KwVar => property(p)?,
        TokenType::KwFun => function(p)?,
        TokenType::KwLet => function(p)?,
        _ => {
            p.emit_notice(
                token.pos,
                NoticeLevel::Error,
                format!("Unexpected token found: {:?}", token).to_string(),
            );
            return Err(());
        }
    }
    Ok(())
}

pub(crate) fn property<'a>(p: &mut Parser<'a>) -> IRError {
    let lpos = p.current_token().pos;
    let mutable = if !p.check(TokenType::KwVal) {
        if !p.check(TokenType::KwVar){
            let message = format!(
                "Expected a val or var keyword token, but instead got {}",
                p.current_token()
            );
            p.emit_notice(lpos, NoticeLevel::Error, message);
            return Err(());
        }
        true
    }else{
        false
    };
    p.advance().unwrap();
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
            p.emit_notice(
                lpos,
                NoticeLevel::Error,
                "Failed to extract string data from identifier token.".to_string(),
            );
            return Err(());
        }
    };
    let signature = if p.consume(TokenType::Colon).is_ok() {
        if let Ok(t) = type_(p){
            t
        }else{
            p.emit_notice(p.prev_token().pos, NoticeLevel::Error, "Could not create type signature for property.".to_string());
            return Err(())
        }
    } else {
        p.advance()
            .expect("Failed to advance parser to next token.");
        TypeSignature::Untyped
    };

    if !p.check_consume(TokenType::Equal) {
        p.emit_notice(
            lpos,
            NoticeLevel::Error,
            "Value property must be initialized.".to_string(),
        );
        let found_token = p.current_token();
        let data = match &found_token.data {
            TokenData::Float(f) => f.to_string(),
            TokenData::Integer(i) => i.to_string(),
            TokenData::Str(s) => (*s).to_string(),
            TokenData::String(s) => s.clone(),
            _ => "Unknown".to_string(),
        };
        p.emit_notice(
            found_token.pos,
            NoticeLevel::Error,
            format!("Expected '=' but instead got {:?}", data),
        );
        return Err(());
    }
    p.emit_ir(lpos, signature, Instruction::Property(name, mutable));
    expression(p).expect("Could not parse expression.");
    Ok(())
}

pub(crate) fn function<'a>(p: &mut Parser<'a>) -> IRError {
    let lpos = p.current_token().pos;
    if !p.check_consume(TokenType::KwFun) {
        let message = format!(
            "Expected a fun keyword token, but instead got {}",
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
            p.emit_notice(
                lpos,
                NoticeLevel::Error,
                "Failed to extract string data from identifier token.".to_string(),
            );
            return Err(());
        }
    };
    if p.advance().is_err(){
        return Err(())
    }
    let mut params = Vec::<TypeSignature>::new();
    let mut param_ir = Vec::<ir::hir::ChannelIr>::new();
    if p.check(TokenType::LParen){
        
        loop{
            if p.check(TokenType::RParen){
                break;
            }
            let loc = p.next_token().pos;
            let param_name = match p.consume(TokenType::Identifier).unwrap() {
                TokenData::Str(s) => (*s).to_string(),
                _ => {
                    p.emit_notice(
                        lpos,
                        NoticeLevel::Error,
                        "Failed to extract string data from identifier token.".to_string(),
                    );
                    return Err(());
                }
            };
            let _ = p.consume(TokenType::Colon);
            let param_typename = match p.consume(TokenType::Identifier).unwrap() {
                TokenData::Str(s) => (*s).to_string(),
                _ => {
                    p.emit_notice(
                        lpos,
                        NoticeLevel::Error,
                        "Failed to extract string data from identifier token.".to_string(),
                    );
                    return Err(());
                }
            };
            let type_sig = ir::type_signature::TypeSignature::Primitive(PrimitiveType::new(param_typename.as_str()));
            params.push(type_sig.clone());
            param_ir.push(ir::hir::ChannelIr{
                pos: loc,
                sig: type_sig,
                ins: Instruction::FnParam(param_name)
            });
            p.advance().unwrap();
        }
    }
    let typename = if p.next_token().type_ == TokenType::Colon{
        match p.consume(TokenType::Identifier).unwrap() {
            TokenData::Str(s) => (*s).to_string(),
            _ => {
                p.emit_notice(
                    lpos,
                    NoticeLevel::Error,
                    "Failed to extract string data from identifier token.".to_string(),
                );
                return Err(());
            }
        }
    }else{
        "Unit".to_string()
    };
    let function_sig = ir::type_signature::TypeSignature::Function(ir::type_signature::FunctionSignature{
        parameters: params,
        return_type_signature: Box::new(TypeSignature::Primitive(PrimitiveType::new(typename.as_str())))
    });
    p.emit_ir(lpos, function_sig, ir::hir::Instruction::Fn(name));
    for ir in param_ir{
        p.emit_ir(ir.pos, ir.sig, ir.ins);
    }
    if p.consume(TokenType::LCurly).is_err(){
        return Err(())
    }

    p.context = ParseContext::Local;
    
    while !p.check_consume(TokenType::RCurly){
        if p.advance().is_err(){
            return Err(())
        }
        if local_statements(p).is_err(){
            return Err(())
        }
    }
    p.emit_ir(lpos, TypeSignature::None, ir::hir::Instruction::EndFn);

    Ok(())
}

pub(crate) fn local_statements<'a>(p: &mut Parser<'a>) -> IRError{
    match p.current_token().type_{
        TokenType::KwLet => local_var(p),
        TokenType::Identifier => {
            match p.next_token().type_{
                TokenType::Equal => unimplemented!(),
                _ => unimplemented!()
            }
        },
        _ => statement(p)
    }
}

pub(crate) fn local_var<'a>(p: &mut Parser<'a>) -> IRError {
    if p.context != ParseContext::Local{
        p.emit_notice(p.current_token().pos, NoticeLevel::Error, "Found 'let' outside of local context.".to_string());
        return Err(())
    }
    if !p.check_consume(TokenType::KwLet) {
        p.emit_notice(
            p.current_token().pos,
            NoticeLevel::Error,
            "Expected keyword 'let' for defining an local variable.".to_string(),
        );
    }
    let pos = p.current_token().pos;
    if !p.check(TokenType::Identifier) {
        let message = format!(
            "Expected an identifier token, but instead got {}",
            p.current_token()
        );
        p.emit_notice(pos, NoticeLevel::Error, message);
        return Err(());
    }
    let name = match p.current_token().data {
        TokenData::Str(s) => (*s).to_string(),
        _ => {
            p.emit_notice(
                pos,
                NoticeLevel::Error,
                "Failed to extract string data from identifier token.".to_string(),
            );
            return Err(());
        }
    };
    let signature = if p.consume(TokenType::Colon).is_ok() {
        if let Ok(t) = type_(p){
            t
        }else{
            p.emit_notice(p.current_token().pos, NoticeLevel::Error, "Could not create type signature for local variable.".to_string());
            return Err(())
        }
    } else {
        p.advance()
            .expect("Failed to advance parser to next token.");
        TypeSignature::Untyped
    };
    p.emit_ir(pos, signature, Instruction::LocalVar(name.clone(), false));

    if !p.check_consume(TokenType::Equal) {
        p.emit_notice(
            pos,
            NoticeLevel::Error,
            "Local property must be initialized.".to_string(),
        );
        let found_token = p.current_token();
        let data = match &found_token.data {
            TokenData::Float(f) => f.to_string(),
            TokenData::Integer(i) => i.to_string(),
            TokenData::Str(s) => (*s).to_string(),
            TokenData::String(s) => s.clone(),
            _ => "Unknown".to_string(),
        };
        p.emit_notice(
            found_token.pos,
            NoticeLevel::Error,
            format!("Expected '=' but instead got {:?}", data),
        );
        return Err(());
    }

    if expression(p).is_err() {
        p.emit_notice(
            pos,
            NoticeLevel::Error,
            format!("Local variable {} cannot go uninitialized.", name),
        );
    }
    if p.advance().is_err(){
        return Err(())
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
        p.emit_notice(
            token.pos,
            NoticeLevel::Error,
            "Expected an infix function, but instead got nil_func".to_string(),
        );
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
                p.emit_notice(
                    pos,
                    NoticeLevel::Error,
                    format!(
                        "Expected integer token data, but instead found {:?}",
                        current_token.data
                    ),
                );
                return Err(());
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
    if p.advance().is_err(){
        p.emit_notice(p.current_token().pos, NoticeLevel::Error, "Could not advance parser.".to_string());
        return Err(())
    }
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
