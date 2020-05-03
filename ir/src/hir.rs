use serde::{Deserialize, Serialize};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::Chunk;

use core::pos::BiPos;

use ir_traits::{
    Instruction,
    ReadInstruction,
    WriteInstruction,
};
use ir_derive::{
    Instruction,
    ReadInstruction,
    WriteInstruction,
};

#[derive(FromPrimitive, Instruction, WriteInstruction, ReadInstruction, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum HIRInstruction {
    //The start of module `name`.
    Module,
    ///The end of module `name`
    EndModule,
    //The start of a function. The name of the function is expected to follow.
    Fn,
    EndFn,
    //The start of a param. The name and type of the param must follow.
    FnParam,
    ///The end of the function params list
    EndParams,
    //A property which must be given a name and whether it is mutable or not. An expression must follow.
    Property,
    //A local variable which must be given a name and whether it is mutable or not. An expression must follow.
    LocalVar,

    ///The start of a new block
    Block,
    ///The end of the active block
    EndBlock,

    Add,
    Sub,
    Mult,
    Div,

    Integer,
    Float,
    Bool,
    String,
    None,
    
    Halt,
}

pub fn padding() -> String{
    String::from(" ").repeat(4)
}

impl std::fmt::Display for Chunk{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        loop{
            let ins = self.read_instruction();
            match &ins{
                Some(HIRInstruction::Module) => {
                    let pos = BiPos::default();
                    let name = self.read_string();
                    writeln!(f, "{}{}Module{}{}", pos, padding(), padding(), name)?;
                }
                Some(HIRInstruction::EndModule) => {
                    let pos = BiPos::default();
                    let name = self.read_string();
                    writeln!(f, "{}{}EndModule{}{}", pos, padding(), padding(), name)?;
                }
                Some(HIRInstruction::Fn) => {
                    let pos = self.read_pos();
                    let name = self.read_string();
                    writeln!(f, "{}{}Fn{}{}", pos, padding(), padding(), name)?;
                    loop{
                        let ins = self.read_instruction().unwrap();
                        match &ins{
                            HIRInstruction::FnParam => {
                                let pos = self.read_pos();
                                let name = self.read_string();
                                let typename = self.read_string();
                                writeln!(f, "{}{}FnParam{}{}: {}", pos, padding(), padding(), name, typename)?;
                            }
                            HIRInstruction::EndParams => {
                                break;
                            }
                            _ => write!(f, "Error: corrupt bytecode. Expected either FnParam or EndParams but instead got {:?}", ins)?,
                        }
                    }
                    let typename = self.read_string();
                    let typename_pos = self.read_pos();
                    writeln!(f, "{}{}FnType{}{}", typename_pos, padding(), padding(), typename)?;
                }
                Some(HIRInstruction::Block) => {
                    let pos = self.read_pos();
                    writeln!(f, "{}{}Block", pos, padding())?;
                }
                Some(HIRInstruction::EndBlock) => {
                    let pos = self.read_pos();
                    writeln!(f, "{}{}EndBlock", pos, padding())?;
                }
                Some(HIRInstruction::EndFn) => {
                    writeln!(f, "{}{}EndFn", BiPos::default(), padding())?;
                }
                Some(HIRInstruction::LocalVar) => {
                    let pos = self.read_pos();
                    let mutable = self.read_bool();
                    let _mut_pos = self.read_pos();
                    let name = self.read_string();
                    let _name_pos = self.read_pos();
                    let typename = self.read_string();
                    let _typename_pos = self.read_pos();
                    writeln!(f, "{}{}Local{}{}{}: {}", pos, padding(), if mutable{ "Var" }else{ "Val" }, padding(), name, typename)?;
                }
                Some(HIRInstruction::Property) => {
                    let pos = self.read_pos();
                    let mutable = self.read_bool();
                    let name = self.read_string();
                    let typename = self.read_string();
                    writeln!(f, "{}{}Property{}{}{}: {}", pos, padding(), if mutable { "Var" } else { "Val" }, padding(), name, typename)?;
                }
                Some(HIRInstruction::Integer) => {
                    let pos = self.read_pos();
                    let value = self.read_int();
                    writeln!(f, "{}{}Integer{}{}", pos, padding(), padding(), value)?;
                }
                Some(HIRInstruction::Float) => {
                    let pos = self.read_pos();
                    let value = self.read_float();
                    writeln!(f, "{}{}Float{}{}", pos, padding(), padding(), value)?;
                }
                Some(HIRInstruction::String) => {
                    let pos = self.read_pos();
                    let value = self.read_string();
                    writeln!(f, "{}{}String{}{}", pos, padding(), padding(), value)?;
                }
                Some(HIRInstruction::Bool) => {
                    let pos = self.read_pos();
                    let value = self.read_bool();
                    writeln!(f, "{}{}Bool{}{}", pos, padding(), padding(), value)?;
                }
                Some(HIRInstruction::Add) => {
                    let pos = self.read_pos();
                    writeln!(f, "{}{}Add", pos, padding())?;
                }
                Some(HIRInstruction::Sub) => {
                    let pos = self.read_pos();
                    writeln!(f, "{}{}Sub", pos, padding())?;
                }
                Some(HIRInstruction::Mult) => {
                    let pos = self.read_pos();
                    writeln!(f, "{}{}Mult", pos, padding())?;
                }
                Some(HIRInstruction::Div) => {
                    let pos = self.read_pos();
                    writeln!(f, "{}{}Div", pos, padding())?;
                }
                Some(HIRInstruction::None) => {
                    let pos = self.read_pos();
                    writeln!(f, "{}{}None", pos, padding())?;
                }
                Some(_) => {
                    let pos = self.read_pos();
                    writeln!(f, "{}{}Unknown", pos, padding())?;
                }
                None => break,
            }
            self.advance();
        };
        Ok(())
    }
}
