use serde::{Deserialize, Serialize};

use core::pos::BiPos as Position;

pub mod hir;
pub mod type_signature;
pub mod mir;

use mir::{
    MIR,
    MIRInstruction
};
use type_signature::TypeSignature;

pub const TAB_WIDTH: usize = 5;

use std::fmt::{
    Display,
    Formatter,
    Result
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// The name of the module
    pub name: String,
    /// The ir instructions
    pub instructions: Vec<MIRInstruction>,
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

    pub fn push_ir(&mut self, ir: MIR) {
        self.push(ir.pos, ir.sig, ir.ins)
    }

    /// Push an instruction into the module
    pub fn push(&mut self, pos: Position, sig: TypeSignature, ins: MIRInstruction) {
        self.positions.push(pos);
        self.signatures.push(sig);
        self.instructions.push(ins);
    }
}

fn repeat_char(c: char, times: usize) -> String{
    std::iter::repeat(c).take(times).collect::<String>()
}

fn fmt_tab(f: &mut Formatter<'_>, depth: usize) -> Result{
    if depth != 0{
        for _ in 0 .. depth{
            write!(f, "|{}", repeat_char(' ', TAB_WIDTH))?;
        }
        Ok(())
    }else{
        Ok(())
    }
}

use core::ansi;

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
                EndFun => {
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
                Bool(i) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Bool {}\"{}\"{}",
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
                HeapAlloc(name, size) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}{} = {}HeapAlloc({}{}{}){}",
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Blue,
                        ansi::Fg::White,
                        size,
                        ansi::Fg::Blue,
                        ansi::Fg::Reset
                    )?;
                },
                StackAlloc(name, size) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}{} = {}StackAlloc({}{}{}){}",
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Blue,
                        ansi::Fg::White,
                        size,
                        ansi::Fg::Blue,
                        ansi::Fg::Reset
                    )?;
                },
                Lateinit => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}Lateinit{}",
                        ansi::Fg::Red,
                        ansi::Fg::Reset
                    )?;
                },
                ObjMut(name) => {
                    fmt_tab(f, depth)?;
                    writeln!(
                        f,
                        "{}ObjMut({}{}{}){}",
                        ansi::Fg::Red,
                        ansi::Fg::White,
                        name,
                        ansi::Fg::Red,
                        ansi::Fg::Reset
                    )?;
                },
            }
        }
        Ok(())
    }
}