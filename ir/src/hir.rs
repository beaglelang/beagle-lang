use super::type_signature::TypeSignature;
use core::pos::BiPos as Position;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HIR {
    pub pos: Position,
    pub sig: TypeSignature,
    pub ins: HIRInstruction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HIRInstruction {
    //The module being parsed, which needs a name.
    Module(String),
    EndModule,
    //The start of a function. The name of the function is expected to follow.
    Fn(String),
    EndFn,
    //The start of a param. The name and type of the param must follow.
    FnParam(String),
    //A property which must be given a name and whether it is mutable or not. An expression must follow.
    Property(String, bool),
    //A local variable which must be given a name and whether it is mutable or not. An expression must follow.
    LocalVar(String, bool),

    Integer(i32),
    Float(f32),
    Bool(bool),
    String(String),
    Halt,
}


