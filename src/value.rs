use crate::ExeState;

pub type LuaFunc = fn(&mut ExeState) -> i32;

#[derive(Clone, PartialEq)]
pub enum Value {
    Nil,
    Boolean(bool),
    Integer(i64),
    Float(f64),
    String(String),
    Function(LuaFunc),
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
            Self::String(s) => f.write_str(s),
            Self::Function(func) => write!(f, "function: {func:#x?}"),
        }
    }
}

impl Value {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }
}
