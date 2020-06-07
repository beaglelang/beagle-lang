use crate::statement::Statement;

pub struct Module{
    pub ident: String,
    pub statements: Vec<Statement>,
}