#[derive(Debug, Clone)]
pub struct IRElement<'i>{
    name: &'i str,
    opcode: u8
}
