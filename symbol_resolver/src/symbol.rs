use crate::{
    fun::Fun,
    property::Property,
    local::Local
};

pub enum Symbol<'a>{
    Property(&'a Property),
    Fun(&'a Fun),
    Local(&'a Local)
}