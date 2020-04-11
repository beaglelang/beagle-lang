use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeSignature{
    None,
    Untyped,
    Primitive(PrimitiveType),
}

impl TypeSignature{
    #[inline]
    pub fn is_integer(&self) -> bool{
        matches!(self, Self::Primitive(PrimitiveType::Integer))
    }

    #[inline]
    pub fn is_float(&self) -> bool{
        matches!(self, Self::Primitive(PrimitiveType::Float))
    }

    #[inline]
    pub fn is_bool(&self) -> bool{
        matches!(self, Self::Primitive(PrimitiveType::Bool))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimitiveType{
    None,
    Integer,
    Float,
    Bool,
}

impl PrimitiveType{
    pub fn new(type_string: &str) -> Self{
        match type_string{
            "Int" => Self::Integer,
            "Float" => Self::Float,
            "Bool" => Self::Bool,
            &_ => Self::None
        }
    }
}