use super::{
    Load,
    ident::Identifier,
    MemmyGenerator
};

use core::pos::BiPos;

use ir::{
    Chunk,
    hir::HIRInstruction,
};
use ir_traits::ReadInstruction;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum Value{
    Int(i32),
    Float(f32),
    String(String),
    Bool(bool),
    Custom(Identifier),
}

#[derive(Debug, Clone)]
#[allow(dead_code)]

pub enum ExpressionKind{
    Value(Value),
    Group(Expression),
    Binary(OpKind, Expression, Expression),
}

#[derive(Debug, Clone)]
pub enum OpKind{
    Plus,
    Minus,
    Mult,
    Div
}

#[derive(Debug, Clone)]
pub struct Expression{
    kind: Box<ExpressionKind>,
    pos: BiPos
}

impl Load for Expression{
    type Output = Expression;

    fn load(chunk: &Chunk, memmy: &MemmyGenerator) -> Result<Self::Output, ()> {
        let pos = match chunk.read_pos(){
            Ok(pos) => pos,
            Err(msg) => {
                memmy.emit_error(msg, BiPos::default())?;
                return Err(())
            }
        };

        let opcode = &chunk.read_instruction();
        match &opcode{
            Some(HIRInstruction::Integer) => {
                let value = chunk.read_int();
                return Ok(Expression{
                    kind: Box::new(ExpressionKind::Value(Value::Int(value))),
                    pos,
                })
            }
            Some(HIRInstruction::Float) => {
                let value = chunk.read_float();
                return Ok(Expression{
                    kind: Box::new(ExpressionKind::Value(Value::Float(value))),
                    pos,
                })

            }
            Some(HIRInstruction::String) => {
                let value = chunk.read_string();
                return Ok(Expression{
                    kind: Box::new(ExpressionKind::Value(Value::String(value.to_owned()))),
                    pos,
                })

            }
            Some(HIRInstruction::Bool) => {
                let value = chunk.read_bool();
                return Ok(Expression{
                    kind: Box::new(ExpressionKind::Value(Value::Bool(value))),
                    pos,
                })
            }
            Some(HIRInstruction::Add) => {
                let left = match Expression::load(chunk, memmy){
                    Ok(left) => left,
                    Err(()) => return Err(())
                };
                let right = match Expression::load(chunk, memmy){
                    Ok(right) => right,
                    Err(()) => return Err(())
                };
                return Ok(Expression{
                    kind: Box::new(ExpressionKind::Binary(OpKind::Plus, left, right)),
                    pos,
                })
            }
            Some(HIRInstruction::Sub) => {
                let left = match Expression::load(chunk, memmy){
                    Ok(left) => left,
                    Err(()) => return Err(())
                };
                let right = match Expression::load(chunk, memmy){
                    Ok(right) => right,
                    Err(()) => return Err(())
                };
                return Ok(Expression{
                    kind: Box::new(ExpressionKind::Binary(OpKind::Minus, left, right)),
                    pos,
                })
            }
            Some(HIRInstruction::Mult) => {
                let left = match Expression::load(chunk, memmy){
                    Ok(left) => left,
                    Err(()) => return Err(())
                };
                let right = match Expression::load(chunk, memmy){
                    Ok(right) => right,
                    Err(()) => return Err(())
                };
                return Ok(Expression{
                    kind: Box::new(ExpressionKind::Binary(OpKind::Mult, left, right)),
                    pos,
                })
            }
            Some(HIRInstruction::Div) => {
                let left = match Expression::load(chunk, memmy){
                    Ok(left) => left,
                    Err(()) => return Err(())
                };
                let right = match Expression::load(chunk, memmy){
                    Ok(right) => right,
                    Err(()) => return Err(())
                };
                return Ok(Expression{
                    kind: Box::new(ExpressionKind::Binary(OpKind::Div, left, right)),
                    pos,
                })
            }
            _ => {
                memmy.emit_error(format!("This feature is not yet implemented: {:?}", opcode.clone().unwrap()), BiPos::default())?;
                return Err(())
            }
        }
    }
}