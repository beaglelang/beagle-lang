use core::pos::BiPos;
use crate::type_signature::{
    TypeSignature
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
    ///Heap allocation of `size`
    HeapAlloc(usize),
    ///Stack allocation of `size`.
    ///Either an object contruction or a lateinit instruction must proceed this.
    StackAlloc(usize),
    ///Uninitialized/late initializer.
    ///This is used for preallocating something without an immediate value. 
    ///`None` is the placehold value, so instead of an unsafe empty place in memory, None will fill the emptyness.
    ///`None` is an object that can be stretched to fit any place whatsoever, and will simply just be garbage data.
    ///The syntax for this is:
    ///     let something: A = None
    Lateinit(usize),
    ///Mutate object `name`.
    ///An expression must proceed this instruction.
    ObjMut(String),
    ///Halt compiler
    Halt
}

pub struct MIR{
    pos: BiPos,
    sig: TypeSignature,
    ins: MIRInstruction
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

use std::fmt::{
    Display,
    Formatter,
    Result,
};
use core::ansi;
use super::fmt_tab;

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use MIRInstruction::*;
        let mut depth = 0;
        for (ins, sig) in self.instructions.iter().zip(self.signatures.iter()){
            match ins{
                Halt => {
                    fmt_tab(f, depth)?;
                    writeln!(f, "{}HALT{}", ansi::Fg::BrightRed, ansi::Fg::Reset)?;
                },
                Module(mname) => {
                    fmt_tab(f, depth)?;
                    depth += 1;
                    writeln!(
                        f,
                        "{}Module{} {}{}",
                        ansi::Fg::Blue,
                        ansi::Fg::Yellow,
                        mname,
                        ansi::Fg::Reset
                    )?;
                },
                EndModule => {
                    depth -= 1;
                    writeln!(
                        f,
                        "{}EndMod{}",
                        ansi::Fg::Blue,
                        ansi::Fg::Reset
                    )?;
                }
                Fun(name) => {
                    fmt_tab(f, depth)?;
                    depth += 1;
                    writeln!(
                        f,
                        "{}Function{} {}{}{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::Red,
                        name,
                        sig,
                        ansi::Fg::Reset
                    )?;
                },
                EndFn => {
                    depth -= 1;
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}EndFun{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::Reset
                    )?;
                },
                FunParam(name) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Parameter{} {}: {}{}{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Yellow,
                        sig,
                        ansi::Fg::Reset
                    )?;
                },
                ObjInit(name, mutable) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Init{} {} {}{}{}: {}{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::Green,
                        if *mutable {
                            "variable"
                        }else{
                            "value"
                        },
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Blue,
                        sig,
                        ansi::Fg::Reset
                    )?;
                }
                Integer(i) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Int {}{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::Green,
                        i
                    )?;
                },
                Float(i) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Float {}{}{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::Green,
                        i,
                        ansi::Fg::Reset
                    )?;
                },
                String(i) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}String {}\"{}\"{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::Green,
                        i,
                        ansi::Fg::Reset
                    )?;
                },
                Unit => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Unit{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::Reset,
                    )?;
                },
                Drop(name) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Drop {}{}{}",
                        ansi::Fg::Magenta,
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Reset
                    )?;
                },
                Ref(name) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Ref {}{}{}",
                        ansi::Fg::BrightRed,
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Reset
                    )?;
                },
                Move(name) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Mov {}{}{}",
                        ansi::Fg::BrightRed,
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Reset
                    )?;
                },
                Copy(name) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Copy {}{}{}",
                        ansi::Fg::BrightRed,
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Reset
                    )?;
                },
                HeapAlloc(size) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}HeapAlloc({}{}){}",
                        ansi::Fg::Blue,
                        ansi::Fg::White,
                        size,
                        ansi::Fg::Reset
                    )?;
                },
                StackAlloc(size) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}StackAlloc({}{}){}",
                        ansi::Fg::Blue,
                        ansi::Fg::White,
                        size,
                        ansi::Fg::Reset
                    )?;
                },
            }
        }
        Ok(())
    }
}