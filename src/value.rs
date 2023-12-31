use smol_str::SmolStr;

use crate::str::LossyStr;
use crate::ExeState;

pub type LuaFunc = fn(&mut ExeState) -> i32;

#[derive(Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(LossyStr),
    Function(LuaFunc),
    Identifier(SmolStr),
}

impl Default for Value {
    fn default() -> Self {
        Self::Nil
    }
}

impl std::fmt::Debug for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => f.write_str("nil"),
            Self::Boolean(b) => write!(f, "{b}"),
            Self::Integer(i) => write!(f, "{i}"),
            Self::Float(x) => write!(f, "{x:?}"),
            Self::String(s) => write!(f, "{s}"),
            Self::Identifier(s) => f.write_str(s),
            Self::Function(func) => write!(f, "function: {func:#x?}"),
        }
    }
}

impl Value {
    pub fn as_identifier(&self) -> Option<&SmolStr> {
        match self {
            Self::Identifier(s) => Some(s),
            _ => None,
        }
    }
}
