use crate::{
    parser::{
        rules,
        functions
    },
    Parser,
    tokens::{
        TokenType,
        TokenData,
        LexerToken
    }
};

use ir::{
    type_signature::{
        TypeSignature,
        PrimitiveType,
    },
    Position,
    hir::{
        Instruction
    }
};

type IRError = Result<(),String>;

pub(crate) fn nil_func<'a>(p: &mut Parser<'a>) -> IRError{
    panic!("This is a placehold func for parse rules. This represents a null pointer function. This should not have been called.")
}

pub(crate) fn module<'a>(p: &mut Parser<'a>) -> IRError{
    p.emit_ir(Position::default(), TypeSignature::None, Instruction::Module(p.name.clone()));
    while !p.check(TokenType::Eof){
        if let Err(m) = declaration_or_statement(p){
            return Err(format!("Failed to convert token to IR: {}",m))
        }
        p.advance().unwrap();
    };
    Ok(())
}

pub(crate) fn declaration_or_statement<'a>(p: &mut Parser<'a>) -> IRError{
    match p.current_token().type_{
        TokenType::KwMod => declaration(p),
        _ => statement(p),
    }
}

pub(crate) fn declaration<'a>(p: &mut Parser<'a>) -> IRError{
    Ok(())
}

pub(crate) fn statement<'a>(p: &mut Parser<'a>) -> IRError{
    match p.current_token().type_{
        TokenType::KwVal => property_val(p)?,
        TokenType::KwVar => property_var(p)?,
        TokenType::KwLet => local_var(p)?,
        _ => return Err(format!("Incorrect statement: Not implemented: {:?}", p.current_token()))
    }
    Ok(())
}

pub(crate) fn property_val<'a>(p: &mut Parser<'a>) -> IRError{
    let lpos = p.current_token().pos;
    if p.consume(TokenType::KwVal, "Expected keyword 'val' for defining an immutable property.").is_err(){
        let message = format!("Expected a val keyword token, but instead got {}", p.current_token());
        return Err(message.to_string());
    }
    if !p.check(TokenType::Identifier){
        let message = format!("Expected an identifier token, but instead got {}", p.current_token());
        return Err(message.to_string());
    }
    let name = match p.current_token().data{
        TokenData::Str(s) => (*s).to_string(),
        _ => {
            panic!("Failed to extract string data from identifier token.")
        }
    };
    let (signature, is_untyped) = if p.check_consume(TokenType::Colon){
        (type_(p).expect("Could not create type signature for value property."), false)
    }else{
        p.advance().expect("Failed to advance parser to next token.");
        (TypeSignature::Untyped, false)
    };

    if !p.check_consume(TokenType::Equal){
        return Err("Value property must be initialized.".to_string());
    }
    let irpos = Position{
        start: (lpos.start.0 as u32, lpos.start.1 as u32),
        end: (lpos.end.0 as u32, lpos.start.1 as u32)
    };
    p.emit_ir(irpos, signature, Instruction::Property(name, false));
    expression(p).expect("Could not parse expression.");
    Ok(())
}

pub(crate) fn property_var<'a>(p: &mut Parser<'a>) -> IRError{
    let was_error = p.consume(TokenType::KwVar, "Expected keyword 'var' for defining an mutable property.").is_err();
    Ok(())
}

pub(crate) fn local_var<'a>(p: &mut Parser<'a>) -> IRError{
    let mut was_error = p.consume(TokenType::KwVal, "Expected keyword 'val' for defining an immutable property.").is_err();
    let name = match p.consume(TokenType::Identifier, "Expected an identifier for value property").unwrap(){
        TokenData::Str(s) => (*s).to_string(),
        _ => {
            panic!("Failed to extract string data from identifier token.")
        }
    };
    let (signature, is_untyped) = if p.check_consume(TokenType::Colon){
        (type_(p).expect("Could not create type signature for local variable."), false)
    }else{
        (TypeSignature::Untyped, false)
    };

    if p.check_consume(TokenType::Equal){
        // if expression(p).is_err(){
        //     was_error = true;
        // }
        if expression(p).is_err(){
            was_error = true;
        }
        true
    }else if is_untyped{
        was_error = true;
        return Err(format!("Local variable {} cannot go uninitialized.", name));
    }else{
        false
    };
    Ok(())
}

fn expression<'a>(p: &mut Parser<'a>) -> IRError{
    let token = p.current_token();
    let rule = rules::PARSER_RULE_TABLE
        .get(&token.type_).expect(format!("Could not find parse rule for current token: {:?}", token).as_str());
    let infix = rule.infix;
    if infix as usize == nil_func as usize {
        return Err("Expected an infix function, but instead got nil_func".to_string());
    }
    infix(p).unwrap();
    Ok(())
}

pub(crate) fn literal<'a>(p: &mut Parser<'a>) -> IRError{
    // p.advance();
    let current_token = p.current_token();
    let pos = Position{
        start: (current_token.pos.start.0 as u32, current_token.pos.start.1 as u32),
        end: (current_token.pos.end.0 as u32, current_token.pos.end.1 as u32),
    };
    let token_type = current_token.type_;
    let token_data = current_token.data.clone();
    match token_type{
        TokenType::Number => {
            match current_token.data{
                TokenData::Integer(int) => {
                    p.emit_ir(
                        pos, 
                        TypeSignature::Primitive(PrimitiveType::Integer), 
                        Instruction::Integer(int as i32)
                    );
                }
                _ => return Err(format!("Expected integer token data, but instead found {:?}", current_token.data))
            }
        }
        TokenType::String => {
            match token_data{
                TokenData::Str(s) => {
                    p.emit_ir(
                        pos,
                        TypeSignature::Primitive(PrimitiveType::String),
                        Instruction::String(s.to_string().clone())
                    )
                }
                _ => unimplemented!()
            }
        }
        _ => unimplemented!()
    }
    Ok(())
}

fn type_<'a>(p: &mut Parser<'a>) -> Result<TypeSignature, ()>{
    let current_token = &p.current_token();
    let ret = match (&current_token.type_, &current_token.data){
        (TokenType::Identifier, TokenData::Str(s)) => {
            Ok(TypeSignature::Primitive(PrimitiveType::new(s)))
        }
        _ => {
            Err(())
        }
    };
    if ret.is_ok(){
        p.advance().unwrap();
        ret
    }else{
        Err(())
    }
}