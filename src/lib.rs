#[cfg(test)]
mod tests;

mod bytecode;
mod lex;
mod parse;
mod str;
mod value;
mod vm;

pub(crate) use self::{
    bytecode::{ByteCode, ByteCodeStack},
    lex::{LexError, Lexer, Token},
    parse::ParseProto,
    value::Value,
    vm::ExeState,
};

pub fn rua(source: &str) -> anyhow::Result<()> {
    let proto = ParseProto::new(source).parse()?;
    let mut state = ExeState::new();
    state.execute(proto)
}
