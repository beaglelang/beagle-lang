use ident::Identifier;

use ty::Ty;
use core::pos::BiPos;
use super::Statement;


#[derive(Debug, Clone)]
pub struct Fun{
    pub ident: Identifier,
    pub ty: Ty,
    pub params: Vec<FunParam>,
    pub body: Vec<Statement>,
    pub pos: BiPos,
}

#[derive(Debug, Clone)]
pub struct FunParam{
    pub ident: Identifier,
    pub ty: Ty,
    pub pos: BiPos
}