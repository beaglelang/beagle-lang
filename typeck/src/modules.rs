use super::{
    statement::Statement,
    Typeck
};

#[derive(Debug, Clone)]
pub struct Module{
    pub ident: String,
    pub statements: Vec<Statement>,
}

impl<'a> super::Check<'a> for Module{
    fn check(&self, typeck: &'a Typeck) -> Result<(), ()> {
        for statement in self.statements.iter(){
            if statement.check(typeck).is_err(){
                return Err(())
            }
        }
        Ok(())
    }
}