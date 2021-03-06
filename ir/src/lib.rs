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
        self.ins_ptr.replace(index);
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
        self.ins_ptr.clone().into_inner() < self.code.len()
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
        self.write_usize(bipos.offset.0);
        self.write_usize(bipos.offset.1);
        self.write_usize(bipos.line_region.0);
        self.write_usize(bipos.line_region.1);
    }

    pub fn read_pos(&self) -> Result<BiPos, String>{
        let start_line = match self.read_usize(){
            Ok(start_line) => start_line,
            Err(msg) => return Err(msg)
        };
        let start_col = match self.read_usize(){
            Ok(start_col) => start_col,
            Err(msg) => return Err(msg)
        };
        let end_line = match self.read_usize(){
            Ok(end_line) => end_line,
            Err(msg) => return Err(msg)
        };
        let end_col = match self.read_usize(){
            Ok(end_col) => end_col,
            Err(msg) => return Err(msg)
        };
        let offset_start = match self.read_usize(){
            Ok(offset_start) => offset_start,
            Err(msg) => return Err(msg)
        };
        let offset_end = match self.read_usize(){
            Ok(offset_end) => offset_end,
            Err(msg) => return Err(msg)
        };
        let line_region_start = match self.read_usize(){
            Ok(line_region_start) => line_region_start,
            Err(msg) => return Err(msg)
        };
        let line_region_end = match self.read_usize(){
            Ok(line_region_end) => line_region_end,
            Err(msg) => return Err(msg)
        };
        Ok(BiPos{
            start: Position(start_line, start_col),
            end: Position(end_line, end_col),
            offset: Position(offset_start, offset_end),
            line_region: Position(line_region_start, line_region_end)
        })
    }

    pub fn read_float(&self) -> f32{
        self.read_float_at(self.ins_ptr.clone().into_inner())
    }

    pub fn read_float_at(&self, idx: usize) -> f32{
        let float = f32::from_be_bytes(unsafe { *(self.code[idx..idx+4].as_ptr() as *const [u8; 4]) });
        self.inc_ins_ptr(5);
        return float
    }

    pub fn read_string(&self) -> &str{
        self.read_string_at(self.ins_ptr.clone().into_inner())
    }

    pub fn read_string_at(&self, idx: usize) -> &str{
        let length = self.read_int() as usize;
        let start = idx+4;
        let string= &self.code[start..start+length];
        self.inc_ins_ptr(length as usize);
        return std::str::from_utf8(string).unwrap()
    }

    pub fn write_string(&mut self, str: String){
        self.write_int(str.len() as i32);
        self.code.append(str.as_bytes().to_vec().as_mut())
    }

    pub fn write_str(&mut self, str: &str){
        self.write_int(str.len() as i32);
        self.code.append(str.as_bytes().to_vec().as_mut())
    }

    pub fn read_usize(&self) -> Result<usize, String>{
        self.read_usize_at(self.ins_ptr.clone().into_inner())
    }

    pub fn inc_ins_ptr(&self, amount: usize){
        let old = self.ins_ptr.clone().into_inner();
        self.ins_ptr.replace(old + amount);
    }
    
    pub fn dec_ins_ptr(&self, amount: usize){
        let old = self.ins_ptr.clone().into_inner();
        self.ins_ptr.replace(old - amount);
    }

    pub fn read_usize_at(&self, idx: usize) -> Result<usize, String>{
        if self.code.len() < idx || self.code.len() < idx+8{
            return Err(format!("Cannot read usize from chunk cause chunk length is less than given idx: {}", idx))
        }
        let float = usize::from_be_bytes(unsafe { *(self.code[idx..idx+8].as_ptr() as *const [u8; 8]) });
        self.inc_ins_ptr(std::mem::size_of::<usize>());
        return Ok(float)
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