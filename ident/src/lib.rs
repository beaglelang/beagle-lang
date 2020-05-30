use core::pos::BiPos;
///A part of an IR that contains an identifier.
#[derive(Debug, Clone)]
pub struct Identifier{
    ///The identifier
    pub ident: String,
    ///The in source location of the identifier.
    pub pos: BiPos,
}
