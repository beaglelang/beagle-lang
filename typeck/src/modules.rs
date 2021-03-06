use super::{
    Typeck
};

use stmt::Statement;

#[derive(Debug, Clone)]
pub struct Module{
    pub ident: String,
    pub statements: Vec<Statement>,
}

impl<'a> super::Check<'a> for Module{
    fn check(&self, typeck: &'a Typeck) -> Result<(), ()> {
        for statement in self.statements.iter(){
            if let Err(()) = statement.check(typeck){
                return Err(())
            }
        }
        Ok(())
    }
}