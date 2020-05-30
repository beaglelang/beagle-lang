use super::{
    Typeck
};

use stmt::Statement;

use notices::{
    Notice,
};

#[derive(Debug, Clone)]
pub struct Module{
    pub ident: String,
    pub statements: Vec<Statement>,
}

impl<'a> super::Check<'a> for Module{
    fn check(&self, typeck: &'a Typeck) -> Result<(), Notice> {
        for statement in self.statements.iter(){
            if let Err(notice) = statement.check(typeck){
                return Err(notice)
            }
        }
        Ok(())
    }
}