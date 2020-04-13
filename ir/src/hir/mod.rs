use super::type_signature::TypeSignature;
use core::pos::BiPos as Position;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// The name of the module
    pub name: String,
    /// The ir instructions
    pub instructions: Vec<Instruction>,
    /// The signatures of each ir instruction
    pub signatures: Vec<TypeSignature>,
    /// The positions in code of each ir instruction
    pub positions: Vec<Position>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Module {
            name,
            instructions: Vec::new(),
            signatures: Vec::new(),
            positions: Vec::new(),
        }
    }

    pub fn push_ir(&mut self, ir: ChannelIr) {
        self.push(ir.pos, ir.sig, ir.ins)
    }

    /// Push an instruction into the module
    pub fn push(&mut self, pos: Position, sig: TypeSignature, ins: Instruction) {
        self.positions.push(pos);
        self.signatures.push(sig);
        self.instructions.push(ins);
    }
}

#[derive(Debug, Clone)]
pub struct ChannelIr {
    pub pos: Position,
    pub sig: TypeSignature,
    pub ins: Instruction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Instruction {
    //The module being parsed, which needs a name.
    Module(String),
    //The start of a function. The name of the function is expected to follow.
    Fn(String),
    //The start of a param. The name and type of the param must follow.
    FnParam(String, String),
    //The return type of the function
    FnType(String),
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
