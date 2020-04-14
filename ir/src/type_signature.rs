use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeSignature {
    None,
    Untyped,
    Primitive(PrimitiveType),
    Function(FunctionSignature),
}

impl TypeSignature {
    #[inline]
    pub fn is_integer(&self) -> bool {
        matches!(self, Self::Primitive(PrimitiveType::Integer))
    }

    #[inline]
    pub fn is_float(&self) -> bool {
        matches!(self, Self::Primitive(PrimitiveType::Float))
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        matches!(self, Self::Primitive(PrimitiveType::Bool))
    }
}

impl Display for TypeSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Untyped => write!(f, "Untyped"),
            Self::Primitive(p) => write!(f, "{}", p),
            // Self::Struct(s) => write!(f, "{}", s),
            Self::Function(func) => write!(f, "{}", func),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimitiveType {
    None,
    Integer,
    Float,
    Bool,
    String,
    Unit,
}

impl PrimitiveType {
    pub fn new(type_string: &str) -> Self {
        match type_string {
            "Int" => Self::Integer,
            "Float" => Self::Float,
            "Bool" => Self::Bool,
            "String" => Self::String,
            "Unit" => Self::Unit,
            &_ => Self::None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature{
    pub parameters: Vec<TypeSignature>,
    pub return_type_signature: Box<TypeSignature>,
}

use std::fmt::{
    Display,
    Formatter,
    Result
};

impl Display for PrimitiveType {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self {
            Self::None => write!(f, "None"),
            Self::Bool => write!(f, "Bool"),
            Self::Integer => write!(f, "Int"),
            Self::Float => write!(f, "Float"),
            Self::String => write!(f, "Str"),
            Self::Unit => write!(f, "Unit")
        }
    }
}

impl Display for FunctionSignature {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "(")?;
        for (i, parameter) in self.parameters.iter().enumerate() {
            if i >= self.parameters.len() - 1{
                write!(f, "{}", parameter)?;
            }else{
                write!(f, "{}, ", parameter)?;
            }
        }
        write!(f, ") -> {}", self.return_type_signature)
    }
}
