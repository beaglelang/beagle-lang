use core::pos::BiPos;

///A part of a local or property whichi contains information about it's mutability. Properties use `var` while locals use `let mut`.
#[derive(Debug, Clone)]
pub struct Mutability{
    pub mutable: bool,
    pub pos: BiPos,
}
