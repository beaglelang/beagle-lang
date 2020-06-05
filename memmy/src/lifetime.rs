use crate::{
    expr::Expression
};

#[derive(Debug, Clone)]
pub struct ObjectLifetime<'a>{
    pub stages: Vec<LifetimeStage<'a>>
}

#[derive(Debug, Clone)]
pub struct LifetimeStage<'a>{
    pub references: Vec<Reference<'a>>,
}

#[derive(Debug, Clone)]
pub struct Reference<'a>{
    pub reference: &'a Expression,
    pub kind: RefKind
}

#[derive(Debug, Clone)]
pub enum RefKind{
    PropertyRef,
    LocalRef,
    FunctionRef,
    RefRef
}