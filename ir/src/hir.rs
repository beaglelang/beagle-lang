use super::type_signature::TypeSignature;
use core::pos::BiPos as Position;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// The name of the module
    pub name: String,
    /// The ir instructions
    pub instructions: Vec<HIRInstruction>,
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

    pub fn push_ir(&mut self, ir: HIR) {
        self.push(ir.pos, ir.sig, ir.ins)
    }

    /// Push an instruction into the module
    pub fn push(&mut self, pos: Position, sig: TypeSignature, ins: HIRInstruction) {
        self.positions.push(pos);
        self.signatures.push(sig);
        self.instructions.push(ins);
    }
}

#[derive(Debug, Clone)]
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

use std::fmt::{
    Display,
    Formatter,
    Result
};

use core::ansi;
use super::{
    fmt_tab,
};

impl Display for Module {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        use HIRInstruction::*;
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
                Fn(name) => {
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
                }
                FnParam(name) => {
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
                Property(name, mutable) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Property{} {} {}{}{}: {}{}",
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
                LocalVar(name, mutable) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Local{} {} {}{}{}: {}{}",
                        ansi::Fg::Cyan,
                        ansi::Fg::Green,
                        if *mutable {
                            "value"
                        }else{
                            "variable"
                        },
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Blue,
                        sig,
                        ansi::Fg::Reset
                    )?;
                },
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
                }
            }
        }
        Ok(())
    }
}
