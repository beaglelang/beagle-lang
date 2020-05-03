use serde::{Deserialize, Serialize};

use core::pos::{
    BiPos,
    Position,
};

pub mod hir;
pub mod mir;

#[repr(u8)]
#[derive(Debug, Clone, PartialEq)]
pub enum Constant<'a>{
    Str(&'a str),
    Int(i32),
    Float(f32),
    Bool(bool),
}

use std::cell::RefCell;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chunk{
    pub code: Vec<u8>,
    pub ins_ptr: RefCell<usize>
}

impl Chunk{
    pub fn new() -> Self{
        Self{
            code: Vec::with_capacity(4),
            ins_ptr: RefCell::new(0)
        }
    }

    pub fn set_ins_ptr(&self, index: usize){
        let mut _ins_ptr = self.ins_ptr.clone();
        _ins_ptr.replace(index);
    }

    pub fn jump_to(&self, index: usize) -> Result<(),String>{
        if index >= self.length(){
            return Err(format!("Cannot jump beyond the length of the bytecode: {}", index))
        }
        self.set_ins_ptr(index);
        Ok(())
    }

    pub fn advance(&self){
        self.inc_ins_ptr(1);
    }

    pub fn can_read(&self) -> bool{
        self.ins_ptr.clone().into_inner() + 1 < self.code.len()
    }

    pub fn peek(&self) -> Option<&u8>{
        self.code.get(self.ins_ptr.clone().into_inner() + 1)
    }

    pub fn length(&self) -> usize{
        self.code.len()
    }

    pub fn get_current(&self) -> u8{
        self.code[self.ins_ptr.clone().into_inner()]
    }

    pub fn get_at_index(&self, index: usize) -> u8{
        self.code[index]
    }

    pub fn read_int(&self) -> i32{
        self.read_int_at(self.ins_ptr.clone().into_inner())
    }

    pub fn read_int_at(&self, idx: usize) -> i32{
        let int = i32::from_be_bytes(unsafe { *(self.code[idx..idx+4].as_ptr() as *const [u8; 4]) });
        self.inc_ins_ptr(4);
        return int;
    }

    pub fn write_pos(&mut self, bipos: BiPos){
        self.write_usize(bipos.start.0);
        self.write_usize(bipos.start.1);
        self.write_usize(bipos.end.0);
        self.write_usize(bipos.end.1);
    }

    pub fn read_pos(&self) -> BiPos{
        let start_line = self.read_usize();
        let start_col = self.read_usize();
        let end_line = self.read_usize();
        let end_col = self.read_usize();
        BiPos{
            start: Position(start_line, start_col),
            end: Position(end_line, end_col)
        }
    }

    pub fn read_float(&self) -> f32{
        self.read_float_at(self.ins_ptr.clone().into_inner())
    }

    pub fn read_float_at(&self, idx: usize) -> f32{
        let float = f32::from_be_bytes(unsafe { *(self.code[idx..idx+4].as_ptr() as *const [u8; 4]) });
        self.inc_ins_ptr(5);
        return float
    }

    pub fn read_string(&self) -> String{
        self.read_string_at(self.ins_ptr.clone().into_inner())
    }

    pub fn read_string_at(&self, idx: usize) -> String{
        let length = self.read_int();
        let string = unsafe { String::from_raw_parts(self.code[idx+4..].as_ptr() as *mut u8, length as usize, length as usize) };
        self.inc_ins_ptr(length as usize);
        return string
    }

    pub fn write_string(&mut self, str: String){
        self.code.extend((str.len() as i32).to_be_bytes().iter());
        self.code.extend(str.as_bytes())
    }

    pub fn write_str(&mut self, str: &str){
        self.code.extend((str.len() as i32).to_be_bytes().iter());
        self.code.extend(str.as_bytes())
    }

    pub fn read_usize(&self) -> usize{
        self.read_usize_at(self.ins_ptr.clone().into_inner())
    }

    pub fn inc_ins_ptr(&self, amount: usize){
        let old = self.ins_ptr.clone().into_inner();
        self.ins_ptr.replace(old + amount);
    }

    pub fn read_usize_at(&self, idx: usize) -> usize{
        let float = usize::from_be_bytes(unsafe { *(self.code[idx..idx+8].as_ptr() as *const [u8; 8]) });
        self.inc_ins_ptr(std::mem::size_of::<usize>());
        return float
    }

    pub fn write_usize(&mut self, size: usize){
        self.code.extend(size.to_be_bytes().iter())
    }

    pub fn write_int(&mut self, int: i32){
        self.code.extend(int.to_be_bytes().iter())
    }

    pub fn write_float(&mut self, float: f32){
        self.code.extend(float.to_be_bytes().iter())
    }

    pub fn write_double(&mut self, double: f64){
        self.code.extend(double.to_be_bytes().iter())
    }

    pub fn write_bool(&mut self, boolean: bool){
        self.code.push(boolean as u8)
    }
    
    pub fn read_bool(&self) -> bool{
        let value = self.read_byte();
        value == b'\x01'
    }

    pub fn write_byte(&mut self, byte: u8){
        self.code.push(byte)
    }

    pub fn read_byte(&self) -> u8{
        let b = self.get_current();
        self.advance();
        b
    }

    pub fn write_chunk(&mut self, chunk: Self){
        self.code.extend(chunk.code)
    }
}

impl Iterator for Chunk{
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if !self.can_read(){
            return None
        }
        self.advance();
        Some(self.get_current())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    /// The name of the module
    pub name: String,
    ///The module's bytecode chunk
    pub chunk: Chunk
}

impl Module {
    pub fn new(name: String) -> Self {
        Module {
            name: name.to_string(),
            chunk: Chunk::new()
        }
    }
}