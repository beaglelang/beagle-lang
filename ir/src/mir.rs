use core::pos::BiPos;
use crate::{
    type_signature::{
        TypeSignature
    },
    hir::{
        HIRInstruction
    }
};

use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MIRInstruction{
    ///Module start
    Module(String),
    ///End module
    EndModule,

    ///Initialize object `name` with `mutability`.
    ///An allocation instruction must precede this with the size of the object.
    ///Following this will be a call to the initializer.
    ObjInit(String, bool),
    ///Drop `name`. This can either be a value or a reference.
    ///The drop mechanism is smart. If what is being dropped is a reference, 
    ///the reference counter will decrement the count for object `name`.
    Drop(String),
    ///Function start. 
    ///At the beginning is where local variable preallocation will occur.
    Fun(String),
    ///End function.
    ///This is where all drops to local variables and any references or values passed as arguments will occur.
    EndFun,

    //Function param.
    //The call to the containing function will handle the pass-by.
    FunParam(String),

    //Literals
    ///Integer literal
    Integer(i32),
    ///Float literal
    Float(f32),
    ///String literal
    String(String),
    ///Boolean literal
    Bool(bool),
    ///Unit type
    Unit,

    //Memory management instructions
    ///Create reference for `refee`.
    ///High level references to properties will result in this instruction.
    Ref(String),
    ///Move `name`.
    ///A single reference to a local variables will result in this instruction.
    Move(String),
    ///Copy `name`.
    ///Where n is the number of references to a local variable, all references until n-1 will result in this instruction,
    /// whereas the final reference to a local variable will result in a Move instruction.
    Copy(String),
    ///Heap allocation of `size` for object `name`
    HeapAlloc(String, usize),
    ///Stack allocation of `size` for object `name`.
    ///Either an object contruction or a lateinit instruction must proceed this.
    StackAlloc(String, usize),
    ///Uninitialized/late initializer.
    ///This is used for leaving an resource empty until further notice.
    ///For immutable objects, this grants one free initial mutation for initialization, to which all subsequent mutations will become invalid. 
    ///`None` is the placehold value, so instead of an unsafe empty place in memory, None will fill the emptyness.
    ///`None` is an object that can be stretched to fit any place whatsoever, and will simply just be garbage data.
    ///The syntax for this is:
    ///     let something: A = None
    Lateinit,
    ///Mutate object `name`.
    ///An expression must proceed this instruction.
    ObjMut(String),
    ///Halt compiler
    Halt
}

impl MIRInstruction{
    pub fn from_hir(ins: HIRInstruction) -> MIRInstruction{
        match ins{
            HIRInstruction::Module(m) => MIRInstruction::Module(m),
            HIRInstruction::EndModule => MIRInstruction::EndModule,
            HIRInstruction::Fn(name) => MIRInstruction::Fun(name),
            HIRInstruction::EndFn => MIRInstruction::EndFun,
            HIRInstruction::FnParam(m) => MIRInstruction::FunParam(m),
            HIRInstruction::LocalVar(name, mutable) => MIRInstruction::ObjInit(name, mutable),
            HIRInstruction::Property(name, mutable) => MIRInstruction::ObjInit(name, mutable),
            HIRInstruction::Integer(i) => MIRInstruction::Integer(i),
            HIRInstruction::Float(f) => MIRInstruction::Float(f),
            HIRInstruction::String(s) => MIRInstruction::String(s),
            HIRInstruction::Bool(b) => MIRInstruction::Bool(b),
            HIRInstruction::Halt => MIRInstruction::Halt,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MIR{
    pub pos: BiPos,
    pub sig: TypeSignature,
    pub ins: MIRInstruction
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// The name of the module
    pub name: String,
    /// The ir instructions
    pub instructions: Vec<MIRInstruction>,
    /// The signatures of each ir instruction
    pub signatures: Vec<TypeSignature>,
    /// The positions in code of each ir instruction
    pub positions: Vec<BiPos>,
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

    pub fn push_ir(&mut self, ir: MIR) {
        self.push(ir.pos, ir.sig, ir.ins)
    }

    /// Push an instruction into the module
    pub fn push(&mut self, pos: BiPos, sig: TypeSignature, ins: MIRInstruction) {
        self.positions.push(pos);
        self.signatures.push(sig);
        self.instructions.push(ins);
    }
}