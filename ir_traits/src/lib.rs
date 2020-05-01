
pub trait Instruction{}

pub trait ReadInstruction<T> where T: Instruction{
    fn read_instruction(&self) -> Option<T>;
}

pub trait WriteInstruction<T> where T: Instruction{
    fn write_instruction(&mut self, ins: T);
}
