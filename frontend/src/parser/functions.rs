use crate::{
    parser::{ParseContext},
    tokens::{TokenData, TokenType},
    Parser,
};

use core::pos::BiPos;

use ir::{
    hir::{
        HIRInstruction
    },
    Chunk,
};

use ir_traits::{
    WriteInstruction
};

use notices::NoticeLevel;

type ParseResult = Result<(), ()>;
type ChunkResult = Result<Chunk, ()>;

pub fn module(p: &mut Parser) -> ParseResult {
    while !p.check(TokenType::Eof) {
        if let Err(()) = declaration_or_statement(p) {
            return Err(());
        }
        // p.advance().unwrap();
    }
    Ok(())
}

pub(crate) fn declaration_or_statement(p: &mut Parser) -> ParseResult {
    match p.current_token().type_ {
        TokenType::KwMod => mod_declaration(p),
        _ => statement(p),
    }
}

pub(crate) fn mod_declaration(_: &mut Parser) -> ParseResult {
    Ok(())
}

pub(crate) fn statement(p: &mut Parser) -> ParseResult {
    let token = p.current_token();
    match token.type_ {
        TokenType::KwVal => property(p)?,
        TokenType::KwVar => property(p)?,
        TokenType::KwFun => function(p)?,
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

pub(crate) fn property(p: &mut Parser) -> ParseResult {
    let mut chunk = Chunk::new();
    chunk.write_instruction(HIRInstruction::Property);
    let lpos = p.current_token().pos;
    chunk.write_pos(lpos);
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
    chunk.write_bool(mutable);
    p.advance().unwrap();
    if !p.check(TokenType::Identifier) {
        let message = format!(
            "Expected an identifier token, but instead got {}",
            p.current_token()
        );
        p.emit_notice(lpos, NoticeLevel::Error, message);
        return Err(());
    }
    let name = match &p.current_token().data {
        TokenData::String(s) => (*s).to_string(),
        _ => {
            p.emit_notice(
                lpos,
                NoticeLevel::Error,
                "Failed to extract string data from identifier token.".to_string(),
            );
            return Err(());
        }
    };

    chunk.write_string(name.clone());
    if p.check_consume_next(TokenType::Colon) {
        if let Ok(t) = type_(p){
            chunk.write_chunk(t);
        }else{
            p.emit_notice(p.prev_token().pos, NoticeLevel::Error, "Could not create type signature for property.".to_string());
            return Err(())
        }
        p.advance()
            .expect("Failed to advance parser to next token.");
    } else {
        p.advance()
            .expect("Failed to advance parser to next token.");
        chunk.write_instruction(HIRInstruction::Unknown);
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

    p.emit_ir_whole(chunk);
    
    expression(p).expect("Could not parse expression.");
    Ok(())
}

pub(crate) fn function(p: &mut Parser) -> ParseResult {
    let mut chunk = Chunk::new();
    let lpos = p.current_token().pos;
    if !p.check_consume(TokenType::KwFun) {
        let message = format!(
            "Expected a fun keyword token, but instead got {}",
            p.current_token()
        );
        p.emit_notice(lpos, NoticeLevel::Error, message);
        return Err(());
    }
    chunk.write_instruction(HIRInstruction::Fn);
    chunk.write_pos(lpos.clone());
    if !p.check(TokenType::Identifier) {
        let message = format!(
            "Expected an identifier token, but instead got {}",
            p.current_token()
        );
        p.emit_notice(lpos, NoticeLevel::Error, message);
        return Err(());
    }
    let name = match &p.current_token().data {
        TokenData::String(s) => (*s).to_string(),
        _ => {
            p.emit_notice(
                lpos,
                NoticeLevel::Error,
                "Failed to extract string data from identifier token.".to_string(),
            );
            return Err(());
        }
    };
    chunk.write_string(name);
    if p.advance().is_err(){
        return Err(())
    }
    if p.check(TokenType::LParen){
        loop{
            if p.check(TokenType::RParen){
                chunk.write_instruction(HIRInstruction::EndParams);
                break;
            }
            let mut param_chunk = Chunk::new();
            param_chunk.write_instruction(HIRInstruction::FnParam);
            let loc = p.next_token().pos;
            let param_name = match p.consume(TokenType::Identifier) {
                Ok(TokenData::String(s)) => (*s).to_string(),
                Ok(_) => {
                    p.emit_notice(
                        lpos,
                        NoticeLevel::Error,
                        "Failed to extract string data from identifier token.".to_string(),
                    );
                    return Err(());
                },
                Err(()) => return Err(())
            };
            param_chunk.write_pos(loc);
            param_chunk.write_string(param_name);
            let _ = p.consume(TokenType::Colon);
            let param_type = type_(p).unwrap();
            param_chunk.write_chunk(param_type);
            chunk.write_chunk(param_chunk);
            p.advance().unwrap();
        }
    }

    if p.check_consume_next(TokenType::Colon){
        let retype_chunk = type_(p)?;
        chunk.write_chunk(retype_chunk);
    }else{
        chunk.write_pos(p.current_token().pos);
        chunk.write_instruction(HIRInstruction::Unit);
    }

    p.emit_ir_whole(chunk);

    if p.consume(TokenType::LCurly).is_err(){
        return Err(())
    }

    p.context = ParseContext::Local;
    
    let mut body_chunk = Chunk::new();
    body_chunk.write_instruction(HIRInstruction::Block);
    body_chunk.write_pos(p.current_token().pos);
    p.emit_ir_whole(body_chunk);
    while !p.check_consume(TokenType::RCurly){
        if local_statements(p).is_err(){
            return Err(())
        }
    }
    let mut end_chunk = Chunk::new();
    end_chunk.write_instruction(HIRInstruction::EndBlock);
    end_chunk.write_pos(p.prev_token().pos);
    end_chunk.write_instruction(HIRInstruction::EndFn);

    p.emit_ir_whole(end_chunk);
    Ok(())
}

pub(crate) fn local_statements(p: &mut Parser) -> ParseResult{
    p.advance().unwrap();
    match p.current_token().type_{
        TokenType::RCurly => return Ok(()),
        TokenType::KwLet => local_var(p)?,
        TokenType::Identifier => {
            match p.next_token().type_{
                TokenType::Equal => unimplemented!(),
                _ => unimplemented!()
            };
        },
        _ => statement(p)?
    };
    // p.advance().unwrap();
    Ok(())
}

pub(crate) fn local_var(p: &mut Parser) -> ParseResult {
    let mut chunk = Chunk::new();
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
    chunk.write_instruction(HIRInstruction::LocalVar);
    let pos = p.current_token().pos;
    chunk.write_pos(pos);
    if p.check_consume_next(TokenType::KwMut){
        chunk.write_bool(true);
        chunk.write_pos(p.current_token().pos);
    }else{
        chunk.write_bool(false);
        chunk.write_pos(BiPos::default());
    }
    if !p.check(TokenType::Identifier) {
        let message = format!(
            "Expected an identifier token, but instead got {}",
            p.current_token()
        );
        p.emit_notice(pos, NoticeLevel::Error, message);
        return Err(());
    }
    let name = match &p.current_token().data {
        TokenData::String(s) => (*s).to_string(),
        _ => {
            p.emit_notice(
                pos,
                NoticeLevel::Error,
                "Failed to extract string data from identifier token.".to_string(),
            );
            return Err(());
        }
    };
    chunk.write_string(name.clone());
    chunk.write_pos(p.current_token().pos);
    if p.next_token().type_ == TokenType::Colon {
        if let Ok(t) = type_(p){
            chunk.write_chunk(t)
        }else{
            p.emit_notice(p.current_token().pos, NoticeLevel::Error, "Could not create type signature for local variable.".to_string());
            return Err(())
        }
    } else {
        p.advance()
            .expect("Failed to advance parser to next token.");
        chunk.write_pos(p.current_token().pos);
        chunk.write_instruction(HIRInstruction::Unknown);
    }
    p.emit_ir_whole(chunk);

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
            format!("Local variable {} cannot go uninitialized.", name.clone()),
        );
    }
    Ok(())
}

fn expression(p: &mut Parser) -> ParseResult {
    let token = p.current_token();
    let mut chunk = Chunk::new();
    match &token.data{
        TokenData::Float(f) => {
            chunk.write_instruction(HIRInstruction::Float);
            chunk.write_pos(token.pos);
            chunk.write_float(*f);
        }
        TokenData::Integer(i) => {
            chunk.write_instruction(HIRInstruction::Integer);
            chunk.write_pos(token.pos);
            chunk.write_int(*i);
        }
        TokenData::String(s) => {
            chunk.write_instruction(HIRInstruction::String);
            chunk.write_pos(token.pos);
            chunk.write_string(s.clone());
        }
        TokenData::None => {
            chunk.write_instruction(HIRInstruction::None);
            chunk.write_pos(token.pos);
        }
    }
    p.emit_ir_whole(chunk);
    p.advance().unwrap();
    Ok(())
}

#[allow(dead_code)]
pub(crate) fn literal(p: &mut Parser) -> ParseResult {
    // p.advance();
    let mut chunk = Chunk::new();
    let current_token = p.current_token();
    let pos = current_token.pos;
    let token_type = current_token.type_;
    let token_data = current_token.data.clone();
    match token_type {
        TokenType::Number => match current_token.data {
            TokenData::Integer(int) => {
                chunk.write_instruction(HIRInstruction::Integer);
                chunk.write_int(int as i32);
                chunk.write_pos(pos);
                p.emit_ir_whole(chunk);
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
            TokenData::String(s) => {
                 chunk.write_instruction(HIRInstruction::String);
                 chunk.write_string(s);
                 chunk.write_pos(pos);
                 p.emit_ir_whole(chunk);
            },
            _ => unimplemented!(),
        },
        _ => unimplemented!(),
    }
    Ok(())
}

fn type_(p: &mut Parser) -> ChunkResult {
    let mut chunk = Chunk::new();
    if p.advance().is_err(){
        p.emit_notice(p.current_token().pos, NoticeLevel::Error, "Could not advance parser.".to_string());
        return Err(())
    }
    let current_token = p.current_token();
    let ret = match (&current_token.type_, &current_token.data) {
        (TokenType::Identifier, TokenData::String(s)) => {
            chunk.write_pos(current_token.pos);
            let str = s.as_str();
            match str{
                "Int" => chunk.write_instruction(HIRInstruction::Integer),
                "Float" => chunk.write_instruction(HIRInstruction::Float),
                "String" => chunk.write_instruction(HIRInstruction::String),
                "Bool" => chunk.write_instruction(HIRInstruction::Bool),
                _ => {
                    chunk.write_instruction(HIRInstruction::Custom);
                    chunk.write_string(s.clone());
                }
            }
            Ok(chunk)
        }
        _ => Err(()),
    };
    if ret.is_ok() {
        ret
    } else {
        Err(())
    }
}
