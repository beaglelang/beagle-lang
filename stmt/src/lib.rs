pub mod fun;
use fun::Fun;
pub mod local;
use local::Local;
pub mod property;
use property::Property;
pub mod modules;
// use modules::Module;

use core::pos::BiPos;

#[derive(Debug, Clone)]
pub struct Statement{
    pub kind: StatementKind,
    pub pos: BiPos,
}

#[derive(Debug, Clone)]
pub enum StatementKind{
    Property(Property),
    Fun(Fun),
    Local(Local),
}
