use serde::{Deserialize, Serialize};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use super::Chunk;

use ansi_term::Colour;

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
    //No type was specified
    Unknown,
    //A non-primitive type like A, Person, or Device
    Custom,
    //aka 'void'
    Unit,
    None,
    
    Halt,
}

impl HIRInstruction{
    pub fn from_string(string: String) -> (Self, Option<String>){
        match string.clone().as_str(){
            "String" => (Self::String, None),
            "Int" => (Self::Integer, None),
            "Float" => (Self::Float, None),
            "Bool" => (Self::Bool, None),
            _ => (Self::Unknown, Some(string))
        }
    }
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
                    let name = self.read_string();
                    write!(f, "{}{}{}{}", padding(), Colour::Blue.paint("Module"), padding(), Colour::White.paint(name))?;
                }
                Some(HIRInstruction::EndModule) => {
                    let name = self.read_string();
                    write!(f, "{}{}{}{}", padding(), Colour::Blue.paint("EndModule"), padding(), Colour::White.paint(name))?;
                }
                Some(HIRInstruction::Fn) => {
                    let _pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let _name_pos = self.read_pos();
                    let name = self.read_string();
                    write!(f, "{}{}{}{}(", padding(), Colour::Blue.paint("Fn"), padding(), Colour::White.paint(name))?;
                    loop{
                        let ins = self.read_instruction().unwrap();
                        match &ins{
                            HIRInstruction::FnParam => {
                                let _pos = match self.read_pos(){
                                    Ok(pos) => pos,
                                    Err(msg) => {
                                        println!("{}", msg);
                                        return Err(std::fmt::Error{})
                                    }
                                };
                                let _name_pos = self.read_pos();
                                let name = self.read_string();
                                let _typename_pos = self.read_pos();
                                let typename = self.read_string();
                                writeln!(f, "{}: {}", Colour::Yellow.paint(format!("FnParam {}", name)), Colour::White.paint(typename))?;
                            }
                            HIRInstruction::EndParams => {
                                break;
                            }
                            _ => write!(f, "Error: corrupt bytecode. Expected either FnParam or EndParams but instead got {:?}", ins)?,
                        }
                    }
                    write!(f, ")")?;
                    let _typename_pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let typename = self.read_string();
                    writeln!(f, "{}", Colour::White.paint(typename))?;
                }
                Some(HIRInstruction::Block) => {
                    let _pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    writeln!(f, "{}", Colour::Black.paint("{"))?;
                }
                Some(HIRInstruction::EndBlock) => {
                    let _pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    writeln!(f, "{}", Colour::Black.paint("}"))?;
                }
                Some(HIRInstruction::EndFn) => {
                    writeln!(f, "{}", Colour::Blue.paint("EndFun"))?;
                }
                Some(HIRInstruction::LocalVar) => {
                    let _pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let mutable = self.read_bool();
                    let _mut_pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let name = self.read_string();
                    let _name_pos = self.read_pos();
                    let typename = self.read_string();
                    let _typename_pos = self.read_pos();
                    writeln!(f, "{} {}: {}", Colour::Purple.paint(if mutable{ "Local Var" }else{ "Local Val" }), Colour::White.paint(name), Colour::White.paint(typename))?;
                }
                Some(HIRInstruction::Property) => {
                    let _pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let _ident_pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let ident = self.read_string();
                    let mutable = self.read_bool();
                    let _mutable_pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
            
                    let _typename_pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    match self.read_instruction(){
                        Some(HIRInstruction::Integer) | Some(HIRInstruction::Float) | Some(HIRInstruction::String) | Some(HIRInstruction::Unit) | Some(HIRInstruction::Custom) => {},
                        Some(_) => {}
                        None =>{
                            println!("{}", format!("Attempted to read type information from typeck while loading property into memmy, found None."));
                            return Err(std::fmt::Error{})
                        }
                    };
                    let typename = self.read_string();
                    writeln!(f, "{} {}: {}", Colour::Purple.paint(if mutable { "Property Var" } else { "Property Val" }), Colour::White.paint(ident), Colour::White.paint(typename))?;
                }
                Some(HIRInstruction::Integer) => {
                    let _pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let value = self.read_int();
                    writeln!(f, "{} {}", Colour::Cyan.paint("Integer"), Colour::White.paint(value.to_string()))?;
                }
                Some(HIRInstruction::Float) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let value = self.read_float();
                    writeln!(f, "{}{}Float{}{}", pos, padding(), padding(), value)?;
                }
                Some(HIRInstruction::String) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let value = self.read_string();
                    writeln!(f, "{}{}String{}{}", pos, padding(), padding(), value)?;
                }
                Some(HIRInstruction::Bool) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    let value = self.read_bool();
                    writeln!(f, "{}{}Bool{}{}", pos, padding(), padding(), value)?;
                }
                Some(HIRInstruction::Add) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    writeln!(f, "{}{}Add", pos, padding())?;
                }
                Some(HIRInstruction::Sub) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    writeln!(f, "{}{}Sub", pos, padding())?;
                }
                Some(HIRInstruction::Mult) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    writeln!(f, "{}{}Mult", pos, padding())?;
                }
                Some(HIRInstruction::Div) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    writeln!(f, "{}{}Div", pos, padding())?;
                }
                Some(HIRInstruction::None) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    writeln!(f, "{}{}None", pos, padding())?;
                }
                Some(_) => {
                    let pos = match self.read_pos(){
                        Ok(pos) => pos,
                        Err(msg) => {
                            println!("{}", msg);
                            return Err(std::fmt::Error{})
                        }
                    };
                    writeln!(f, "{}{}Unknown", pos, padding())?;
                }
                None => break,
            }
            self.advance();
        };
        Ok(())
    }
}
