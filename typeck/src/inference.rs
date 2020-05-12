use super::{
    Typeck,
};

pub trait Inference{
    fn infer_type(&self, typeck: &Typeck) -> Result<(),()>;
}