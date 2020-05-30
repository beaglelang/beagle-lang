use ident::Identifier;
use mutable::Mutability;
use ty::Ty;
use core::pos::BiPos;
use expr::Expr;

use std::cell::RefCell;

#[derive(Debug, Clone)]
pub struct Local{
    pub ident: Identifier,
    pub ty: RefCell<Ty>,
    pub expr: Expr,
    pub pos: BiPos,
    pub mutable: Mutability
}
