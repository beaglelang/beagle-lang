use ir_derive::{
    Instruction,
    ReadInstruction,
    WriteInstruction,
};
use ir_traits::{
    Instruction,
    ReadInstruction,
    WriteInstruction,
};

use super::Chunk;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

use serde::{Serialize, Deserialize};

#[derive(FromPrimitive, Instruction, ReadInstruction, WriteInstruction, Debug, Clone, Serialize, Deserialize, PartialEq)]
#[repr(u8)]
pub enum MIRInstructions{
    ///Module start
    Module,
    ///End module
    EndModule,

    ///Function start. 
    ///At the beginning is where local variable preallocation will occur.
    Fun,
    ///End function.
    ///This is where all drops to local variables and any references or values passed as arguments will occur.
    EndFun,

    //Function param.
    //The call to the containing function will handle the pass-by.
    FunParam,
    //Literals
    ///Integer literal
    Integer,
    ///Float literal
    Float,
    ///String literal
    String,
    ///Boolean literal
    Bool,
    ///Unit type
    Unit,
    ///Initialize object `name` with `mutability`.
    ///An allocation instruction must precede this with the size of the object.
    ///Following this will be a call to the initializer and its mutability.
    ObjInit,
    ///Drop `name`. This can either be a value or a reference.
    ///The drop mechanism is smart. If what is being dropped is a reference, 
    ///the reference counter will decrement the count for object `name`.
    Drop,

    //Memory management instructions
    ///Create reference for `refee`.
    ///High level references to properties will result in this instruction.
    Ref,
    ///Move `name`.
    ///A single reference to a local variables will result in this instruction.
    Move,
    ///Copy `name`.
    ///Where n is the number of references to a local variable, all references until n-1 will result in this instruction,
    /// whereas the final reference to a local variable will result in a Move instruction.
    Copy,
    ///Heap allocation of `size` for object `name`
    HeapAlloc,
    ///Stack allocation of `size` for object `name`.
    ///Either an object contruction or a lateinit instruction must proceed this.
    StackAlloc,
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
    ObjMut,
    ///Halt compiler
    Halt
}